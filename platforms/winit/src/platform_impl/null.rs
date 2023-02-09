// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, TreeUpdate};
use winit::window::Window;

pub type ActionHandlerBox = Box<dyn ActionHandler>;

pub struct Adapter;

impl Adapter {
    pub fn new(
        _window: &Window,
        _source: impl 'static + FnOnce() -> TreeUpdate,
        _action_handler: ActionHandlerBox,
    ) -> Self {
        Self {}
    }

    pub fn update(&self, _update: TreeUpdate) {}

    pub fn update_if_active(&self, _updater: impl FnOnce() -> TreeUpdate) {}
}
