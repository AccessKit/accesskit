// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_macos::Adapter as MacOSAdapter;
use winit::{platform::macos::WindowExtMacOS, window::Window};

pub struct Adapter {
    adapter: MacOSAdapter,
}

impl Adapter {
    pub fn new(
        window: &Window,
        source: Box<dyn FnOnce() -> TreeUpdate>,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        // TODO: fix when the macOS adapter supports laziness
        let adapter =
            unsafe { MacOSAdapter::new(window.ns_view() as *mut _, source(), action_handler) };
        adapter.inject();
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update);
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        // TODO: fix when the macOS adapter supports laziness
        self.adapter.update(updater());
    }
}
