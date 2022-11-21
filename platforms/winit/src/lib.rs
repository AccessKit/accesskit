// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{kurbo::Rect, ActionHandler, ActionRequest, TreeUpdate};
use parking_lot::Mutex;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    window::{Window, WindowId},
};

mod platform_impl;

#[derive(Debug)]
pub struct ActionRequestEvent {
    pub window_id: WindowId,
    pub request: ActionRequest,
}

struct WinitActionHandler<T: From<ActionRequestEvent> + Send + 'static> {
    window_id: WindowId,
    proxy: Mutex<EventLoopProxy<T>>,
}

impl<T: From<ActionRequestEvent> + Send + 'static> ActionHandler for WinitActionHandler<T> {
    fn do_action(&self, request: ActionRequest) {
        let proxy = self.proxy.lock();
        let event = ActionRequestEvent {
            window_id: self.window_id,
            request,
        };
        proxy.send_event(event.into()).ok();
    }
}

pub struct Adapter {
    adapter: platform_impl::Adapter,
}

impl Adapter {
    pub fn new<T: From<ActionRequestEvent> + Send + 'static>(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        event_loop_proxy: EventLoopProxy<T>,
    ) -> Self {
        let action_handler = WinitActionHandler {
            window_id: window.id(),
            proxy: Mutex::new(event_loop_proxy),
        };
        Self::with_action_handler(window, source, Box::new(action_handler))
    }

    /// Use this if you need to provide your own AccessKit action handler
    /// rather than dispatching action requests through the winit event loop.
    /// Remember that an AccessKit action handler can be called on any thread,
    /// depending on the underlying AccessKit platform adapter.
    pub fn with_action_handler(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let adapter = platform_impl::Adapter::new(window, source, action_handler);
        Self { adapter }
    }

    #[cfg(target_os = "linux")]
    pub fn run<F, T>(self, window: Window, event_loop: EventLoop<T>, mut event_handler: F) -> !
    where
        F: 'static + FnMut(&Self, Event<'_, T>, &EventLoopWindowTarget<T>, &mut ControlFlow),
    {
        event_loop.run(move |event, window_target, control_flow| {
            if let Event::WindowEvent { ref event, .. } = event {
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
                        self.adapter.set_root_window_bounds(
                            Rect::from_origin_size(outer_position, outer_size),
                            Rect::from_origin_size(inner_position, inner_size),
                        )
                    }
                    WindowEvent::Resized(outer_size) => {
                        let outer_position: (_, _) = window
                            .outer_position()
                            .unwrap_or_default()
                            .cast::<f64>()
                            .into();
                        let outer_size: (_, _) = outer_size.cast::<f64>().into();
                        let inner_position: (_, _) = window
                            .inner_position()
                            .unwrap_or_default()
                            .cast::<f64>()
                            .into();
                        let inner_size: (_, _) = window.inner_size().cast::<f64>().into();
                        self.adapter.set_root_window_bounds(
                            Rect::from_origin_size(outer_position, outer_size),
                            Rect::from_origin_size(inner_position, inner_size),
                        )
                    }
                    _ => (),
                }
            }
            event_handler(&self, event, window_target, control_flow)
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub fn run<F, T>(&self, window: Window, event_loop: EventLoop<T>, mut event_handler: F) -> !
    where
        F: 'static + FnMut(Event<'_, T>, &EventLoopWindowTarget<T>, &mut ControlFlow),
    {
        event_loop.run(move |event, window_target, control_flow| {
            event_handler(event, window_target, control_flow)
        })
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update)
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater)
    }
}
