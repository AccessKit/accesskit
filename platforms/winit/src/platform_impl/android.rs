// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_android::{
    jni::{objects::JObject, JavaVM},
    InjectingAdapter,
};
use winit::{
    event::WindowEvent, event_loop::ActiveEventLoop, platform::android::ActiveEventLoopExtAndroid,
    window::Window,
};

pub struct Adapter {
    adapter: InjectingAdapter,
}

impl Adapter {
    pub fn new(
        event_loop: &ActiveEventLoop,
        _window: &Window,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        _deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        let app = event_loop.android_app();
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.get_env().unwrap();
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as *mut _) };
        let view = env
            .get_field(
                &activity,
                "mSurfaceView",
                "Lcom/google/androidgamesdk/GameActivity$InputEnabledSurfaceView;",
            )
            .unwrap()
            .l()
            .unwrap();
        let adapter =
            InjectingAdapter::new(&mut env, &view, activation_handler, action_handler).unwrap();
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn process_event(&mut self, _window: &Window, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.adapter
                    .handle_hover_enter_or_move(position.x as _, position.y as _);
            }
            WindowEvent::CursorLeft { .. } => {
                self.adapter.handle_hover_exit();
            }
            _ => (),
        }
    }
}
