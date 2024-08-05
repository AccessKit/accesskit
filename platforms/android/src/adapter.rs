// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Point, TreeUpdate};
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

pub struct Adapter {
    node_id_map: NodeIdMap,
    tree: Tree,
}

impl Adapter {
    pub fn new(initial_state: TreeUpdate) -> Self {
        let tree = Tree::new(initial_state, true);
        Self {
            node_id_map: NodeIdMap::default(),
            tree,
        }
    }

    pub fn populate_node_info(
        &mut self,
        env: &mut JNIEnv,
        host: &JObject,
        virtual_view_id: jint,
        jni_node: &JObject,
    ) -> Result<bool> {
        let tree_state = self.tree.state();
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

    pub fn virtual_view_at_point(&mut self, x: jfloat, y: jfloat) -> jint {
        let tree_state = self.tree.state();
        let root = tree_state.root();
        let point = Point::new(x.into(), y.into());
        let point = root.transform().inverse() * point;
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        self.node_id_map.get_or_create_java_id(&node)
    }
}
