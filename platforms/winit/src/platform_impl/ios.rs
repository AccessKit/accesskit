// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_ios::SubclassingAdapter;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        _event_loop: &ActiveEventLoop,
        window: &Window,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
        deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        #[cfg(feature = "rwh_05")]
        let view = match window.raw_window_handle() {
            RawWindowHandle::UiKit(handle) => handle.ui_view,
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_06")]
        let view = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::UiKit(handle) => handle.ui_view.as_ptr(),
            _ => unreachable!(),
        };

        let adapter = unsafe {
            SubclassingAdapter::new(
                view,
                activation_handler,
                action_handler,
                deactivation_handler,
            )
        };
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(updater) {
            events.raise();
        }
    }

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
