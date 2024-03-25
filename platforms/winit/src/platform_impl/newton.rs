// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_newton::Adapter as UnixAdapter;
use winit::{event::WindowEvent, window::Window};

#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

pub type ActionHandlerBox = Box<dyn ActionHandler + Send>;

pub struct Adapter {
    adapter: UnixAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: ActionHandlerBox,
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
        let adapter = unsafe { UnixAdapter::new(display, surface, source, action_handler) };
        Self { adapter }
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn process_event(&self, _window: &Window, _event: &WindowEvent) {}
}
