// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{
        interfaces::{Event, ObjectEvent, WindowEvent},
        ObjectId,
    },
    context::{AppContext, Context},
    filters::{filter, filter_detached},
    node::NodeWrapper,
    util::WindowBounds,
};
use accesskit::{ActionHandler, NodeId, Rect, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
#[cfg(not(feature = "tokio"))]
use async_channel::Sender;
use atspi::{InterfaceSet, Live, State};
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex, Weak,
};
#[cfg(feature = "tokio")]
use tokio::sync::mpsc::UnboundedSender as Sender;

struct AdapterChangeHandler<'a> {
    adapter: &'a AdapterImpl,
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
        self.adapter.register_interfaces(node.id(), interfaces);
        if is_root && role == Role::Window {
            let adapter_index = AppContext::read().adapter_index(self.adapter.id).unwrap();
            self.adapter.window_created(adapter_index, node.id());
        }

        let live = node.live();
        if live != Live::None {
            if let Some(name) = node.name() {
                self.adapter.emit_object_event(
                    ObjectId::Node {
                        adapter: self.adapter.id,
                        node: node.id(),
                    },
                    ObjectEvent::Announcement(name, live),
                );
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
        self.adapter.emit_object_event(
            ObjectId::Node {
                adapter: self.adapter.id,
                node: node.id(),
            },
            ObjectEvent::StateChanged(State::Defunct, true),
        );
        self.adapter
            .unregister_interfaces(node.id(), node.interfaces());
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
                .unregister_interfaces(new_wrapper.id(), old_interfaces ^ kept_interfaces);
            self.adapter
                .register_interfaces(new_node.id(), new_interfaces ^ kept_interfaces);
            let bounds = *self.adapter.context.read_root_window_bounds();
            new_wrapper.notify_changes(&bounds, self.adapter, &old_wrapper);
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
            self.adapter.emit_object_event(
                ObjectId::Node {
                    adapter: self.adapter.id,
                    node: node.id(),
                },
                ObjectEvent::StateChanged(State::Focused, true),
            );
        }
        if let Some(node) = old_node.map(|node| NodeWrapper::DetachedNode {
            adapter: self.adapter.id,
            node,
        }) {
            self.adapter.emit_object_event(
                ObjectId::Node {
                    adapter: self.adapter.id,
                    node: node.id(),
                },
                ObjectEvent::StateChanged(State::Focused, false),
            );
        }
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
        messages: Sender<Message>,
        initial_state: TreeUpdate,
        is_window_focused: bool,
        root_window_bounds: WindowBounds,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let tree = Tree::new(initial_state, is_window_focused);
        let context = {
            let mut app_context = AppContext::write();
            let context = Context::new(tree, action_handler, root_window_bounds);
            app_context.push_adapter(id, &context);
            context
        };
        AdapterImpl {
            id,
            messages,
            context,
        }
    }

    pub(crate) fn register_tree(&self) {
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
            self.register_interfaces(id, interfaces);
            if id == root_id {
                self.window_created(adapter_index, id);
            }
        }
    }

    pub(crate) fn send_message(&self, message: Message) {
        #[cfg(not(feature = "tokio"))]
        let _ = self.messages.try_send(message);
        #[cfg(feature = "tokio")]
        let _ = self.messages.send(message);
    }

    fn register_interfaces(&self, id: NodeId, new_interfaces: InterfaceSet) {
        self.send_message(Message::RegisterInterfaces {
            adapter_id: self.id,
            context: Arc::downgrade(&self.context),
            node_id: id,
            interfaces: new_interfaces,
        });
    }

    fn unregister_interfaces(&self, id: NodeId, old_interfaces: InterfaceSet) {
        self.send_message(Message::UnregisterInterfaces {
            adapter_id: self.id,
            node_id: id,
            interfaces: old_interfaces,
        });
    }

    pub(crate) fn emit_object_event(&self, target: ObjectId, event: ObjectEvent) {
        self.send_message(Message::EmitEvent(Event::Object { target, event }));
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

    fn window_created(&self, adapter_index: usize, window: NodeId) {
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ChildAdded(
                adapter_index,
                ObjectId::Node {
                    adapter: self.id,
                    node: window,
                },
            ),
        );
    }

    fn window_activated(&self, window: &NodeWrapper<'_>) {
        self.send_message(Message::EmitEvent(Event::Window {
            target: ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            },
            name: window.name().unwrap_or_default(),
            event: WindowEvent::Activated,
        }));
        self.emit_object_event(
            ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            },
            ObjectEvent::StateChanged(State::Active, true),
        );
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ActiveDescendantChanged(ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            }),
        );
    }

    fn window_deactivated(&self, window: &NodeWrapper<'_>) {
        self.send_message(Message::EmitEvent(Event::Window {
            target: ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            },
            name: window.name().unwrap_or_default(),
            event: WindowEvent::Deactivated,
        }));
        self.emit_object_event(
            ObjectId::Node {
                adapter: self.id,
                node: window.id(),
            },
            ObjectEvent::StateChanged(State::Active, false),
        );
    }

    fn window_destroyed(&self, window: NodeId) {
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ChildRemoved(ObjectId::Node {
                adapter: self.id,
                node: window,
            }),
        );
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
        self.emit_object_event(
            ObjectId::Root,
            ObjectEvent::ChildRemoved(ObjectId::Node {
                adapter: self.id,
                node: root_id,
            }),
        );
    }
}

pub(crate) type LazyAdapter = Arc<Lazy<AdapterImpl, Box<dyn FnOnce() -> AdapterImpl + Send>>>;

static NEXT_ADAPTER_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Adapter {
    messages: Sender<Message>,
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
        let messages = AppContext::read().messages.clone();
        let is_window_focused = Arc::new(AtomicBool::new(false));
        let root_window_bounds = Arc::new(Mutex::new(Default::default()));
        let r#impl: LazyAdapter = Arc::new(Lazy::new(Box::new({
            let messages = messages.clone();
            let is_window_focused = Arc::clone(&is_window_focused);
            let root_window_bounds = Arc::clone(&root_window_bounds);
            move || {
                AdapterImpl::new(
                    id,
                    messages,
                    source(),
                    is_window_focused.load(Ordering::Relaxed),
                    *root_window_bounds.lock().unwrap(),
                    action_handler,
                )
            }
        })));
        let adapter = Self {
            id,
            messages: messages.clone(),
            r#impl: r#impl.clone(),
            is_window_focused,
            root_window_bounds,
        };
        adapter.send_message(Message::AddAdapter {
            id,
            adapter: r#impl,
        });
        adapter
    }

    pub(crate) fn send_message(&self, message: Message) {
        #[cfg(not(feature = "tokio"))]
        let _ = self.messages.try_send(message);
        #[cfg(feature = "tokio")]
        let _ = self.messages.send(message);
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        let new_bounds = WindowBounds::new(outer, inner);
        {
            let mut bounds = self.root_window_bounds.lock().unwrap();
            *bounds = new_bounds;
        }
        if let Some(r#impl) = Lazy::get(&self.r#impl) {
            r#impl.set_root_window_bounds(new_bounds);
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    pub fn update_if_active(&self, update_factory: impl FnOnce() -> TreeUpdate) {
        if let Some(r#impl) = Lazy::get(&self.r#impl) {
            r#impl.update(update_factory());
        }
    }

    /// Update the tree state based on whether the window is focused.
    pub fn update_window_focus_state(&self, is_focused: bool) {
        self.is_window_focused.store(is_focused, Ordering::SeqCst);
        if let Some(r#impl) = Lazy::get(&self.r#impl) {
            r#impl.update_window_focus_state(is_focused);
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        self.send_message(Message::RemoveAdapter { id: self.id });
    }
}

pub(crate) enum Message {
    AddAdapter {
        id: usize,
        adapter: LazyAdapter,
    },
    RemoveAdapter {
        id: usize,
    },
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
