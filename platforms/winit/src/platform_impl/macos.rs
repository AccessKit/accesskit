// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_macos::SubclassingAdapter;
use winit::{event::WindowEvent, platform::macos::WindowExtMacOS, window::Window};

pub type ActionHandlerBox = Box<dyn ActionHandler>;

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: ActionHandlerBox,
    ) -> Self {
        let view = window.ns_view();
        let adapter = unsafe { SubclassingAdapter::new(view, source, action_handler) };
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update);
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    pub fn on_event(&self, _window: &Window, event: &WindowEvent) -> bool {
        if let WindowEvent::Focused(is_focused) = event {
            self.adapter.update_view_focus_state(*is_focused);
        }

        true
    }
}
