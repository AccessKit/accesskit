// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{
        interfaces::{
            AccessibleInterface, ActionInterface, ComponentInterface, Event, ObjectEvent,
            ValueInterface, WindowEvent,
        },
        Bus, ObjectId, ACCESSIBLE_PATH_PREFIX,
    },
    node::{filter, filter_detached, NodeWrapper, PlatformNode, PlatformRootNode},
    util::{AppContext, WindowBounds},
};
use accesskit::{kurbo::Rect, ActionHandler, NodeId, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use atspi::{Interface, InterfaceSet, State};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use parking_lot::RwLock;
use std::sync::Arc;
use zbus::Task;

pub struct Adapter {
    atspi_bus: Bus,
    _event_task: Task<()>,
    events: UnboundedSender<Event>,
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
        let (event_sender, event_receiver) = mpsc::unbounded();
        let atspi_bus_copy = atspi_bus.clone();
        let event_task = atspi_bus.connection().inner().executor().spawn(
            async move {
                handle_events(atspi_bus_copy, event_receiver).await;
            },
            "accesskit_event_task",
        );
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
            _event_task: event_task,
            events: event_sender,
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

    pub fn update(&self, update: TreeUpdate) {
        struct Handler<'a> {
            adapter: &'a Adapter,
            tree: &'a Arc<Tree>,
        }
        impl Handler<'_> {
            fn add_node(&mut self, node: &Node) {
                let interfaces = NodeWrapper::Node(node).interfaces();
                self.adapter
                    .register_interfaces(self.tree, node.id(), interfaces)
                    .unwrap();
            }
            fn remove_node(&mut self, node: &DetachedNode) {
                let node = NodeWrapper::DetachedNode(node);
                self.adapter
                    .events
                    .unbounded_send(Event::Object {
                        target: node.id(),
                        event: ObjectEvent::StateChanged(State::Defunct, true),
                    })
                    .unwrap();
                self.adapter
                    .unregister_interfaces(&node.id(), node.interfaces())
                    .unwrap();
            }
        }
        impl TreeChangeHandler for Handler<'_> {
            fn node_added(&mut self, node: &Node) {
                if filter(node) == FilterResult::Include {
                    self.add_node(node);
                }
            }
            fn node_updated(&mut self, old_node: &DetachedNode, new_node: &Node) {
                let filter_old = filter_detached(old_node);
                let filter_new = filter(new_node);
                if filter_new != filter_old {
                    if filter_new == FilterResult::Include {
                        self.add_node(new_node);
                    } else if filter_old == FilterResult::Include {
                        self.remove_node(old_node);
                    }
                } else if filter_new == FilterResult::Include {
                    let old_wrapper = NodeWrapper::DetachedNode(old_node);
                    let new_wrapper = NodeWrapper::Node(new_node);
                    let old_interfaces = old_wrapper.interfaces();
                    let new_interfaces = new_wrapper.interfaces();
                    let kept_interfaces = old_interfaces & new_interfaces;
                    self.adapter
                        .unregister_interfaces(&new_wrapper.id(), old_interfaces ^ kept_interfaces)
                        .unwrap();
                    self.adapter
                        .register_interfaces(
                            self.tree,
                            new_node.id(),
                            new_interfaces ^ kept_interfaces,
                        )
                        .unwrap();
                    new_wrapper.notify_changes(
                        &self.adapter.root_window_bounds.read(),
                        &self.adapter.events,
                        &old_wrapper,
                    );
                }
            }
            fn focus_moved(&mut self, old_node: Option<&DetachedNode>, new_node: Option<&Node>) {
                if let Some(root_window) = root_window(&self.tree.read()) {
                    if old_node.is_none() && new_node.is_some() {
                        self.adapter.window_activated(
                            &NodeWrapper::Node(&root_window),
                            &self.adapter.events,
                        );
                    } else if old_node.is_some() && new_node.is_none() {
                        self.adapter.window_deactivated(
                            &NodeWrapper::Node(&root_window),
                            &self.adapter.events,
                        );
                    }
                }
                if let Some(node) = new_node.map(NodeWrapper::Node) {
                    self.adapter
                        .events
                        .unbounded_send(Event::Object {
                            target: node.id(),
                            event: ObjectEvent::StateChanged(State::Focused, true),
                        })
                        .unwrap();
                }
                if let Some(node) = old_node.map(NodeWrapper::DetachedNode) {
                    self.adapter
                        .events
                        .unbounded_send(Event::Object {
                            target: node.id(),
                            event: ObjectEvent::StateChanged(State::Focused, false),
                        })
                        .unwrap();
                }
            }
            fn node_removed(&mut self, node: &DetachedNode, _: &TreeState) {
                if filter_detached(node) == FilterResult::Include {
                    self.remove_node(node);
                }
            }
        }
        let mut handler = Handler {
            adapter: self,
            tree: &self.tree,
        };
        self.tree.update_and_process_changes(update, &mut handler);
    }

    fn window_activated(&self, window: &NodeWrapper, events: &UnboundedSender<Event>) {
        events
            .unbounded_send(Event::Window {
                target: window.id(),
                name: window.name(),
                event: WindowEvent::Activated,
            })
            .unwrap();
        events
            .unbounded_send(Event::Object {
                target: window.id(),
                event: ObjectEvent::StateChanged(State::Active, true),
            })
            .unwrap();
    }

    fn window_deactivated(&self, window: &NodeWrapper, events: &UnboundedSender<Event>) {
        events
            .unbounded_send(Event::Window {
                target: window.id(),
                name: window.name(),
                event: WindowEvent::Deactivated,
            })
            .unwrap();
        events
            .unbounded_send(Event::Object {
                target: window.id(),
                event: ObjectEvent::StateChanged(State::Active, false),
            })
            .unwrap();
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

async fn handle_events(bus: Bus, mut events: UnboundedReceiver<Event>) {
    while let Some(event) = events.next().await {
        let _ = match event {
            Event::Object { target, event } => bus.emit_object_event(target, event).await,
            Event::Window {
                target,
                name,
                event,
            } => bus.emit_window_event(target, name, event).await,
        };
    }
}
