// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).
#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler};
use accesskit_windows::{SubclassingAdapter, HWND};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub use accesskit_windows::TreeUpdate;

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        _event_loop: &ActiveEventLoop,
        window: &Window,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler + Send,
        _deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        #[cfg(feature = "rwh_05")]
        let hwnd = match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => handle.hwnd,
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        };
        #[cfg(feature = "rwh_06")]
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => handle.hwnd.get() as *mut _,
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        };

        let adapter = SubclassingAdapter::new(HWND(hwnd), activation_handler, action_handler);
        Self { adapter }
    }

    pub fn update_if_active(&mut self, fill: impl FnOnce(&mut TreeUpdate)) {
        self.adapter.update_if_active(fill);
    }

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
