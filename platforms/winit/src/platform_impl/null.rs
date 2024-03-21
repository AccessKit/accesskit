// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use winit::{event::WindowEvent, window::Window};

pub type ActivationHandlerBox = Box<dyn ActivationHandler>;
pub type ActionHandlerBox = Box<dyn ActionHandler>;
pub type DeactivationHandlerBox = Box<dyn DeactivationHandler>;

pub struct Adapter;

impl Adapter {
    pub fn new(
        _window: &Window,
        _activation_handler: ActivationHandlerBox,
        _action_handler: ActionHandlerBox,
        _deactivation_handler: DeactivationHandlerBox,
    ) -> Self {
        Self {}
    }

    pub fn update_if_active(&mut self, _updater: impl FnOnce() -> TreeUpdate) {}

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
