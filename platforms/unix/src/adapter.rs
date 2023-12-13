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
        Bus, ObjectId,
    },
    context::{AppContext, Context},
    filters::{filter, filter_detached},
    node::{NodeWrapper, PlatformNode},
    util::block_on,
};
use accesskit::{ActionHandler, NodeId, Rect, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use async_channel::{Receiver, Sender};
use atspi::{Interface, InterfaceSet, Live, State};
use futures_lite::StreamExt;
use std::{
    pin::pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use zbus::Task;

struct AdapterChangeHandler<'a> {
    adapter: &'a Adapter,
}

impl AdapterChangeHandler<'_> {
    fn add_node(&mut self, node: &Node) {
        let role = node.role();
        let is_root = node.is_root();
        let node = NodeWrapper::Node {
            adapter: self.adapter.id,
            node,
        };
        let interfaces = node.interfaces();
        self.adapter
            .register_interfaces(node.id(), interfaces)
            .unwrap();
        if is_root && role == Role::Window {
            let adapter_index = self
                .adapter
                .context
                .read_app_context()
                .adapter_index(self.adapter.id)
                .unwrap();
            self.adapter.window_created(adapter_index, node.id());
        }

        let live = node.live();
        if live != Live::None {
            if let Some(name) = node.name() {
                self.adapter
                    .events
                    .send_blocking(Event::Object {
                        target: ObjectId::Node {
                            adapter: self.adapter.id,
                            node: node.id(),
                        },
                        event: ObjectEvent::Announcement(name, live),
                    })
                    .unwrap();
            }
        }
    }

    fn remove_node(&mut self, node: &DetachedNode) {
        let role = node.role();
        let is_root = node.is_root();
        let node = NodeWrapper::DetachedNode {
            adapter: self.adapter.id,
            node,
        };
        if is_root && role == Role::Window {
            self.adapter.window_destroyed(node.id());
        }
        self.adapter
            .events
            .send_blocking(Event::Object {
                target: ObjectId::Node {
                    adapter: self.adapter.id,
                    node: node.id(),
                },
                event: ObjectEvent::StateChanged(State::Defunct, true),
            })
            .unwrap();
        self.adapter
            .unregister_interfaces(node.id(), node.interfaces())
            .unwrap();
    }
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
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
            let old_wrapper = NodeWrapper::DetachedNode {
                adapter: self.adapter.id,
                node: old_node,
            };
            let new_wrapper = NodeWrapper::Node {
                adapter: self.adapter.id,
                node: new_node,
            };
            let old_interfaces = old_wrapper.interfaces();
            let new_interfaces = new_wrapper.interfaces();
            let kept_interfaces = old_interfaces & new_interfaces;
            self.adapter
                .unregister_interfaces(new_wrapper.id(), old_interfaces ^ kept_interfaces)
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
                self.adapter.window_activated(&NodeWrapper::Node {
                    adapter: self.adapter.id,
                    node: &root_window,
                });
            } else if old_node.is_some() && new_node.is_none() {
                self.adapter.window_deactivated(&NodeWrapper::Node {
                    adapter: self.adapter.id,
                    node: &root_window,
                });
            }
        }
        if let Some(node) = new_node.map(|node| NodeWrapper::Node {
            adapter: self.adapter.id,
            node,
        }) {
            self.adapter
                .events
                .send_blocking(Event::Object {
                    target: ObjectId::Node {
                        adapter: self.adapter.id,
                        node: node.id(),
                    },
                    event: ObjectEvent::StateChanged(State::Focused, true),
                })
                .unwrap();
        }
        if let Some(node) = old_node.map(|node| NodeWrapper::DetachedNode {
            adapter: self.adapter.id,
            node,
        }) {
            self.adapter
                .events
                .send_blocking(Event::Object {
                    target: ObjectId::Node {
                        adapter: self.adapter.id,
                        node: node.id(),
                    },
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

static NEXT_ADAPTER_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Adapter {
    id: usize,
    atspi_bus: Bus,
    _event_task: Task<()>,
    events: Sender<Event>,
    context: Arc<Context>,
}

impl Adapter {
    /// Create a new Unix adapter.
    pub fn new(
        initial_state: impl 'static + FnOnce() -> TreeUpdate,
        is_window_focused: bool,
        action_handler: Box<dyn ActionHandler + Send>,
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
        let tree = Tree::new(initial_state(), is_window_focused);
        let id = NEXT_ADAPTER_ID.fetch_add(1, Ordering::SeqCst);
        let root_id = tree.state().root_id();
        let app_context = AppContext::get_or_init(
            tree.state().app_name().unwrap_or_default(),
            tree.state().toolkit_name().unwrap_or_default(),
            tree.state().toolkit_version().unwrap_or_default(),
        );
        let context = Context::new(tree, action_handler, &app_context);
        let adapter_index = app_context.write().unwrap().push_adapter(id, &context);
        block_on(async {
            if !atspi_bus.register_root_node(&app_context).await.ok()? {
                atspi_bus
                    .emit_object_event(
                        ObjectId::Root,
                        ObjectEvent::ChildAdded(
                            adapter_index,
                            ObjectId::Node {
                                adapter: id,
                                node: root_id,
                            },
                        ),
                    )
                    .await
                    .ok()
            } else {
                Some(())
            }
        })?;
        let adapter = Adapter {
            id,
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
            let node = tree_state.node_by_id(id).unwrap();
            let wrapper = NodeWrapper::Node {
                adapter: self.id,
                node: &node,
            };
            let interfaces = wrapper.interfaces();
            self.register_interfaces(id, interfaces).unwrap();
            if node.is_root() && node.role() == Role::Window {
                let adapter_index = self
                    .context
                    .read_app_context()
                    .adapter_index(self.id)
                    .unwrap();
                self.window_created(adapter_index, node.id());
            }
        }
    }

    fn register_interfaces(&self, id: NodeId, new_interfaces: InterfaceSet) -> zbus::Result<bool> {
        let path = ObjectId::Node {
            adapter: self.id,
            node: id,
        }
        .path();
        if new_interfaces.contains(Interface::Accessible) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        AccessibleInterface::new(
                            self.atspi_bus.unique_name().to_owned(),
                            PlatformNode::new(&self.context, self.id, id),
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
                        ActionInterface::new(PlatformNode::new(&self.context, self.id, id)),
                    )
                    .await
            })?;
        }
        if new_interfaces.contains(Interface::Component) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        ComponentInterface::new(PlatformNode::new(&self.context, self.id, id)),
                    )
                    .await
            })?;
        }
        if new_interfaces.contains(Interface::Value) {
            block_on(async {
                self.atspi_bus
                    .register_interface(
                        &path,
                        ValueInterface::new(PlatformNode::new(&self.context, self.id, id)),
                    )
                    .await
            })?;
        }
        Ok(true)
    }

    fn unregister_interfaces(
        &self,
        id: NodeId,
        old_interfaces: InterfaceSet,
    ) -> zbus::Result<bool> {
        block_on(async {
            let path = ObjectId::Node {
                adapter: self.id,
                node: id,
            }
            .path();
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
        let mut handler = AdapterChangeHandler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_and_process_changes(update, &mut handler);
    }

    /// Update the tree state based on whether the window is focused.
    pub fn update_window_focus_state(&self, is_focused: bool) {
        let mut handler = AdapterChangeHandler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
    }

    fn window_created(&self, adapter_index: usize, window: NodeId) {
        self.events
            .send_blocking(Event::Object {
                target: ObjectId::Root,
                event: ObjectEvent::ChildAdded(
                    adapter_index,
                    ObjectId::Node {
                        adapter: self.id,
                        node: window,
                    },
                ),
            })
            .unwrap();
    }

    fn window_activated(&self, window: &NodeWrapper) {
        self.events
            .send_blocking(Event::Window {
                target: ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                },
                name: window.name().unwrap_or_default(),
                event: WindowEvent::Activated,
            })
            .unwrap();
        self.events
            .send_blocking(Event::Object {
                target: ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                },
                event: ObjectEvent::StateChanged(State::Active, true),
            })
            .unwrap();
        self.events
            .send_blocking(Event::Object {
                target: ObjectId::Root,
                event: ObjectEvent::ActiveDescendantChanged(ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                }),
            })
            .unwrap();
    }

    fn window_deactivated(&self, window: &NodeWrapper) {
        self.events
            .send_blocking(Event::Window {
                target: ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                },
                name: window.name().unwrap_or_default(),
                event: WindowEvent::Deactivated,
            })
            .unwrap();
        self.events
            .send_blocking(Event::Object {
                target: ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                },
                event: ObjectEvent::StateChanged(State::Active, false),
            })
            .unwrap();
    }

    fn window_destroyed(&self, window: NodeId) {
        self.events
            .send_blocking(Event::Object {
                target: ObjectId::Root,
                event: ObjectEvent::ChildRemoved(ObjectId::Node {
                    adapter: self.id,
                    node: window,
                }),
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

impl Drop for Adapter {
    fn drop(&mut self) {
        block_on(async {
            self.context
                .app_context
                .write()
                .unwrap()
                .remove_adapter(self.id);
            let root_id = self.context.read_tree().state().root_id();
            self.atspi_bus
                .emit_object_event(
                    ObjectId::Root,
                    ObjectEvent::ChildRemoved(ObjectId::Node {
                        adapter: self.id,
                        node: root_id,
                    }),
                )
                .await
                .unwrap();
        });
    }
}

async fn handle_events(bus: Bus, mut events: Receiver<Event>) {
    let mut events = pin!(events);
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
