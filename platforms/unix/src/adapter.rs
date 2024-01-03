// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{
        interfaces::{Event, ObjectEvent, WindowEvent},
        ObjectId,
    },
    context::{ActivationContext, AppContext, Context},
    filters::{filter, filter_detached},
    node::NodeWrapper,
    util::{block_on, WindowBounds},
};
use accesskit::{ActionHandler, NodeId, Rect, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use async_channel::Sender;
use async_once_cell::Lazy;
use atspi::{InterfaceSet, Live, State};
use futures_lite::{future::Boxed, FutureExt};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

struct AdapterChangeHandler<'a> {
    adapter: &'a AdapterImpl,
}

impl AdapterChangeHandler<'_> {
    fn add_node(&mut self, node: &Node) {
        block_on(async {
            let role = node.role();
            let is_root = node.is_root();
            let node = NodeWrapper::Node {
                adapter: self.adapter.id,
                node,
            };
            let interfaces = node.interfaces();
            self.adapter
                .register_interfaces(node.id(), interfaces)
                .await;
            if is_root && role == Role::Window {
                let adapter_index = AppContext::read().adapter_index(self.adapter.id).unwrap();
                self.adapter.window_created(adapter_index, node.id()).await;
            }

            let live = node.live();
            if live != Live::None {
                if let Some(name) = node.name() {
                    self.adapter
                        .emit_object_event(
                            ObjectId::Node {
                                adapter: self.adapter.id,
                                node: node.id(),
                            },
                            ObjectEvent::Announcement(name, live),
                        )
                        .await;
                }
            }
        })
    }

    fn remove_node(&mut self, node: &DetachedNode) {
        block_on(async {
            let role = node.role();
            let is_root = node.is_root();
            let node = NodeWrapper::DetachedNode {
                adapter: self.adapter.id,
                node,
            };
            if is_root && role == Role::Window {
                self.adapter.window_destroyed(node.id()).await;
            }
            self.adapter
                .emit_object_event(
                    ObjectId::Node {
                        adapter: self.adapter.id,
                        node: node.id(),
                    },
                    ObjectEvent::StateChanged(State::Defunct, true),
                )
                .await;
            self.adapter
                .unregister_interfaces(node.id(), node.interfaces())
                .await;
        });
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
            block_on(async {
                self.adapter
                    .unregister_interfaces(new_wrapper.id(), old_interfaces ^ kept_interfaces)
                    .await;
                self.adapter
                    .register_interfaces(new_node.id(), new_interfaces ^ kept_interfaces)
                    .await;
                let bounds = *self.adapter.context.read_root_window_bounds();
                new_wrapper
                    .notify_changes(&bounds, self.adapter, &old_wrapper)
                    .await;
            });
        }
    }

    fn focus_moved(
        &mut self,
        old_node: Option<&DetachedNode>,
        new_node: Option<&Node>,
        current_state: &TreeState,
    ) {
        block_on(async {
            if let Some(root_window) = root_window(current_state) {
                if old_node.is_none() && new_node.is_some() {
                    self.adapter
                        .window_activated(&NodeWrapper::Node {
                            adapter: self.adapter.id,
                            node: &root_window,
                        })
                        .await;
                } else if old_node.is_some() && new_node.is_none() {
                    self.adapter
                        .window_deactivated(&NodeWrapper::Node {
                            adapter: self.adapter.id,
                            node: &root_window,
                        })
                        .await;
                }
            }
            if let Some(node) = new_node.map(|node| NodeWrapper::Node {
                adapter: self.adapter.id,
                node,
            }) {
                self.adapter
                    .emit_object_event(
                        ObjectId::Node {
                            adapter: self.adapter.id,
                            node: node.id(),
                        },
                        ObjectEvent::StateChanged(State::Focused, true),
                    )
                    .await;
            }
            if let Some(node) = old_node.map(|node| NodeWrapper::DetachedNode {
                adapter: self.adapter.id,
                node,
            }) {
                self.adapter
                    .emit_object_event(
                        ObjectId::Node {
                            adapter: self.adapter.id,
                            node: node.id(),
                        },
                        ObjectEvent::StateChanged(State::Focused, false),
                    )
                    .await;
            }
        });
    }

    fn node_removed(&mut self, node: &DetachedNode, _: &TreeState) {
        if filter_detached(node) == FilterResult::Include {
            self.remove_node(node);
        }
    }
}

pub(crate) struct AdapterImpl {
    id: usize,
    messages: Sender<Message>,
    context: Arc<Context>,
}

impl AdapterImpl {
    fn new(
        id: usize,
        initial_state: TreeUpdate,
        is_window_focused: bool,
        root_window_bounds: WindowBounds,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let tree = Tree::new(initial_state, is_window_focused);
        let (messages, context) = {
            let mut app_context = AppContext::write();
            let messages = app_context.messages.clone().unwrap();
            let context = Context::new(tree, action_handler, root_window_bounds);
            app_context.push_adapter(id, &context);
            (messages, context)
        };
        AdapterImpl {
            id,
            messages,
            context,
        }
    }

    pub(crate) async fn register_tree(&self) {
        fn add_children(
            node: Node<'_>,
            to_add: &mut Vec<(NodeId, InterfaceSet)>,
            adapter_id: usize,
        ) {
            for child in node.filtered_children(&filter) {
                let child_id = child.id();
                let wrapper = NodeWrapper::Node {
                    adapter: adapter_id,
                    node: &child,
                };
                let interfaces = wrapper.interfaces();
                to_add.push((child_id, interfaces));
                add_children(child, to_add, adapter_id);
            }
        }

        let mut objects_to_add = Vec::new();

        let (adapter_index, root_id) = {
            let tree = self.context.read_tree();
            let tree_state = tree.state();
            let mut app_context = AppContext::write();
            app_context.name = tree_state.app_name();
            app_context.toolkit_name = tree_state.toolkit_name();
            app_context.toolkit_version = tree_state.toolkit_version();
            let adapter_index = app_context.adapter_index(self.id).unwrap();
            let root = tree_state.root();
            let root_id = root.id();
            let wrapper = NodeWrapper::Node {
                adapter: self.id,
                node: &root,
            };
            objects_to_add.push((root_id, wrapper.interfaces()));
            add_children(root, &mut objects_to_add, self.id);
            (adapter_index, root_id)
        };

        for (id, interfaces) in objects_to_add {
            self.register_interfaces(id, interfaces).await;
            if id == root_id {
                self.window_created(adapter_index, id).await;
            }
        }
    }

    async fn register_interfaces(&self, id: NodeId, new_interfaces: InterfaceSet) {
        self.messages
            .send(Message::RegisterInterfaces {
                adapter_id: self.id,
                context: Arc::downgrade(&self.context),
                node_id: id,
                interfaces: new_interfaces,
            })
            .await
            .unwrap();
    }

    async fn unregister_interfaces(&self, id: NodeId, old_interfaces: InterfaceSet) {
        self.messages
            .send(Message::UnregisterInterfaces {
                adapter_id: self.id,
                node_id: id,
                interfaces: old_interfaces,
            })
            .await
            .unwrap();
    }

    pub(crate) async fn emit_object_event(&self, target: ObjectId, event: ObjectEvent) {
        self.messages
            .send(Message::EmitEvent(Event::Object { target, event }))
            .await
            .unwrap();
    }

    fn set_root_window_bounds(&self, bounds: WindowBounds) {
        let mut old_bounds = self.context.root_window_bounds.write().unwrap();
        *old_bounds = bounds;
    }

    fn update(&self, update: TreeUpdate) {
        let mut handler = AdapterChangeHandler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_and_process_changes(update, &mut handler);
    }

    fn update_window_focus_state(&self, is_focused: bool) {
        let mut handler = AdapterChangeHandler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
    }

    async fn window_created(&self, adapter_index: usize, window: NodeId) {
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ChildAdded(
                adapter_index,
                ObjectId::Node {
                    adapter: self.id,
                    node: window,
                },
            ),
        )
        .await;
    }

    async fn window_activated(&self, window: &NodeWrapper<'_>) {
        self.messages
            .send(Message::EmitEvent(Event::Window {
                target: ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                },
                name: window.name().unwrap_or_default(),
                event: WindowEvent::Activated,
            }))
            .await
            .unwrap();
        self.emit_object_event(
            ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            },
            ObjectEvent::StateChanged(State::Active, true),
        )
        .await;
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ActiveDescendantChanged(ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            }),
        )
        .await;
    }

    async fn window_deactivated(&self, window: &NodeWrapper<'_>) {
        self.messages
            .send(Message::EmitEvent(Event::Window {
                target: ObjectId::Node {
                    adapter: self.id,
                    node: window.id(),
                },
                name: window.name().unwrap_or_default(),
                event: WindowEvent::Deactivated,
            }))
            .await
            .unwrap();
        self.emit_object_event(
            ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            },
            ObjectEvent::StateChanged(State::Active, false),
        )
        .await;
    }

    async fn window_destroyed(&self, window: NodeId) {
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ChildRemoved(ObjectId::Node {
                adapter: self.id,
                node: window,
            }),
        )
        .await;
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

impl Drop for AdapterImpl {
    fn drop(&mut self) {
        AppContext::write().remove_adapter(self.id);
        let root_id = self.context.read_tree().state().root_id();
        block_on(async {
            self.emit_object_event(
                ObjectId::Root,
                ObjectEvent::ChildRemoved(ObjectId::Node {
                    adapter: self.id,
                    node: root_id,
                }),
            )
            .await;
        });
    }
}

pub(crate) type LazyAdapter = Pin<Arc<Lazy<AdapterImpl, Boxed<AdapterImpl>>>>;

static NEXT_ADAPTER_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Adapter {
    id: usize,
    r#impl: LazyAdapter,
    is_window_focused: Arc<AtomicBool>,
    root_window_bounds: Arc<Mutex<WindowBounds>>,
}

impl Adapter {
    /// Create a new Unix adapter.
    pub fn new(
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let id = NEXT_ADAPTER_ID.fetch_add(1, Ordering::SeqCst);
        let is_window_focused = Arc::new(AtomicBool::new(false));
        let is_window_focused_copy = is_window_focused.clone();
        let root_window_bounds = Arc::new(Mutex::new(Default::default()));
        let root_window_bounds_copy = root_window_bounds.clone();
        let r#impl = Arc::pin(Lazy::new(
            async move {
                let is_window_focused = is_window_focused_copy.load(Ordering::Relaxed);
                let root_window_bounds = *root_window_bounds_copy.lock().unwrap();
                AdapterImpl::new(
                    id,
                    source(),
                    is_window_focused,
                    root_window_bounds,
                    action_handler,
                )
            }
            .boxed(),
        ));
        let adapter = Self {
            id,
            r#impl: r#impl.clone(),
            is_window_focused,
            root_window_bounds,
        };
        block_on(async move { ActivationContext::activate_eventually(id, r#impl).await });
        adapter
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        let new_bounds = WindowBounds::new(outer, inner);
        {
            let mut bounds = self.root_window_bounds.lock().unwrap();
            *bounds = new_bounds;
        }
        if let Some(r#impl) = Lazy::try_get(&self.r#impl) {
            r#impl.set_root_window_bounds(new_bounds);
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    pub fn update_if_active(&self, update_factory: impl FnOnce() -> TreeUpdate) {
        if let Some(r#impl) = Lazy::try_get(&self.r#impl) {
            r#impl.update(update_factory());
        }
    }

    /// Update the tree state based on whether the window is focused.
    pub fn update_window_focus_state(&self, is_focused: bool) {
        self.is_window_focused.store(is_focused, Ordering::SeqCst);
        if let Some(r#impl) = Lazy::try_get(&self.r#impl) {
            r#impl.update_window_focus_state(is_focused);
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        block_on(async {
            ActivationContext::remove_adapter(self.id).await;
        })
    }
}

pub(crate) enum Message {
    RegisterInterfaces {
        adapter_id: usize,
        context: Weak<Context>,
        node_id: NodeId,
        interfaces: InterfaceSet,
    },
    UnregisterInterfaces {
        adapter_id: usize,
        node_id: NodeId,
        interfaces: InterfaceSet,
    },
    EmitEvent(Event),
}
