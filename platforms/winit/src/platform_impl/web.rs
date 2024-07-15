// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_web::Adapter as WebAdapter;
use winit::{event::WindowEvent, platform::web::WindowExtWebSys, window::Window};

pub struct Adapter {
    adapter: WebAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler + Send,
        _deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        let parent_id = window
            .canvas()
            .expect("cannot get canvas element for the window")
            .id();
        let adapter = WebAdapter::new(&parent_id, activation_handler, action_handler);
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn process_event(&mut self, _window: &Window, event: &WindowEvent) {
        if let WindowEvent::Focused(is_focused) = event {
            self.adapter.update_host_focus_state(*is_focused);
        }
    }
}
