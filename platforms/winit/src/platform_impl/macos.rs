// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).
#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_macos::SubclassingAdapter;
use winit::{event::WindowEvent, window::Window};

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
        _deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        #[cfg(feature = "rwh_05")]
        let view = match window.raw_window_handle() {
            RawWindowHandle::AppKit(handle) => handle.ns_view,
            RawWindowHandle::UiKit(_) => unimplemented!(),
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_06")]
        let view = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr(),
            RawWindowHandle::UiKit(_) => unimplemented!(),
            _ => unreachable!(),
        };

        let adapter = unsafe { SubclassingAdapter::new(view, activation_handler, action_handler) };
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(updater) {
            events.raise();
        }
    }

    pub fn process_event(&mut self, _window: &Window, event: &WindowEvent) {
        if let WindowEvent::Focused(is_focused) = event {
            if let Some(events) = self.adapter.update_view_focus_state(*is_focused) {
                events.raise();
            }
        }
    }
}
