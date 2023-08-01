// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::*;
use jni::{objects::JObject, JNIEnv};

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_init(mut env: JNIEnv, obj: JObject) {
    let update = TreeUpdate::default();
    unsafe { env.set_rust_field(&obj, "ptr", update) }.unwrap();
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_drop(mut env: JNIEnv, obj: JObject) {
    let _ = unsafe { env.take_rust_field::<_, _, TreeUpdate>(&obj, "ptr") };
}
