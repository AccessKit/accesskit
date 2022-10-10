// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActionRequest, TreeUpdate};
use parking_lot::Mutex;
use winit::{
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
        source: Box<dyn FnOnce() -> TreeUpdate + Send>,
        event_loop_proxy: EventLoopProxy<T>,
    ) -> Self {
        let action_handler = WinitActionHandler {
            window_id: window.id(),
            proxy: Mutex::new(event_loop_proxy),
        };
        let adapter = platform_impl::Adapter::new(window, source, Box::new(action_handler));
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update)
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater)
    }
}
