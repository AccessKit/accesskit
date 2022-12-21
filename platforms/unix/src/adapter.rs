// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{
        interfaces::{
            AccessibleInterface, ActionInterface, ComponentInterface, ObjectEvent, QueuedEvent,
            ValueInterface, WindowEvent,
        },
        Bus, ObjectId, ACCESSIBLE_PATH_PREFIX,
    },
    node::{filter, NodeWrapper, PlatformNode, PlatformRootNode},
    util::{AppContext, WindowBounds},
};
use accesskit::{kurbo::Rect, ActionHandler, NodeId, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, Node, Tree, TreeChangeHandler, TreeState};
use atspi::{Interface, InterfaceSet, State};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Adapter {
    atspi_bus: Bus,
    _app_context: Arc<RwLock<AppContext>>,
    root_window_bounds: Arc<RwLock<WindowBounds>>,
    tree: Arc<Tree>,
}

impl Adapter {
    pub fn new(
        app_name: String,
        toolkit_name: String,
        toolkit_version: String,
        initial_state: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Option<Self> {
        let mut atspi_bus = Bus::a11y_bus()?;
        let tree = Arc::new(Tree::new(initial_state(), action_handler));
        let app_context = Arc::new(RwLock::new(AppContext::new(
            app_name,
            toolkit_name,
            toolkit_version,
        )));
        atspi_bus
            .register_root_node(PlatformRootNode::new(&app_context, &tree))
            .ok()?;
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
                let interfaces = NodeWrapper::Node(&reader.node_by_id(id).unwrap()).interfaces();
                adapter
                    .register_interfaces(&adapter.tree, id, interfaces)
                    .unwrap();
            }
        }
        Some(adapter)
    }

    fn register_interfaces(
        &self,
        tree: &Arc<Tree>,
        id: NodeId,
        new_interfaces: InterfaceSet,
    ) -> zbus::Result<bool> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, ObjectId::from(id).as_str());
        if new_interfaces.contains(Interface::Accessible) {
            self.atspi_bus.register_interface(
                &path,
                AccessibleInterface::new(
                    self.atspi_bus.unique_name().to_owned(),
                    PlatformNode::new(tree, id),
                ),
            )?;
        }
        if new_interfaces.contains(Interface::Action) {
            self.atspi_bus
                .register_interface(&path, ActionInterface::new(PlatformNode::new(tree, id)))?;
        }
        if new_interfaces.contains(Interface::Component) {
            self.atspi_bus.register_interface(
                &path,
                ComponentInterface::new(PlatformNode::new(tree, id), &self.root_window_bounds),
            )?;
        }
        if new_interfaces.contains(Interface::Value) {
            self.atspi_bus
                .register_interface(&path, ValueInterface::new(PlatformNode::new(tree, id)))?;
        }
        Ok(true)
    }

    fn unregister_interfaces(
        &self,
        id: &ObjectId,
        old_interfaces: InterfaceSet,
    ) -> zbus::Result<bool> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, id.as_str());
        if old_interfaces.contains(Interface::Accessible) {
            self.atspi_bus
                .unregister_interface::<AccessibleInterface<PlatformNode>>(&path)?;
        }
        if old_interfaces.contains(Interface::Action) {
            self.atspi_bus
                .unregister_interface::<ActionInterface>(&path)?;
        }
        if old_interfaces.contains(Interface::Component) {
            self.atspi_bus
                .unregister_interface::<ComponentInterface>(&path)?;
        }
        if old_interfaces.contains(Interface::Value) {
            self.atspi_bus
                .unregister_interface::<ValueInterface>(&path)?;
        }
        Ok(true)
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
                let interfaces = NodeWrapper::Node(node).interfaces();
                self.adapter
                    .register_interfaces(self.tree, node.id(), interfaces)
                    .unwrap();
            }
            fn node_updated(&mut self, old_node: &DetachedNode, new_node: &Node) {
                let old_wrapper = NodeWrapper::DetachedNode(old_node);
                let new_wrapper = NodeWrapper::Node(new_node);
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
            fn focus_moved(&mut self, old_node: Option<&DetachedNode>, new_node: Option<&Node>) {
                if let Some(root_window) = root_window(&self.tree.read()) {
                    if old_node.is_none() && new_node.is_some() {
                        self.adapter
                            .window_activated(&NodeWrapper::Node(&root_window), &mut self.queue);
                    } else if old_node.is_some() && new_node.is_none() {
                        self.adapter
                            .window_deactivated(&NodeWrapper::Node(&root_window), &mut self.queue);
                    }
                }
                if let Some(node) = new_node.map(NodeWrapper::Node) {
                    self.queue.push(QueuedEvent::Object {
                        target: node.id(),
                        event: ObjectEvent::StateChanged(State::Focused, true),
                    });
                }
                if let Some(node) = old_node.map(NodeWrapper::DetachedNode) {
                    self.queue.push(QueuedEvent::Object {
                        target: node.id(),
                        event: ObjectEvent::StateChanged(State::Focused, false),
                    });
                }
            }
            fn node_removed(&mut self, node: &DetachedNode, _: &TreeState) {
                let node = NodeWrapper::DetachedNode(node);
                self.queue.push(QueuedEvent::Object {
                    target: node.id(),
                    event: ObjectEvent::StateChanged(State::Defunct, true),
                });
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

fn root_window(current_state: &TreeState) -> Option<Node> {
    const WINDOW_ROLES: &[Role] = &[Role::AlertDialog, Role::Dialog, Role::Window];
    let root = current_state.root();
    if WINDOW_ROLES.contains(&root.role()) {
        Some(root)
    } else {
        None
    }
}

#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents {
    bus: Bus,
    queue: Vec<QueuedEvent>,
}

impl QueuedEvents {
    pub fn raise(&self) {
        for event in &self.queue {
            let _ = match &event {
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
