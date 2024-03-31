// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, Rect, TreeUpdate};
use accesskit_unix::Adapter as UnixAdapter;
use winit::{event::WindowEvent, window::Window};

pub struct Adapter {
    adapter: UnixAdapter,
}

impl Adapter {
    pub fn new(
        _: &Window,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        deactivation_handler: impl 'static + DeactivationHandler + Send,
    ) -> Self {
        let adapter = UnixAdapter::new(activation_handler, action_handler, deactivation_handler);
        Self { adapter }
    }

    fn set_root_window_bounds(&mut self, outer: Rect, inner: Rect) {
        self.adapter.set_root_window_bounds(outer, inner);
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }

    fn update_window_focus_state(&mut self, is_focused: bool) {
        self.adapter.update_window_focus_state(is_focused);
    }

    pub fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        match event {
            WindowEvent::Moved(outer_position) => {
                let outer_position: (_, _) = outer_position.cast::<f64>().into();
                let outer_size: (_, _) = window.outer_size().cast::<f64>().into();
                let inner_position: (_, _) = window
                    .inner_position()
                    .unwrap_or_default()
                    .cast::<f64>()
                    .into();
                let inner_size: (_, _) = window.inner_size().cast::<f64>().into();
                self.set_root_window_bounds(
                    Rect::from_origin_size(outer_position, outer_size),
                    Rect::from_origin_size(inner_position, inner_size),
                )
            }
            WindowEvent::Resized(inner_size) => {
                let outer_position: (_, _) = window
                    .outer_position()
                    .unwrap_or_default()
                    .cast::<f64>()
                    .into();
                let outer_size: (_, _) = window.outer_size().cast::<f64>().into();
                let inner_position: (_, _) = window
                    .inner_position()
                    .unwrap_or_default()
                    .cast::<f64>()
                    .into();
                let inner_size: (_, _) = inner_size.cast::<f64>().into();
                self.set_root_window_bounds(
                    Rect::from_origin_size(outer_position, outer_size),
                    Rect::from_origin_size(inner_position, inner_size),
                )
            }
            WindowEvent::Focused(is_focused) => {
                self.update_window_focus_state(*is_focused);
            }
            _ => (),
        }
    }
}
