// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::TreeUpdate;
use accesskit_mac::Adapter;
use cocoa::appkit::NSWindow;
use cocoa::base::id;
use jni::objects::{JClass, JString};
use jni::sys::jlong;
use jni::JNIEnv;

fn new_common(env: JNIEnv, view: id, initial_state_json: JString) -> jlong {
    let initial_state_json: String = env.get_string(initial_state_json).unwrap().into();
    let initial_state = serde_json::from_str::<TreeUpdate>(&initial_state_json).unwrap();
    let adapter = Adapter::new(view, initial_state);
    Box::into_raw(Box::new(adapter)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_mac_AccessKitMacAdapter_nativeNewForNSWindow(
    env: JNIEnv,
    _class: JClass,
    window_ptr: jlong,
    initial_state_json: JString,
) -> jlong {
    let window = window_ptr as id;
    let view = unsafe { window.contentView() };
    new_common(env, view, initial_state_json)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_mac_AccessKitMacAdapter_nativeNewForNSView(
    env: JNIEnv,
    _class: JClass,
    view_ptr: jlong,
    initial_state_json: JString,
) -> jlong {
    let view = view_ptr as id;
    new_common(env, view, initial_state_json)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_mac_AccessKitMacAdapter_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let _boxed_adapter = unsafe { Box::from_raw(ptr as *mut Adapter) };
    // Let the box drop at the end of the scope.
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_mac_AccessKitMacAdapter_nativeUpdate(
    env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    update_json: JString,
) {
    let adapter = unsafe { &mut *(ptr as *mut Adapter) };
    let update_json: String = env.get_string(update_json).unwrap().into();
    let update = serde_json::from_str::<TreeUpdate>(&update_json).unwrap();
    adapter.update(update);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_mac_AccessKitMacAdapter_nativeInject(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let adapter = unsafe { &mut *(ptr as *mut Adapter) };
    adapter.inject();
}
