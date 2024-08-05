// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    ActivationHandler, NodeBuilder, NodeId, Point, Role, Tree as TreeData, TreeUpdate,
};
use accesskit_consumer::Tree;
use jni::{
    errors::Result,
    objects::JObject,
    sys::{jfloat, jint},
    JNIEnv,
};

use crate::{
    filters::filter,
    node::NodeWrapper,
    util::{NodeIdMap, HOST_VIEW_ID},
};

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
    ///
    /// TODO: dispatch events
    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        match &mut self.state {
            State::Inactive => (),
            State::Placeholder(_) => {
                self.state = State::Active(Tree::new(update_factory(), true));
            }
            State::Active(tree) => {
                tree.update(update_factory());
            }
        }
    }

    pub fn populate_node_info<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        env: &mut JNIEnv,
        host: &JObject,
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
        wrapper.populate_node_info(env, host, &mut self.node_id_map, jni_node)?;
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
}
