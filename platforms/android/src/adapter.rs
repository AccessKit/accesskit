// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::TreeUpdate;
use accesskit_consumer::Tree;
use jni::{errors::Result, objects::JObject, sys::jint, JNIEnv};

use crate::{classes::AccessibilityNodeInfo, node::NodeWrapper, util::NodeIdMap};

const HOST_VIEW_ID: jint = -1;

pub struct Adapter {
    node_id_map: NodeIdMap,
    node_info_class: AccessibilityNodeInfo,
    tree: Tree,
}

impl Adapter {
    pub fn new(env: &mut JNIEnv, initial_state: TreeUpdate) -> Result<Self> {
        let node_info_class = AccessibilityNodeInfo::initialize_class(env)?;
        let tree = Tree::new(initial_state, true);
        Ok(Self {
            node_id_map: NodeIdMap::default(),
            node_info_class,
            tree,
        })
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
        wrapper.populate_node_info(
            env,
            host,
            &self.node_info_class,
            &mut self.node_id_map,
            jni_node,
        )?;
        Ok(true)
    }
}
