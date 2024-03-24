// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).
#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_windows::{SubclassingAdapter, HWND};
use winit::{event::WindowEvent, window::Window};

pub type ActivationHandlerBox = Box<dyn ActivationHandler>;
pub type ActionHandlerBox = Box<dyn ActionHandler + Send>;
pub type DeactivationHandlerBox = Box<dyn DeactivationHandler>;

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        activation_handler: ActivationHandlerBox,
        action_handler: ActionHandlerBox,
        _deactivation_handler: DeactivationHandlerBox,
    ) -> Self {
        #[cfg(feature = "rwh_05")]
        let hwnd = match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => handle.hwnd as isize,
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_06")]
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        };

        let adapter = SubclassingAdapter::new(HWND(hwnd), activation_handler, action_handler);
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(updater) {
            events.raise();
        }
    }

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
