// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::sync::Arc;

use accesskit::{ActionHandler, Role, TreeUpdate};
use accesskit_consumer::{Node, Tree, TreeChange};

use crate::atspi::{
    interfaces::{
        AccessibleInterface, ActionInterface, Interface, Interfaces, ObjectEvent, QueuedEvent,
        ValueInterface, WindowEvent,
    },
    Bus, State, ACCESSIBLE_PATH_PREFIX,
};
use crate::node::{AppState, PlatformNode, PlatformRootNode, ResolvedPlatformNode};
use parking_lot::RwLock;

pub struct Adapter<'a> {
    atspi_bus: Bus<'a>,
    _app_state: Arc<RwLock<AppState>>,
    tree: Arc<Tree>,
}

impl<'a> Adapter<'a> {
    pub fn new(
        app_name: String,
        toolkit_name: String,
        toolkit_version: String,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Option<Self> {
        let mut atspi_bus = Bus::a11y_bus()?;
        let tree = Tree::new(initial_state, action_handler);
        let app_state = Arc::new(RwLock::new(AppState::new(
            app_name,
            toolkit_name,
            toolkit_version,
        )));
        atspi_bus
            .register_root_node(PlatformRootNode::new(
                Arc::downgrade(&app_state),
                Arc::downgrade(&tree),
            ))
            .ok()?;
        let adapter = Adapter {
            atspi_bus,
            _app_state: app_state,
            tree,
        };
        {
            let reader = adapter.tree.read();
            let mut objects_to_add = Vec::new();

            fn add_children<'b>(node: Node<'b>, to_add: &mut Vec<ResolvedPlatformNode<'b>>) {
                for child in node.unignored_children() {
                    to_add.push(ResolvedPlatformNode::new(child));
                    add_children(child, to_add);
                }
            }

            objects_to_add.push(ResolvedPlatformNode::new(reader.root()));
            add_children(reader.root(), &mut objects_to_add);
            for node in objects_to_add {
                adapter.register_interfaces(&node, node.interfaces()).ok()?;
            }
        }
        Some(adapter)
    }

    fn register_interfaces(
        &self,
        node: &ResolvedPlatformNode,
        new_interfaces: Interfaces,
    ) -> zbus::Result<bool> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, node.id().as_str());
        if new_interfaces.contains(Interface::Accessible) {
            self.atspi_bus.register_interface(
                &path,
                AccessibleInterface::new(self.atspi_bus.unique_name().to_owned(), node.downgrade()),
            )?;
        }
        if new_interfaces.contains(Interface::Action) {
            self.atspi_bus
                .register_interface(&path, ActionInterface::new(node.downgrade()))?;
        }
        if new_interfaces.contains(Interface::Value) {
            self.atspi_bus
                .register_interface(&path, ValueInterface::new(node.downgrade()))?;
        }
        Ok(true)
    }

    fn unregister_interfaces(
        &self,
        node: &ResolvedPlatformNode,
        old_interfaces: Interfaces,
    ) -> zbus::Result<bool> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, node.id().as_str());
        if old_interfaces.contains(Interface::Action) {
            self.atspi_bus
                .unregister_interface::<ActionInterface>(&path)?;
        }
        if old_interfaces.contains(Interface::Value) {
            self.atspi_bus
                .unregister_interface::<ValueInterface>(&path)?;
        }
        Ok(true)
    }

    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let mut queue = Vec::new();
        self.tree.update_and_process_changes(update, |change| {
            match change {
                TreeChange::FocusMoved { old_node, new_node } => {
                    let old_window = old_node.and_then(|node| containing_window(node));
                    let new_window = new_node.and_then(|node| containing_window(node));
                    if old_window.map(|n| n.id()) != new_window.map(|n| n.id()) {
                        if let Some(window) = old_window {
                            self.window_deactivated(&ResolvedPlatformNode::new(window), &mut queue);
                        }
                        if let Some(window) = new_window {
                            self.window_activated(&ResolvedPlatformNode::new(window), &mut queue);
                        }
                    }
                    if let Some(node) = new_node.map(|node| ResolvedPlatformNode::new(node)) {
                        queue.push(QueuedEvent::Object {
                            target: node.id(),
                            event: ObjectEvent::StateChanged(State::Focused, true),
                        });
                        queue.push(QueuedEvent::Focus(node.id()));
                    }
                    if let Some(node) = old_node.map(|node| ResolvedPlatformNode::new(node)) {
                        queue.push(QueuedEvent::Object {
                            target: node.id(),
                            event: ObjectEvent::StateChanged(State::Focused, false),
                        });
                    }
                }
                TreeChange::NodeUpdated { old_node, new_node } => {
                    let old_node = ResolvedPlatformNode::new(old_node);
                    let new_node = ResolvedPlatformNode::new(new_node);
                    let old_interfaces = old_node.interfaces();
                    let new_interfaces = new_node.interfaces();
                    let kept_interfaces = old_interfaces & new_interfaces;
                    self.unregister_interfaces(&new_node, old_interfaces ^ kept_interfaces)
                        .unwrap();
                    self.register_interfaces(&new_node, new_interfaces ^ kept_interfaces)
                        .unwrap();
                    new_node.enqueue_changes(&mut queue, &old_node);
                }
                // TODO: handle other events
                _ => (),
            };
        });
        QueuedEvents {
            bus: self.atspi_bus.clone(),
            queue,
        }
    }

    fn window_activated(&self, window: &ResolvedPlatformNode, queue: &mut Vec<QueuedEvent>) {
        queue.push(QueuedEvent::Window {
            target: window.id(),
            name: window.name(),
            event: WindowEvent::Activated,
        });
        queue.push(QueuedEvent::Object {
            target: window.id(),
            event: ObjectEvent::StateChanged(State::Active, true),
        });
    }

    fn window_deactivated(&self, window: &ResolvedPlatformNode, queue: &mut Vec<QueuedEvent>) {
        queue.push(QueuedEvent::Window {
            target: window.id(),
            name: window.name(),
            event: WindowEvent::Deactivated,
        });
        queue.push(QueuedEvent::Object {
            target: window.id(),
            event: ObjectEvent::StateChanged(State::Active, false),
        });
    }

    fn root_platform_node(&self) -> PlatformNode {
        let reader = self.tree.read();
        let node = reader.root();
        PlatformNode::new(&node)
    }
}

fn containing_window(node: Node) -> Option<Node> {
    const WINDOW_ROLES: &[Role] = &[Role::AlertDialog, Role::Dialog, Role::Window];
    if WINDOW_ROLES.contains(&node.role()) {
        Some(node)
    } else {
        while let Some(node) = node.parent() {
            if WINDOW_ROLES.contains(&node.role()) {
                return Some(node);
            }
        }
        None
    }
}

#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents<'a> {
    bus: Bus<'a>,
    queue: Vec<QueuedEvent>,
}

impl<'a> QueuedEvents<'a> {
    pub fn raise(&self) {
        for event in &self.queue {
            let _ = match &event {
                QueuedEvent::Focus(target) => self.bus.emit_focus_event(target),
                QueuedEvent::Object { target, event } => self.bus.emit_object_event(target, event),
                QueuedEvent::Window {
                    target,
                    name,
                    event,
                } => self.bus.emit_window_event(target, name, event),
            };
        }
    }
}
