// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_macos::SubclassingAdapter;
use winit::{
    event::WindowEvent,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};

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
        let view = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr(),
            RawWindowHandle::UiKit(_) => unimplemented!(),
            _ => unreachable!(),
        };
        let adapter = unsafe { SubclassingAdapter::new(view, source, action_handler) };
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        let events = self.adapter.update(update);
        events.raise();
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(updater) {
            events.raise();
        }
    }

    pub fn process_event(&self, _window: &Window, event: &WindowEvent) {
        if let WindowEvent::Focused(is_focused) = event {
            if let Some(events) = self.adapter.update_view_focus_state(*is_focused) {
                events.raise();
            }
        }
    }
}
