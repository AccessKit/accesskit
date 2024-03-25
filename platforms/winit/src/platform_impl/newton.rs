// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_newton::Adapter as NewtonAdapter;
use winit::{event::WindowEvent, window::Window};

#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

pub struct Adapter {
    adapter: NewtonAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        deactivation_handler: impl 'static + DeactivationHandler + Send,
    ) -> Self {
        // TODO: make this function sound!
        #[cfg(feature = "rwh_05")]
        let display = match window.raw_display_handle() {
            RawDisplayHandle::Wayland(handle) => handle.display,
            RawDisplayHandle::Xlib(_) => unimplemented!(),
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_06")]
        let display = match window.display_handle().unwrap().as_raw() {
            RawDisplayHandle::Wayland(handle) => handle.display.as_ptr(),
            RawDisplayHandle::Xlib(_) => unimplemented!(),
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_05")]
        let surface = match window.raw_window_handle() {
            RawWindowHandle::Wayland(handle) => handle.surface,
            RawWindowHandle::Xlib(_) => unimplemented!(),
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_06")]
        let surface = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Wayland(handle) => handle.surface.as_ptr(),
            RawWindowHandle::Xlib(_) => unimplemented!(),
            _ => unreachable!(),
        };
        let adapter = unsafe {
            NewtonAdapter::new(
                display,
                surface,
                activation_handler,
                action_handler,
                deactivation_handler,
            )
        };
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
