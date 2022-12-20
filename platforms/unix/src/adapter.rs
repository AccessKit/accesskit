// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{kurbo::Rect, ActionHandler, NodeId, Role, TreeUpdate};
use accesskit_consumer::{Node, Tree, TreeChangeHandler};
use atspi::{Interface, InterfaceSet, State};
use crate::{
    atspi::{
        interfaces::{
            AccessibleInterface, ActionInterface, ComponentInterface,
            ObjectEvent, QueuedEvent, ValueInterface, WindowEvent,
        },
        Bus, ObjectId, ACCESSIBLE_PATH_PREFIX,
    },
    node::{filter, NodeWrapper, PlatformNode, PlatformRootNode},
    util::{AppContext, WindowBounds},
};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Adapter {
    atspi_bus: Option<Bus>,
    _app_context: Arc<RwLock<AppContext>>,
    root_window_bounds: Arc<RwLock<WindowBounds>>,
    tree: Arc<Tree>,
}

impl Adapter {
    pub fn new(
        app_name: String,
        toolkit_name: String,
        toolkit_version: String,
        initial_state: Box<dyn FnOnce() -> TreeUpdate>,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let mut atspi_bus = Bus::a11y_bus();
        let tree = Arc::new(Tree::new(initial_state(), action_handler));
        let app_context = Arc::new(RwLock::new(AppContext::new(
            app_name,
            toolkit_name,
            toolkit_version,
        )));
        atspi_bus.as_mut().and_then(|bus| {
            bus.register_root_node(PlatformRootNode::new(&app_context, &tree))
                .ok()
        });
        let adapter = Adapter {
            atspi_bus,
            _app_context: app_context,
            root_window_bounds: Arc::new(RwLock::new(WindowBounds::default())),
            tree,
        };
        {
            let reader = adapter.tree.read();
            let mut objects_to_add = Vec::new();

            fn add_children(node: Node<'_>, to_add: &mut Vec<NodeId>) {
                for child in node.filtered_children(&filter) {
                    to_add.push(child.id());
                    add_children(child, to_add);
                }
            }

            objects_to_add.push(reader.root().id());
            add_children(reader.root(), &mut objects_to_add);
            for id in objects_to_add {
                let interfaces = NodeWrapper::new(&reader.node_by_id(id).unwrap())
                        .interfaces();
                adapter
                    .register_interfaces(&adapter.tree, id, interfaces)
                    .unwrap();
            }
        }
        adapter
    }

    fn register_interfaces(
        &self,
        tree: &Arc<Tree>,
        id: NodeId,
        new_interfaces: InterfaceSet,
    ) -> zbus::Result<bool> {
        self.atspi_bus.as_ref().map_or(Ok(false), |bus| {
            let atspi_id = ObjectId::from(id);
            let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, atspi_id.as_str());
            if new_interfaces.contains(Interface::Accessible) {
                bus.register_interface(
                    &path,
                    AccessibleInterface::new(
                        bus.unique_name().to_owned(),
                        PlatformNode::new(tree, id),
                    ),
                )?;
            }
            if new_interfaces.contains(Interface::Action) {
                bus.register_interface(
                    &path,
                    ActionInterface::new(PlatformNode::new(tree, id)),
                )?;
            }
            if new_interfaces.contains(Interface::Component) {
                bus.register_interface(
                    &path,
                    ComponentInterface::new(PlatformNode::new(tree, id), &self.root_window_bounds),
                )?;
            }
            if new_interfaces.contains(Interface::Value) {
                bus.register_interface(
                    &path,
                    ValueInterface::new(PlatformNode::new(tree, id)),
                )?;
            }
            Ok(true)
        })
    }

    fn unregister_interfaces(
        &self,
        id: &ObjectId,
        old_interfaces: InterfaceSet,
    ) -> zbus::Result<bool> {
        self.atspi_bus.as_ref().map_or(Ok(false), |bus| {
            let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, id.as_str());
            if old_interfaces.contains(Interface::Accessible) {
                bus.unregister_interface::<AccessibleInterface<PlatformNode>>(&path)?;
            }
            if old_interfaces.contains(Interface::Action) {
                bus.unregister_interface::<ActionInterface>(&path)?;
            }
            if old_interfaces.contains(Interface::Component) {
                bus.unregister_interface::<ComponentInterface>(&path)?;
            }
            if old_interfaces.contains(Interface::Value) {
                bus.unregister_interface::<ValueInterface>(&path)?;
            }
            Ok(true)
        })
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        let mut bounds = self.root_window_bounds.write();
        bounds.outer = outer;
        bounds.inner = inner;
    }

    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        struct Handler<'a> {
            adapter: &'a Adapter,
            tree: &'a Arc<Tree>,
            queue: Vec<QueuedEvent>,
        }
        impl TreeChangeHandler for Handler<'_> {
            fn node_added(&mut self, node: &Node) {
                let interfaces = NodeWrapper::new(node).interfaces();
                self.adapter
                    .register_interfaces(self.tree, node.id(), interfaces)
                    .unwrap();
            }
            fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
                let old_wrapper = NodeWrapper::new(old_node);
                let new_wrapper = NodeWrapper::new(new_node);
                let old_interfaces = old_wrapper.interfaces();
                let new_interfaces = new_wrapper.interfaces();
                let kept_interfaces = old_interfaces & new_interfaces;
                self.adapter
                    .unregister_interfaces(&new_wrapper.id(), old_interfaces ^ kept_interfaces)
                    .unwrap();
                self.adapter
                    .register_interfaces(self.tree, new_node.id(), new_interfaces ^ kept_interfaces)
                    .unwrap();
                new_wrapper.enqueue_changes(&mut self.queue, &old_wrapper);
            }
            fn focus_moved(&mut self, old_node: Option<&Node>, new_node: Option<&Node>) {
                let old_window = old_node.and_then(|node| containing_window(*node));
                let new_window = new_node.and_then(|node| containing_window(*node));
                if old_window.map(|n| n.id()) != new_window.map(|n| n.id()) {
                    if let Some(window) = old_window {
                        self.adapter.window_deactivated(
                            &NodeWrapper::new(&window),
                            &mut self.queue,
                        );
                    }
                    if let Some(window) = new_window {
                        self.adapter.window_activated(
                            &NodeWrapper::new(&window),
                            &mut self.queue,
                        );
                    }
                }
                if let Some(node) = new_node.map(NodeWrapper::new) {
                    self.queue.push(QueuedEvent::Object {
                        target: node.id(),
                        event: ObjectEvent::StateChanged(State::Focused, true),
                    });
                }
                if let Some(node) = old_node.map(NodeWrapper::new) {
                    self.queue.push(QueuedEvent::Object {
                        target: node.id(),
                        event: ObjectEvent::StateChanged(State::Focused, false),
                    });
                }
            }
            fn node_removed(&mut self, node: &Node) {
                let node = NodeWrapper::new(node);
                self.adapter
                    .unregister_interfaces(&node.id(), node.interfaces())
                    .unwrap();
            }
        }
        let mut handler = Handler {
            adapter: self,
            tree: &self.tree,
            queue: Vec::new(),
        };
        self.tree.update_and_process_changes(update, &mut handler);
        QueuedEvents {
            bus: self.atspi_bus.clone(),
            queue: handler.queue,
        }
    }

    fn window_activated(&self, window: &NodeWrapper, queue: &mut Vec<QueuedEvent>) {
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

    fn window_deactivated(&self, window: &NodeWrapper, queue: &mut Vec<QueuedEvent>) {
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
pub struct QueuedEvents {
    bus: Option<Bus>,
    queue: Vec<QueuedEvent>,
}

impl QueuedEvents {
    pub fn raise(&self) {
        if let Some(bus) = &self.bus {
            for event in &self.queue {
                let _ = match &event {
                    QueuedEvent::Object { target, event } => bus.emit_object_event(target, event),
                    QueuedEvent::Window {
                        target,
                        name,
                        event,
                    } => bus.emit_window_event(target, name, event),
                };
            }
        }
    }
}
