// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use jni::{
    objects::JObject,
    sys::{jint, jvalue},
};
use std::collections::HashMap;

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

    pub(crate) fn get_or_create_java_id(&mut self, accesskit_id: NodeId) -> jint {
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

pub(crate) fn bool_value(value: bool) -> jvalue {
    jvalue { z: value as u8 }
}

pub(crate) fn id_value(id_map: &mut NodeIdMap, value: NodeId) -> jvalue {
    jvalue {
        i: id_map.get_or_create_java_id(value),
    }
}

pub(crate) fn object_value<'local, O>(value: O) -> jvalue
where
    O: AsRef<JObject<'local>>,
{
    jvalue {
        l: value.as_ref().as_raw(),
    }
}
