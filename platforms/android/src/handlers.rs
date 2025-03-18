// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};
use jni::{objects::JObject, JNIEnv};

/// Like ['ActivationHandler'], but also receives the JNI environment and
/// a local reference to the Android view.
pub trait ActivationHandlerWithAndroidContext {
    fn request_initial_tree<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &JObject<'local>,
    ) -> Option<TreeUpdate>;
}

impl<T: ActivationHandler> ActivationHandlerWithAndroidContext for T {
    fn request_initial_tree<'local>(
        &mut self,
        _env: &mut JNIEnv<'local>,
        _view: &JObject<'local>,
    ) -> Option<TreeUpdate> {
        self.request_initial_tree()
    }
}

/// Like ['ActionHandler'], but also receives the JNI environment and
/// a local reference to the Android view.
pub trait ActionHandlerWithAndroidContext {
    fn do_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &JObject<'local>,
        request: ActionRequest,
    );
}

impl<T: ActionHandler> ActionHandlerWithAndroidContext for T {
    fn do_action<'local>(
        &mut self,
        _env: &mut JNIEnv<'local>,
        _view: &JObject<'local>,
        request: ActionRequest,
    ) {
        self.do_action(request);
    }
}
