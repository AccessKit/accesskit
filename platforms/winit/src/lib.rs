// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

/// ## Compatibility with async runtimes
///
/// The following only applies on Linux/Unix:
///
/// While this crate's API is purely blocking, it internally spawns asynchronous tasks on an executor.
///
/// - If you use tokio, make sure to enable the `tokio` feature of this crate.
/// - If you use another async runtime or if you don't use one at all, the default feature will suit your needs.

#[cfg(all(
    feature = "accesskit_unix",
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    not(feature = "async-io"),
    not(feature = "tokio")
))]
compile_error!("Either \"async-io\" (default) or \"tokio\" feature must be enabled.");

#[cfg(all(
    feature = "accesskit_unix",
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    feature = "async-io",
    feature = "tokio"
))]
compile_error!(
    "Both \"async-io\" (default) and \"tokio\" features cannot be enabled at the same time."
);

#[cfg(all(not(feature = "rwh_05"), not(feature = "rwh_06")))]
compile_error!("Either \"rwh_06\" (default) or \"rwh_05\" feature must be enabled.");

#[cfg(all(feature = "rwh_05", feature = "rwh_06"))]
compile_error!(
    "Both \"rwh_06\" (default) and \"rwh_05\" features cannot be enabled at the same time."
);

use accesskit::{ActionHandler, ActionRequest, TreeUpdate};
use winit::{
    event::WindowEvent,
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
};

#[cfg(feature = "rwh_05")]
#[allow(unused)]
use rwh_05 as raw_window_handle;
#[cfg(feature = "rwh_06")]
#[allow(unused)]
use rwh_06 as raw_window_handle;

mod platform_impl;

#[derive(Debug)]
pub struct ActionRequestEvent {
    pub window_id: WindowId,
    pub request: ActionRequest,
}

struct WinitActionHandler<T: From<ActionRequestEvent> + Send + 'static> {
    window_id: WindowId,
    proxy: EventLoopProxy<T>,
}

impl<T: From<ActionRequestEvent> + Send + 'static> WinitActionHandler<T> {
    fn new(window_id: WindowId, proxy: EventLoopProxy<T>) -> Self {
        Self { window_id, proxy }
    }
}

impl<T: From<ActionRequestEvent> + Send + 'static> ActionHandler for WinitActionHandler<T> {
    fn do_action(&mut self, request: ActionRequest) {
        let event = ActionRequestEvent {
            window_id: self.window_id,
            request,
        };
        self.proxy.send_event(event.into()).ok();
    }
}

pub struct Adapter {
    adapter: platform_impl::Adapter,
}

impl Adapter {
    /// Creates a new AccessKit adapter for a winit window. This must be done
    /// before the window is shown for the first time. This means that you must
    /// use [`winit::window::WindowBuilder::with_visible`] to make the window
    /// initially invisible, then create the adapter, then show the window.
    pub fn new<T: From<ActionRequestEvent> + Send + 'static>(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        event_loop_proxy: EventLoopProxy<T>,
    ) -> Self {
        let action_handler = WinitActionHandler::new(window.id(), event_loop_proxy);
        Self::with_action_handler(window, source, Box::new(action_handler))
    }

    /// Creates a new AccessKit adapter for a winit window. This must be done
    /// before the window is shown for the first time. This means that you must
    /// use [`winit::window::WindowBuilder::with_visible`] to make the window
    /// initially invisible, then create the adapter, then show the window.
    ///
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

    /// Allows reacting to window events.
    ///
    /// This must be called whenever a new window event is received
    /// and before it is handled by the application.
    pub fn process_event(&self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }
}
