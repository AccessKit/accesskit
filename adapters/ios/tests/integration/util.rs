// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{cell::Cell, rc::Rc};

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send_id};
use objc2_ui_kit::UIView;

#[derive(Default)]
pub struct HandlerEvents {
    pub tree_requested: Cell<bool>,
    pub dropped: Cell<bool>,
}

pub struct NoTree(pub Rc<HandlerEvents>);

impl ActivationHandler for NoTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.0.tree_requested.set(true);
        None
    }
}

impl Drop for NoTree {
    fn drop(&mut self) {
        self.0.dropped.set(true);
    }
}

pub struct NoActions;

impl ActionHandler for NoActions {
    fn do_action(&mut self, _request: ActionRequest) {}
}

pub struct NoDeactivation;

impl DeactivationHandler for NoDeactivation {
    fn deactivate_accessibility(&mut self) {}
}

pub fn new_view() -> Retained<UIView> {
    unsafe { msg_send_id![class!(UIView), new] }
}

// KVO overrides the `class` method to hide its own subclass.
pub fn class_name(view: &UIView) -> String {
    let object: &AnyObject = view;
    object.class().name().to_owned()
}
