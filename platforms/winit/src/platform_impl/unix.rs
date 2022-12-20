// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{kurbo::Rect, ActionHandler, TreeUpdate};
use accesskit_unix::Adapter as UnixAdapter;
use winit::window::Window;

pub struct Adapter {
    adapter: UnixAdapter,
}

impl Adapter {
    pub fn new(
        _: &Window,
        source: Box<dyn FnOnce() -> TreeUpdate>,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let adapter = UnixAdapter::new(
            String::new(),
            String::new(),
            String::new(),
            source,
            action_handler,
        );
        Self { adapter }
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        self.adapter.set_root_window_bounds(outer, inner);
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update).raise();
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update(updater()).raise();
    }
}
