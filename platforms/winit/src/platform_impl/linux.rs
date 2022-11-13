// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_linux::Adapter as LinuxAdapter;
use winit::window::Window;

pub struct Adapter {
    adapter: LinuxAdapter,
}

impl Adapter {
    pub fn new(
        _: &Window,
        source: Box<dyn FnOnce() -> TreeUpdate>,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let adapter = LinuxAdapter::new(
            String::new(),
            String::new(),
            String::new(),
            source,
            action_handler,
        );
        Self { adapter }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update).raise();
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update(updater()).raise();
    }
}
