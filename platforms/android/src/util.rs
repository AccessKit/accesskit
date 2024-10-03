// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use accesskit_consumer::Node;
use jni::{
    objects::{JClass, JObject},
    sys::jint,
    JNIEnv,
};
use std::collections::HashMap;

pub(crate) const ACTION_FOCUS: jint = 1 << 0;
pub(crate) const ACTION_CLICK: jint = 1 << 4;
pub(crate) const ACTION_NEXT_AT_MOVEMENT_GRANULARITY: jint = 1 << 8;
pub(crate) const ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY: jint = 1 << 9;
pub(crate) const ACTION_SET_SELECTION: jint = 1 << 17;
pub(crate) const EVENT_VIEW_FOCUSED: jint = 1 << 3;
pub(crate) const EVENT_VIEW_HOVER_ENTER: jint = 1 << 7;
pub(crate) const EVENT_VIEW_HOVER_EXIT: jint = 1 << 8;
pub(crate) const EVENT_WINDOW_CONTENT_CHANGED: jint = 1 << 11;
pub(crate) const HOST_VIEW_ID: jint = -1;
pub(crate) const LIVE_REGION_NONE: jint = 0;
pub(crate) const LIVE_REGION_POLITE: jint = 1;
pub(crate) const LIVE_REGION_ASSERTIVE: jint = 2;
pub(crate) const MOVEMENT_GRANULARITY_CHARACTER: jint = 1 << 0;
pub(crate) const MOVEMENT_GRANULARITY_WORD: jint = 1 << 1;
pub(crate) const MOVEMENT_GRANULARITY_LINE: jint = 1 << 2;
pub(crate) const MOVEMENT_GRANULARITY_PARAGRAPH: jint = 1 << 3;

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

pub(crate) fn send_event(
    env: &mut JNIEnv,
    callback_class: &JClass,
    host: &JObject,
    virtual_view_id: jint,
    event_type: jint,
) {
    env.call_static_method(
        callback_class,
        "sendEvent",
        "(Landroid/view/View;II)V",
        &[host.into(), virtual_view_id.into(), event_type.into()],
    )
    .unwrap();
}
