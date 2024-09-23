// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, NodeBuilder, NodeId, Point, Role,
    Tree as TreeData, TreeUpdate,
};
use accesskit_consumer::{Node, Tree, TreeChangeHandler};
use jni::{
    errors::Result,
    objects::{JClass, JObject},
    sys::{jfloat, jint},
    JNIEnv,
};

use crate::{filters::filter, node::NodeWrapper, util::*};

fn send_window_content_changed(env: &mut JNIEnv, callback_class: &JClass, host: &JObject) {
    send_event(
        env,
        callback_class,
        host,
        HOST_VIEW_ID,
        EVENT_WINDOW_CONTENT_CHANGED,
    );
}

fn send_focus_event_if_applicable(
    env: &mut JNIEnv,
    callback_class: &JClass,
    host: &JObject,
    node_id_map: &mut NodeIdMap,
    node: &Node,
) {
    if node.is_root() && node.role() == Role::Window {
        return;
    }
    let id = node_id_map.get_or_create_java_id(node);
    send_event(env, callback_class, host, id, EVENT_VIEW_FOCUSED);
}

struct AdapterChangeHandler<'a> {
    env: &'a mut JNIEnv<'a>,
    callback_class: &'a JClass<'a>,
    host: &'a JObject<'a>,
    node_id_map: &'a mut NodeIdMap,
    sent_window_content_changed: bool,
}

impl<'a> AdapterChangeHandler<'a> {
    fn new(
        env: &'a mut JNIEnv<'a>,
        callback_class: &'a JClass<'a>,
        host: &'a JObject<'a>,
        node_id_map: &'a mut NodeIdMap,
    ) -> Self {
        Self {
            env,
            callback_class,
            host,
            node_id_map,
            sent_window_content_changed: false,
        }
    }
}

impl AdapterChangeHandler<'_> {
    fn send_window_content_changed_if_needed(&mut self) {
        if self.sent_window_content_changed {
            return;
        }
        send_window_content_changed(self.env, self.callback_class, self.host);
        self.sent_window_content_changed = true;
    }
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
    fn node_added(&mut self, _node: &Node) {
        self.send_window_content_changed_if_needed();
        // TODO: live regions?
    }

    fn node_updated(&mut self, _old_node: &Node, _new_node: &Node) {
        self.send_window_content_changed_if_needed();
        // TODO: other events
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            send_focus_event_if_applicable(
                self.env,
                self.callback_class,
                self.host,
                self.node_id_map,
                new_node,
            );
        }
    }

    fn node_removed(&mut self, _node: &Node) {
        self.send_window_content_changed_if_needed();
        // TODO: other events?
    }
}

const PLACEHOLDER_ROOT_ID: NodeId = NodeId(0);

#[derive(Default)]
enum State {
    #[default]
    Inactive,
    Placeholder(Tree),
    Active(Tree),
}

impl State {
    fn get_or_init_tree<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> &Tree {
        match self {
            Self::Inactive => {
                *self = match activation_handler.request_initial_tree() {
                    Some(initial_state) => Self::Active(Tree::new(initial_state, true)),
                    None => {
                        let placeholder_update = TreeUpdate {
                            nodes: vec![(
                                PLACEHOLDER_ROOT_ID,
                                NodeBuilder::new(Role::Window).build(),
                            )],
                            tree: Some(TreeData::new(PLACEHOLDER_ROOT_ID)),
                            focus: PLACEHOLDER_ROOT_ID,
                        };
                        Self::Placeholder(Tree::new(placeholder_update, true))
                    }
                };
                self.get_or_init_tree(activation_handler)
            }
            Self::Placeholder(tree) => tree,
            Self::Active(tree) => tree,
        }
    }

    fn get_full_tree(&mut self) -> Option<&Tree> {
        match self {
            Self::Inactive => None,
            Self::Placeholder(_) => None,
            Self::Active(tree) => Some(tree),
        }
    }
}

#[derive(Default)]
pub struct Adapter {
    node_id_map: NodeIdMap,
    state: State,
}

impl Adapter {
    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    pub fn update_if_active<'a>(
        &'a mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
        env: &'a mut JNIEnv<'a>,
        callback_class: &'a JClass<'a>,
        host: &'a JObject<'a>,
    ) {
        match &mut self.state {
            State::Inactive => (),
            State::Placeholder(_) => {
                let tree = Tree::new(update_factory(), true);
                send_window_content_changed(env, callback_class, host);
                let state = tree.state();
                if let Some(focus) = state.focus() {
                    send_focus_event_if_applicable(
                        env,
                        callback_class,
                        host,
                        &mut self.node_id_map,
                        &focus,
                    );
                }
                self.state = State::Active(tree);
            }
            State::Active(tree) => {
                let mut handler =
                    AdapterChangeHandler::new(env, callback_class, host, &mut self.node_id_map);
                tree.update_and_process_changes(update_factory(), &mut handler);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn populate_node_info<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        env: &mut JNIEnv,
        host: &JObject,
        host_screen_x: jint,
        host_screen_y: jint,
        virtual_view_id: jint,
        jni_node: &JObject,
    ) -> Result<bool> {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let node = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root()
        } else {
            let Some(accesskit_id) = self.node_id_map.get_accesskit_id(virtual_view_id) else {
                return Ok(false);
            };
            let Some(node) = tree_state.node_by_id(accesskit_id) else {
                return Ok(false);
            };
            node
        };

        let wrapper = NodeWrapper(&node);
        wrapper.populate_node_info(
            env,
            host,
            host_screen_x,
            host_screen_y,
            &mut self.node_id_map,
            jni_node,
        )?;
        Ok(true)
    }

    pub fn virtual_view_at_point<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        x: jfloat,
        y: jfloat,
    ) -> jint {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let root = tree_state.root();
        let point = Point::new(x.into(), y.into());
        let point = root.transform().inverse() * point;
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        self.node_id_map.get_or_create_java_id(&node)
    }

    pub fn perform_action<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        _env: &mut JNIEnv,
        _host: &JObject,
        virtual_view_id: jint,
        action: jint,
        _arguments: &JObject,
    ) -> Result<bool> {
        let Some(tree) = self.state.get_full_tree() else {
            return Ok(false);
        };
        let tree_state = tree.state();
        let target = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root_id()
        } else {
            let Some(accesskit_id) = self.node_id_map.get_accesskit_id(virtual_view_id) else {
                return Ok(false);
            };
            accesskit_id
        };
        let request = match action {
            ACTION_CLICK => ActionRequest {
                action: {
                    let node = tree_state.node_by_id(target).unwrap();
                    if node.is_focusable() && !node.is_focused() && !node.is_clickable() {
                        Action::Focus
                    } else {
                        Action::Default
                    }
                },
                target,
                data: None,
            },
            ACTION_FOCUS => ActionRequest {
                action: Action::Focus,
                target,
                data: None,
            },
            _ => {
                return Ok(false);
            }
        };
        action_handler.do_action(request);
        Ok(true)
    }
}
