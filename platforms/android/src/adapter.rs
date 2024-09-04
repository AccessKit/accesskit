// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, NodeBuilder, NodeId, Point, Role,
    Tree as TreeData, TreeUpdate,
};
use accesskit_consumer::Tree;
use jni::{
    errors::Result,
    objects::{JClass, JObject},
    sys::{jfloat, jint},
    JNIEnv,
};

use crate::{filters::filter, node::NodeWrapper, util::*};

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
    pub fn update_if_active(
        &mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
        env: &mut JNIEnv,
        callback_class: &JClass,
        host: &JObject,
    ) {
        match &mut self.state {
            State::Inactive => {
                return;
            }
            State::Placeholder(_) => {
                self.state = State::Active(Tree::new(update_factory(), true));
            }
            State::Active(tree) => {
                tree.update(update_factory());
            }
        }
        // TODO: Send other events; only send a change event if the tree
        // actually changed.
        send_event(
            env,
            callback_class,
            host,
            HOST_VIEW_ID,
            EVENT_WINDOW_CONTENT_CHANGED,
        );
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
                action: Action::Default,
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
