// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::*;
use jni::{objects::JClass, sys::{jint, jlong}, JNIEnv};

use crate::{box_from_jptr, into_jptr, mut_from_jptr, ref_from_jptr};

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeNew(
    _env: JNIEnv,
    _class: JClass,
    role: jint,
) -> jlong {
    let role = role as Role;
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
