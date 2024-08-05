// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use accesskit_consumer::Node;
use jni::sys::jint;
use std::collections::HashMap;

pub(crate) const HOST_VIEW_ID: jint = -1;

#[derive(Default)]
pub(crate) struct NodeIdMap {
    java_to_accesskit: HashMap<jint, NodeId>,
    accesskit_to_java: HashMap<NodeId, jint>,
    next_java_id: jint,
}

impl NodeIdMap {
    pub(crate) fn get_accesskit_id(&self, java_id: jint) -> Option<NodeId> {
        self.java_to_accesskit.get(&java_id).copied()
    }

    pub(crate) fn get_or_create_java_id(&mut self, node: &Node) -> jint {
        if node.is_root() {
            return HOST_VIEW_ID;
        }
        let accesskit_id = node.id();
        if let Some(id) = self.accesskit_to_java.get(&accesskit_id) {
            return *id;
        }
        let java_id = self.next_java_id;
        self.next_java_id += 1;
        self.accesskit_to_java.insert(accesskit_id, java_id);
        self.java_to_accesskit.insert(java_id, accesskit_id);
        java_id
    }
}
