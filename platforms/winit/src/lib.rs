// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActionRequest, TreeUpdate};
#[cfg(any(
    all(
        feature = "accesskit_unix",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ),
    target_os = "windows"
))]
use std::sync::{Mutex, MutexGuard};
use winit::{
    event::WindowEvent,
    event_loop::EventLoopProxy,
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
    #[cfg(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    ))]
    proxy: Mutex<EventLoopProxy<T>>,
    #[cfg(not(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    )))]
    proxy: EventLoopProxy<T>,
}

impl<T: From<ActionRequestEvent> + Send + 'static> WinitActionHandler<T> {
    #[cfg(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    ))]
    fn new(window_id: WindowId, proxy: EventLoopProxy<T>) -> Self {
        Self {
            window_id,
            proxy: Mutex::new(proxy),
        }
    }
    #[cfg(not(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    )))]
    fn new(window_id: WindowId, proxy: EventLoopProxy<T>) -> Self {
        Self { window_id, proxy }
    }

    #[cfg(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    ))]
    fn proxy(&self) -> MutexGuard<'_, EventLoopProxy<T>> {
        self.proxy.lock().unwrap()
    }
    #[cfg(not(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    )))]
    fn proxy(&self) -> &EventLoopProxy<T> {
        &self.proxy
    }
}

impl<T: From<ActionRequestEvent> + Send + 'static> ActionHandler for WinitActionHandler<T> {
    fn do_action(&self, request: ActionRequest) {
        let event = ActionRequestEvent {
            window_id: self.window_id,
            request,
        };
        self.proxy().send_event(event.into()).ok();
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
        let action_handler = WinitActionHandler::new(window.id(), event_loop_proxy);
        Self::with_action_handler(window, source, Box::new(action_handler))
    }

    /// Use this if you need to provide your own AccessKit action handler
    /// rather than dispatching action requests through the winit event loop.
    /// Remember that an AccessKit action handler can be called on any thread,
    /// depending on the underlying AccessKit platform adapter.
    pub fn with_action_handler(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: platform_impl::ActionHandlerBox,
    ) -> Self {
        let adapter = platform_impl::Adapter::new(window, source, action_handler);
        Self { adapter }
    }

    #[cfg(not(all(
        feature = "accesskit_unix",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    )))]
    #[must_use]
    pub fn on_event(&self, _window: &Window, _event: &WindowEvent) -> bool {
        true
    }
    #[cfg(all(
        feature = "accesskit_unix",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    #[must_use]
    pub fn on_event(&self, window: &Window, event: &WindowEvent) -> bool {
        use accesskit::Rect;

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
        true
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update)
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater)
    }
}
