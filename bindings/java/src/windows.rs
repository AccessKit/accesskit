// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::*;
use accesskit_windows::*;
use jni::{
    objects::{JClass, JObject},
    sys::jlong,
    JNIEnv,
};

use crate::{box_from_jptr, into_jptr, ref_from_jptr};

// TODO: eliminate the need for this
struct NullActionHandler;

impl ActionHandler for NullActionHandler {
    fn do_action(&self, _request: ActionRequest) {}
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_WindowsSubclassingAdapter_nativeNew(
    env: JNIEnv,
    _class: JClass,
    hwnd: jlong,
    initial_state_supplier: JObject,
) -> jlong {
    let hwnd = HWND(hwnd as _);
    let jvm = env.get_java_vm().unwrap();
    let initial_state_supplier = env.new_global_ref(initial_state_supplier).unwrap();
    let initial_state_source = move || {
        let mut env = jvm.attach_current_thread().unwrap();
        let ptr = env
            .call_method(&initial_state_supplier, "get", "()J", &[])
            .unwrap()
            .j()
            .unwrap();
        *box_from_jptr::<TreeUpdate>(ptr)
    };
    // TODO: real action handler
    let adapter =
        SubclassingAdapter::new(hwnd, initial_state_source, Box::new(NullActionHandler {}));
    into_jptr(adapter)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_WindowsSubclassingAdapter_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<SubclassingAdapter>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_WindowsSubclassingAdapter_nativeUpdateIfActive(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    update_supplier: JObject,
) {
    let adapter = ref_from_jptr::<SubclassingAdapter>(ptr);
    let update_source = move || {
        let ptr = env
            .call_method(&update_supplier, "get", "()J", &[])
            .unwrap()
            .j()
            .unwrap();
        *box_from_jptr::<TreeUpdate>(ptr)
    };
    if let Some(events) = adapter.update_if_active(update_source) {
        events.raise();
    }
}
