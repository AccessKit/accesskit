// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use accesskit_consumer::Node;
use jni::{objects::JObject, sys::jint, JNIEnv};
use std::collections::HashMap;

pub(crate) const ACTION_FOCUS: jint = 1 << 0;
pub(crate) const ACTION_CLICK: jint = 1 << 4;
pub(crate) const ACTION_ACCESSIBILITY_FOCUS: jint = 1 << 6;
pub(crate) const ACTION_CLEAR_ACCESSIBILITY_FOCUS: jint = 1 << 7;
pub(crate) const ACTION_NEXT_AT_MOVEMENT_GRANULARITY: jint = 1 << 8;
pub(crate) const ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY: jint = 1 << 9;
pub(crate) const ACTION_SCROLL_FORWARD: jint = 1 << 12;
pub(crate) const ACTION_SCROLL_BACKWARD: jint = 1 << 13;
pub(crate) const ACTION_SET_SELECTION: jint = 1 << 17;

pub(crate) const ACTION_ARGUMENT_MOVEMENT_GRANULARITY_INT: &str =
    "ACTION_ARGUMENT_MOVEMENT_GRANULARITY_INT";
pub(crate) const ACTION_ARGUMENT_EXTEND_SELECTION_BOOLEAN: &str =
    "ACTION_ARGUMENT_EXTEND_SELECTION_BOOLEAN";
pub(crate) const ACTION_ARGUMENT_SELECTION_START_INT: &str = "ACTION_ARGUMENT_SELECTION_START_INT";
pub(crate) const ACTION_ARGUMENT_SELECTION_END_INT: &str = "ACTION_ARGUMENT_SELECTION_END_INT";

pub(crate) const CONTENT_CHANGE_TYPE_SUBTREE: jint = 1 << 0;

pub(crate) const EVENT_VIEW_CLICKED: jint = 1;
pub(crate) const EVENT_VIEW_FOCUSED: jint = 1 << 3;
pub(crate) const EVENT_VIEW_TEXT_CHANGED: jint = 1 << 4;
pub(crate) const EVENT_VIEW_HOVER_ENTER: jint = 1 << 7;
pub(crate) const EVENT_VIEW_HOVER_EXIT: jint = 1 << 8;
pub(crate) const EVENT_VIEW_SCROLLED: jint = 1 << 12;
pub(crate) const EVENT_VIEW_TEXT_SELECTION_CHANGED: jint = 1 << 13;
pub(crate) const EVENT_VIEW_ACCESSIBILITY_FOCUSED: jint = 1 << 15;
pub(crate) const EVENT_VIEW_ACCESSIBILITY_FOCUS_CLEARED: jint = 1 << 16;
pub(crate) const EVENT_VIEW_TEXT_TRAVERSED_AT_MOVEMENT_GRANULARITY: jint = 1 << 17;
pub(crate) const EVENT_WINDOW_CONTENT_CHANGED: jint = 1 << 11;

pub(crate) const FOCUS_INPUT: jint = 1;
pub(crate) const FOCUS_ACCESSIBILITY: jint = 2;

pub(crate) const HOST_VIEW_ID: jint = -1;

pub(crate) const LIVE_REGION_NONE: jint = 0;
pub(crate) const LIVE_REGION_POLITE: jint = 1;
pub(crate) const LIVE_REGION_ASSERTIVE: jint = 2;

pub(crate) const MOTION_ACTION_HOVER_MOVE: jint = 7;
pub(crate) const MOTION_ACTION_HOVER_ENTER: jint = 9;
pub(crate) const MOTION_ACTION_HOVER_EXIT: jint = 10;

pub(crate) const MOVEMENT_GRANULARITY_CHARACTER: jint = 1 << 0;
pub(crate) const MOVEMENT_GRANULARITY_WORD: jint = 1 << 1;
pub(crate) const MOVEMENT_GRANULARITY_LINE: jint = 1 << 2;
pub(crate) const MOVEMENT_GRANULARITY_PARAGRAPH: jint = 1 << 3;

#[derive(Debug, Default)]
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

pub(crate) fn bundle_contains_key(env: &mut JNIEnv, bundle: &JObject, key: &str) -> bool {
    let key = env.new_string(key).unwrap();
    env.call_method(
        bundle,
        "containsKey",
        "(Ljava/lang/String;)Z",
        &[(&key).into()],
    )
    .unwrap()
    .z()
    .unwrap()
}

pub(crate) fn bundle_get_int(env: &mut JNIEnv, bundle: &JObject, key: &str) -> jint {
    let key = env.new_string(key).unwrap();
    env.call_method(bundle, "getInt", "(Ljava/lang/String;)I", &[(&key).into()])
        .unwrap()
        .i()
        .unwrap()
}

pub(crate) fn bundle_get_bool(env: &mut JNIEnv, bundle: &JObject, key: &str) -> bool {
    let key = env.new_string(key).unwrap();
    env.call_method(
        bundle,
        "getBoolean",
        "(Ljava/lang/String;)Z",
        &[(&key).into()],
    )
    .unwrap()
    .z()
    .unwrap()
}

pub(crate) fn get_package_name<'local>(
    env: &mut JNIEnv<'local>,
    view: &JObject,
) -> JObject<'local> {
    let context = env
        .call_method(view, "getContext", "()Landroid/content/Context;", &[])
        .unwrap()
        .l()
        .unwrap();
    env.call_method(&context, "getPackageName", "()Ljava/lang/String;", &[])
        .unwrap()
        .l()
        .unwrap()
}
