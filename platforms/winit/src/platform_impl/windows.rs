// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_windows::{Adapter as WindowsAdapter, SubclassingAdapter};
use windows::Win32::Foundation::HWND;
use winit::{platform::windows::WindowExtWindows, window::Window};

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        source: Box<dyn FnOnce() -> TreeUpdate>,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let hwnd = HWND(window.hwnd());
        let adapter = WindowsAdapter::new(hwnd, source, action_handler);
        let adapter = SubclassingAdapter::new(adapter);
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update).raise();
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater).raise();
    }
}
