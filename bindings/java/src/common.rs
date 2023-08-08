// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::*;
use jni::{
    objects::{JByteArray, JClass},
    sys::{jdouble, jint, jlong},
    JNIEnv,
};

use crate::{box_from_jptr, box_str_from_utf8_jbytes, into_jptr, mut_from_jptr};

fn node_id_from_parts(low: jlong, high: jlong) -> NodeId {
    let num = ((high as u128) << 64) | (low as u128);
    NodeId(unsafe { std::num::NonZeroU128::new_unchecked(num) })
}

fn optional_node_id_from_parts(low: jlong, high: jlong) -> Option<NodeId> {
    (low != 0 || high != 0).then(|| node_id_from_parts(low, high))
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_Node_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<Node>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeNew(
    _env: JNIEnv,
    _class: JClass,
    role: jint,
) -> jlong {
    let role = Role::n(role as u8).unwrap();
    let builder = NodeBuilder::new(role);
    into_jptr(builder)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<NodeBuilder>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeAddAction(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    action: jint,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let action = Action::n(action as u8).unwrap();
    builder.add_action(action);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetDefaultActionVerb(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jint,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let value = DefaultActionVerb::n(value as u8).unwrap();
    builder.set_default_action_verb(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetName(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: JByteArray,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let value = box_str_from_utf8_jbytes(&mut env, value);
    builder.set_name(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetValue(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: JByteArray,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let value = box_str_from_utf8_jbytes(&mut env, value);
    builder.set_value(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetBounds(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    x0: jdouble,
    y0: jdouble,
    x1: jdouble,
    y1: jdouble,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.set_bounds(Rect { x0, y0, x1, y1 })
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeAddChild(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    id_low: jlong,
    id_high: jlong,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let id = node_id_from_parts(id_low, id_high);
    builder.push_child(id);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeClearChildren(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.clear_children();
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetCheckedState(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jint,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let value = CheckedState::n(value as u8).unwrap();
    builder.set_checked_state(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetLive(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jint,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let value = Live::n(value as u8).unwrap();
    builder.set_live(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetNumericValue(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jdouble,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.set_numeric_value(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetMinNumericValue(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jdouble,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.set_min_numeric_value(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetMaxNumericValue(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jdouble,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.set_max_numeric_value(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetNumericValueStep(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jdouble,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.set_numeric_value_step(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetNumericValueJump(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: jdouble,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    builder.set_numeric_value_jump(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetTextSelection(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    anchor_id_low: jlong,
    anchor_id_high: jlong,
    anchor_character_index: jint,
    focus_id_low: jlong,
    focus_id_high: jlong,
    focus_character_index: jint,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let anchor_id = node_id_from_parts(anchor_id_low, anchor_id_high);
    let focus_id = node_id_from_parts(focus_id_low, focus_id_high);
    builder.set_text_selection(TextSelection {
        anchor: TextPosition {
            node: anchor_id,
            character_index: anchor_character_index as usize,
        },
        focus: TextPosition {
            node: focus_id,
            character_index: focus_character_index as usize,
        },
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeBuild(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jlong {
    let builder = box_from_jptr::<NodeBuilder>(ptr);
    let mut classes = NodeClassSet::lock_global();
    let node = builder.build(&mut classes);
    into_jptr(node)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeNew(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let update = TreeUpdate::default();
    into_jptr(update)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<TreeUpdate>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeAddNode(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    id_low: jlong,
    id_high: jlong,
    node_ptr: jlong,
) {
    let update = mut_from_jptr::<TreeUpdate>(ptr);
    let id = node_id_from_parts(id_low, id_high);
    let node = box_from_jptr::<Node>(node_ptr);
    update.nodes.push((id, *node));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeSetTree(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    root_low: jlong,
    root_high: jlong,
    root_scroller_low: jlong,
    root_scroller_high: jlong,
) {
    let update = mut_from_jptr::<TreeUpdate>(ptr);
    let root = node_id_from_parts(root_low, root_high);
    let root_scroller = optional_node_id_from_parts(root_scroller_low, root_scroller_high);
    update.tree = Some(Tree {
        root,
        root_scroller,
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeClearTree(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let update = mut_from_jptr::<TreeUpdate>(ptr);
    update.tree = None;
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeSetFocus(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    id_low: jlong,
    id_high: jlong,
) {
    let update = mut_from_jptr::<TreeUpdate>(ptr);
    let id = node_id_from_parts(id_low, id_high);
    update.focus = Some(id);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeClearFocus(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let update = mut_from_jptr::<TreeUpdate>(ptr);
    update.focus = None;
}
