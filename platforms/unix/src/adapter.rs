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
    context::Context,
    node::{filter, filter_detached, NodeWrapper, PlatformNode},
    util::{block_on, AppContext},
};
use accesskit::{ActionHandler, NodeId, Rect, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use async_channel::{Receiver, Sender};
use atspi::{Interface, InterfaceSet, State};
use futures_lite::StreamExt;
use std::sync::Arc;
use zbus::Task;

pub struct Adapter {
    atspi_bus: Bus,
    _event_task: Task<()>,
    events: Sender<Event>,
    context: Arc<Context>,
}

impl Adapter {
    /// Create a new Unix adapter.
    pub fn new(
        app_name: String,
        toolkit_name: String,
        toolkit_version: String,
        initial_state: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
    ) -> Option<Self> {
        let mut atspi_bus = block_on(async { Bus::a11y_bus().await })?;
        let (event_sender, event_receiver) = async_channel::unbounded();
        let atspi_bus_copy = atspi_bus.clone();
        #[cfg(feature = "tokio")]
        let _guard = crate::util::TOKIO_RT.enter();
        let event_task = atspi_bus.connection().executor().spawn(
            async move {
                handle_events(atspi_bus_copy, event_receiver).await;
            },
            "accesskit_event_task",
        );
        let tree = Tree::new(initial_state());
        let app_context = AppContext::new(app_name, toolkit_name, toolkit_version);
        let context = Context::new(tree, action_handler, app_context);
        block_on(async { atspi_bus.register_root_node(&context).await.ok() })?;
        let adapter = Adapter {
            atspi_bus,
            _event_task: event_task,
            events: event_sender,
            context,
        };
        adapter.register_tree();
        Some(adapter)
    }

    fn register_tree(&self) {
        let tree = self.context.read_tree();
        let tree_state = tree.state();
        let mut objects_to_add = Vec::new();

        fn add_children(node: Node<'_>, to_add: &mut Vec<NodeId>) {
            for child in node.filtered_children(&filter) {
                to_add.push(child.id());
                add_children(child, to_add);
            }
        }

        objects_to_add.push(tree_state.root().id());
        add_children(tree_state.root(), &mut objects_to_add);
        for id in objects_to_add {
            let interfaces = NodeWrapper::Node(&tree_state.node_by_id(id).unwrap()).interfaces();
            self.register_interfaces(id, interfaces).unwrap();
        }
    }

    fn register_interfaces(&self, id: NodeId, new_interfaces: InterfaceSet) -> zbus::Result<bool> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, ObjectId::from(id).as_str());
        if new_interfaces.contains(Interface::Accessible) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        AccessibleInterface::new(
                            self.atspi_bus.unique_name().to_owned(),
                            PlatformNode::new(&self.context, id),
                        ),
                    )
                    .await
            })?;
        }
        if new_interfaces.contains(Interface::Action) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        ActionInterface::new(PlatformNode::new(&self.context, id)),
                    )
                    .await
            })?;
        }
        if new_interfaces.contains(Interface::Component) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        ComponentInterface::new(PlatformNode::new(&self.context, id)),
                    )
                    .await
            })?;
        }
        if new_interfaces.contains(Interface::Value) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        ValueInterface::new(PlatformNode::new(&self.context, id)),
                    )
                    .await
            })?;
        }
        Ok(true)
    }

    fn unregister_interfaces(
        &self,
        id: &ObjectId,
        old_interfaces: InterfaceSet,
    ) -> zbus::Result<bool> {
        block_on(async {
            let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, id.as_str());
            if old_interfaces.contains(Interface::Accessible) {
                self.atspi_bus
                    .unregister_interface::<AccessibleInterface<PlatformNode>>(&path)
                    .await?;
            }
            if old_interfaces.contains(Interface::Action) {
                self.atspi_bus
                    .unregister_interface::<ActionInterface>(&path)
                    .await?;
            }
            if old_interfaces.contains(Interface::Component) {
                self.atspi_bus
                    .unregister_interface::<ComponentInterface>(&path)
                    .await?;
            }
            if old_interfaces.contains(Interface::Value) {
                self.atspi_bus
                    .unregister_interface::<ValueInterface>(&path)
                    .await?;
            }
            Ok(true)
        })
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        let mut bounds = self.context.root_window_bounds.write().unwrap();
        bounds.outer = outer;
        bounds.inner = inner;
    }

    /// Apply the provided update to the tree.
    pub fn update(&self, update: TreeUpdate) {
        struct Handler<'a> {
            adapter: &'a Adapter,
        }
        impl Handler<'_> {
            fn add_node(&mut self, node: &Node) {
                let interfaces = NodeWrapper::Node(node).interfaces();
                self.adapter
                    .register_interfaces(node.id(), interfaces)
                    .unwrap();
            }
            fn remove_node(&mut self, node: &DetachedNode) {
                let node = NodeWrapper::DetachedNode(node);
                self.adapter
                    .events
                    .send_blocking(Event::Object {
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
                        .register_interfaces(new_node.id(), new_interfaces ^ kept_interfaces)
                        .unwrap();
                    new_wrapper.notify_changes(
                        &self.adapter.context.read_root_window_bounds(),
                        &self.adapter.events,
                        &old_wrapper,
                    );
                }
            }
            fn focus_moved(
                &mut self,
                old_node: Option<&DetachedNode>,
                new_node: Option<&Node>,
                current_state: &TreeState,
            ) {
                if let Some(root_window) = root_window(current_state) {
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
                        .send_blocking(Event::Object {
                            target: node.id(),
                            event: ObjectEvent::StateChanged(State::Focused, true),
                        })
                        .unwrap();
                }
                if let Some(node) = old_node.map(NodeWrapper::DetachedNode) {
                    self.adapter
                        .events
                        .send_blocking(Event::Object {
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
        let mut handler = Handler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_and_process_changes(update, &mut handler);
    }

    fn window_activated(&self, window: &NodeWrapper, events: &Sender<Event>) {
        events
            .send_blocking(Event::Window {
                target: window.id(),
                name: window.name(),
                event: WindowEvent::Activated,
            })
            .unwrap();
        events
            .send_blocking(Event::Object {
                target: window.id(),
                event: ObjectEvent::StateChanged(State::Active, true),
            })
            .unwrap();
    }

    fn window_deactivated(&self, window: &NodeWrapper, events: &Sender<Event>) {
        events
            .send_blocking(Event::Window {
                target: window.id(),
                name: window.name(),
                event: WindowEvent::Deactivated,
            })
            .unwrap();
        events
            .send_blocking(Event::Object {
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

async fn handle_events(bus: Bus, mut events: Receiver<Event>) {
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
