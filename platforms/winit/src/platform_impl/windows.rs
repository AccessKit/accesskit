// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_windows::{SubclassingAdapter, HWND};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::{event::WindowEvent, window::Window};

pub type ActionHandlerBox = Box<dyn ActionHandler + Send>;

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: ActionHandlerBox,
    ) -> Self {
        let hwnd = HWND(match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => handle.hwnd as isize,
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        });
        let adapter = SubclassingAdapter::new(hwnd, source, action_handler);
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update).raise();
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(updater) {
            events.raise();
        }
    }

    pub fn on_event(&self, _window: &Window, _event: &WindowEvent) -> bool {
        true
    }
}
