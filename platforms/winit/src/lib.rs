// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Rect, TreeUpdate,
};
use accesskit_adapter::Adapter;
use raw_window_handle::HasWindowHandle as _;
use std::sync::Arc;
use winit_core::event_loop::ActiveEventLoop;
use winit_core::{
    event::WindowEvent as WinitWindowEvent,
    window::{Window, WindowId},
};

pub use accesskit_adapter::WindowEvent;

#[cfg(target_os = "android")]
use winit_android::EventLoopExtAndroid as _;

#[derive(Clone)]
struct WinitHandler {
    window_id: WindowId,
    callback: Arc<dyn Fn(WindowId, WindowEvent) + Send + Sync + 'static>,
}

impl ActivationHandler for WinitHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        (self.callback)(self.window_id, WindowEvent::InitialTreeRequested);
        None
    }
}
impl DeactivationHandler for WinitHandler {
    fn deactivate_accessibility(&mut self) {
        (self.callback)(self.window_id, WindowEvent::AccessibilityDeactivated);
    }
}
impl ActionHandler for WinitHandler {
    fn do_action(&mut self, request: ActionRequest) {
        (self.callback)(self.window_id, WindowEvent::ActionRequested(request));
    }
}

#[derive(Debug)]
pub struct Event {
    pub window_id: WindowId,
    pub window_event: WindowEvent,
}

pub struct WinitAdapter {
    inner: Adapter,
}

impl WinitAdapter {
    /// Creates a new AccessKit adapter for a winit window. This must be done
    /// before the window is shown for the first time. This means that you must
    /// use [`winit::window::WindowAttributes::with_visible`] to make the window
    /// initially invisible, then create the adapter, then show the window.
    ///
    /// This constructor uses a winit event loop proxy to deliver AccessKit
    /// events to the main event loop. The primary disadvantage of this approach
    /// is that it's not possible to synchronously return an initial tree
    /// in response to the [`WindowEvent::InitialTreeRequested`] event,
    /// so some platform adapters will have to use a temporary placeholder tree
    /// until you send the first update. For an optimal implementation,
    /// consider using [`Adapter::with_direct_handlers`] or
    /// [`Adapter::with_mixed_handlers`] instead.
    ///
    /// # Panics
    ///
    /// Panics if the window is already visible.
    pub fn new(
        event_loop: &dyn ActiveEventLoop,
        window: &dyn Window,
        callback: impl Fn(WindowId, WindowEvent) + Send + Sync + 'static,
    ) -> Self {
        let window_id = window.id();
        let handler = WinitHandler {
            window_id,
            callback: Arc::new(callback) as _,
        };

        // Silence unused variable warning
        #[cfg(not(target_os = "android"))]
        let _ = event_loop;

        Self {
            inner: accesskit_adapter::Adapter::with_split_handlers(
                #[cfg(not(target_os = "android"))]
                &window.window_handle().unwrap().as_raw(),
                #[cfg(target_os = "android")]
                event_loop.android_app(),
                handler.clone(),
                handler.clone(),
                handler,
            ),
        }
    }

    /// Allows reacting to window events.
    ///
    /// This must be called whenever a new window event is received
    /// and before it is handled by the application.
    pub fn process_event(&mut self, window: &dyn Window, event: &WinitWindowEvent) {
        match event {
            WinitWindowEvent::Focused(is_focused) => {
                self.inner.set_focus(*is_focused);
            }
            WinitWindowEvent::Moved(_) | WinitWindowEvent::SurfaceResized(_) => {
                let outer_position: (_, _) = window
                    .outer_position()
                    .unwrap_or_default()
                    .cast::<f64>()
                    .into();
                let outer_size: (_, _) = window.outer_size().cast::<f64>().into();
                let inner_position: (_, _) = window.surface_position().cast::<f64>().into();
                let inner_size: (_, _) = window.surface_size().cast::<f64>().into();

                self.inner.set_window_bounds(
                    Rect::from_origin_size(outer_position, outer_size),
                    Rect::from_origin_size(inner_position, inner_size),
                )
            }
            _ => (),
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// or if the caller created the adapter using [`EventLoopProxy`], then
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.inner.update_if_active(updater);
    }
}
