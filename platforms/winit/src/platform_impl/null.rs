// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, Node, NodeId, Role, Tree};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub struct TreeUpdate;

impl accesskit::TreeUpdate for TreeUpdate {
    fn set_node(&mut self, _: NodeId, _: Role, _: impl FnOnce(&mut Node)) {}
    fn set_tree(&mut self, _: Tree) {}
    fn set_focus(&mut self, _: NodeId) {}
}

pub struct Adapter;

impl Adapter {
    pub fn new(
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _activation_handler: impl 'static + ActivationHandler,
        _action_handler: impl 'static + ActionHandler,
        _deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        Self {}
    }

    pub fn update_if_active(&mut self, _fill: impl FnOnce(&mut TreeUpdate)) {}

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
