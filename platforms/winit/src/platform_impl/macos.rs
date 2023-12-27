// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).
#[cfg(feature = "rwh_05")]
use crate::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[cfg(feature = "rwh_06")]
use crate::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_macos::SubclassingAdapter;
use winit::{event::WindowEvent, window::Window};

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

        let adapter = unsafe { SubclassingAdapter::new(view, source, action_handler) };
        Self { adapter }
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
