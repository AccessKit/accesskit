// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, NodeId, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use atspi_common::{InterfaceSet, Live, State};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

use crate::{
    context::{AppContext, Context},
    filters::{filter, filter_detached},
    node::{NodeIdOrRoot, NodeWrapper, PlatformNode, PlatformRoot},
    util::WindowBounds,
    AdapterCallback, Event, ObjectEvent, WindowEvent,
};

struct AdapterChangeHandler<'a> {
    adapter: &'a Adapter,
}

impl AdapterChangeHandler<'_> {
    fn add_node(&mut self, node: &Node) {
        let role = node.role();
        let is_root = node.is_root();
        let node = NodeWrapper::Node(node);
        let interfaces = node.interfaces();
        self.adapter.register_interfaces(node.id(), interfaces);
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
                    .emit_object_event(node.id(), ObjectEvent::Announcement(name, live));
            }
        }
    }

    fn remove_node(&mut self, node: &DetachedNode) {
        let role = node.role();
        let is_root = node.is_root();
        let node = NodeWrapper::DetachedNode(node);
        if is_root && role == Role::Window {
            self.adapter.window_destroyed(node.id());
        }
        self.adapter
            .emit_object_event(node.id(), ObjectEvent::StateChanged(State::Defunct, true));
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
            let old_wrapper = NodeWrapper::DetachedNode(old_node);
            let new_wrapper = NodeWrapper::Node(new_node);
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
                self.adapter
                    .window_activated(&NodeWrapper::Node(&root_window));
            } else if old_node.is_some() && new_node.is_none() {
                self.adapter
                    .window_deactivated(&NodeWrapper::Node(&root_window));
            }
        }
    }

    fn node_removed(&mut self, node: &DetachedNode, _: &TreeState) {
        if filter_detached(node) == FilterResult::Include {
            self.remove_node(node);
        }
    }
}

static NEXT_ADAPTER_ID: AtomicUsize = AtomicUsize::new(0);

pub struct AdapterIdToken(usize);

impl AdapterIdToken {
    pub fn next() -> Self {
        let id = NEXT_ADAPTER_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

pub struct Adapter {
    id: usize,
    callback: Box<dyn AdapterCallback + Send + Sync>,
    context: Arc<Context>,
}

impl Adapter {
    pub fn new(
        app_context: &Arc<RwLock<AppContext>>,
        callback: Box<dyn AdapterCallback + Send + Sync>,
        initial_state: TreeUpdate,
        is_window_focused: bool,
        root_window_bounds: WindowBounds,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let id_token = AdapterIdToken::next();
        Self::with_id(
            id_token,
            app_context,
            callback,
            initial_state,
            is_window_focused,
            root_window_bounds,
            action_handler,
        )
    }

    pub fn with_id(
        id_token: AdapterIdToken,
        app_context: &Arc<RwLock<AppContext>>,
        callback: Box<dyn AdapterCallback + Send + Sync>,
        initial_state: TreeUpdate,
        is_window_focused: bool,
        root_window_bounds: WindowBounds,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let id = id_token.0;
        let tree = Tree::new(initial_state, is_window_focused);
        let context = Context::new(app_context, tree, action_handler, root_window_bounds);
        context.write_app_context().push_adapter(id, &context);
        let adapter = Self {
            id,
            callback,
            context,
        };
        adapter.register_tree();
        adapter
    }

    fn register_tree(&self) {
        fn add_children(node: Node<'_>, to_add: &mut Vec<(NodeId, InterfaceSet)>) {
            for child in node.filtered_children(&filter) {
                let child_id = child.id();
                let wrapper = NodeWrapper::Node(&child);
                let interfaces = wrapper.interfaces();
                to_add.push((child_id, interfaces));
                add_children(child, to_add);
            }
        }

        let mut objects_to_add = Vec::new();

        let (adapter_index, root_id) = {
            let tree = self.context.read_tree();
            let tree_state = tree.state();
            let mut app_context = self.context.write_app_context();
            app_context.name = tree_state.app_name();
            app_context.toolkit_name = tree_state.toolkit_name();
            app_context.toolkit_version = tree_state.toolkit_version();
            let adapter_index = app_context.adapter_index(self.id).unwrap();
            let root = tree_state.root();
            let root_id = root.id();
            let wrapper = NodeWrapper::Node(&root);
            objects_to_add.push((root_id, wrapper.interfaces()));
            add_children(root, &mut objects_to_add);
            (adapter_index, root_id)
        };

        for (id, interfaces) in objects_to_add {
            self.register_interfaces(id, interfaces);
            if id == root_id {
                self.window_created(adapter_index, id);
            }
        }
    }

    pub fn platform_node(&self, id: NodeId) -> PlatformNode {
        PlatformNode::new(&self.context, self.id, id)
    }

    pub fn root_id(&self) -> NodeId {
        self.context.read_tree().state().root_id()
    }

    pub fn platform_root(&self) -> PlatformRoot {
        PlatformRoot::new(&self.context.app_context)
    }

    fn register_interfaces(&self, id: NodeId, new_interfaces: InterfaceSet) {
        self.callback.register_interfaces(self, id, new_interfaces);
    }

    fn unregister_interfaces(&self, id: NodeId, old_interfaces: InterfaceSet) {
        self.callback
            .unregister_interfaces(self, id, old_interfaces);
    }

    pub(crate) fn emit_object_event(&self, target: NodeId, event: ObjectEvent) {
        let target = NodeIdOrRoot::Node(target);
        self.callback
            .emit_event(self, Event::Object { target, event });
    }

    fn emit_root_object_event(&self, event: ObjectEvent) {
        let target = NodeIdOrRoot::Root;
        self.callback
            .emit_event(self, Event::Object { target, event });
    }

    pub fn set_root_window_bounds(&self, new_bounds: WindowBounds) {
        let mut bounds = self.context.root_window_bounds.write().unwrap();
        *bounds = new_bounds;
    }

    pub fn update(&self, update: TreeUpdate) {
        let mut handler = AdapterChangeHandler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_and_process_changes(update, &mut handler);
    }

    pub fn update_window_focus_state(&self, is_focused: bool) {
        let mut handler = AdapterChangeHandler { adapter: self };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
    }

    fn window_created(&self, adapter_index: usize, window: NodeId) {
        self.emit_root_object_event(ObjectEvent::ChildAdded(adapter_index, window));
    }

    fn window_activated(&self, window: &NodeWrapper<'_>) {
        self.callback.emit_event(
            self,
            Event::Window {
                target: window.id(),
                name: window.name().unwrap_or_default(),
                event: WindowEvent::Activated,
            },
        );
        self.emit_object_event(window.id(), ObjectEvent::StateChanged(State::Active, true));
        self.emit_root_object_event(ObjectEvent::ActiveDescendantChanged(window.id()));
    }

    fn window_deactivated(&self, window: &NodeWrapper<'_>) {
        self.callback.emit_event(
            self,
            Event::Window {
                target: window.id(),
                name: window.name().unwrap_or_default(),
                event: WindowEvent::Deactivated,
            },
        );
        self.emit_object_event(window.id(), ObjectEvent::StateChanged(State::Active, false));
    }

    fn window_destroyed(&self, window: NodeId) {
        self.emit_root_object_event(ObjectEvent::ChildRemoved(window));
    }

    pub fn id(&self) -> usize {
        self.id
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
        let root_id = self.context.read_tree().state().root_id();
        self.window_destroyed(root_id);
        // Note: We deliberately do the following here, not in a Drop
        // implementation on context, because AppContext owns a second
        // strong reference to Context, and we need that to be released.
        self.context.write_app_context().remove_adapter(self.id);
    }
}
