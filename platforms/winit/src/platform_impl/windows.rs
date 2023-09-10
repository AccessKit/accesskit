// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_windows::{SubclassingAdapter, HWND};
use winit::{event::WindowEvent, platform::windows::WindowExtWindows, window::Window};

pub type ActionHandlerBox = Box<dyn ActionHandler + Send + Sync>;

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: ActionHandlerBox,
    ) -> Self {
        let hwnd = HWND(window.hwnd());
        let adapter = SubclassingAdapter::new(hwnd, source, action_handler);
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update);
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn on_event(&self, _window: &Window, _event: &WindowEvent) -> bool {
        true
    }
}
