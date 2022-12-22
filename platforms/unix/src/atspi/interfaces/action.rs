// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::PlatformNode;
use serde::{Deserialize, Serialize};
use zbus::{dbus_interface, fdo, zvariant::Type};

#[derive(Deserialize, Serialize, Type)]
pub(crate) struct Action {
    pub localized_name: String,
    pub description: String,
    pub key_binding: String,
}

pub(crate) struct ActionInterface(PlatformNode);

impl ActionInterface {
    pub fn new(node: PlatformNode) -> Self {
        Self(node)
    }
}

#[dbus_interface(name = "org.a11y.atspi.Action")]
impl ActionInterface {
    #[dbus_interface(property)]
    fn n_actions(&self) -> i32 {
        self.0.n_actions().unwrap_or(0)
    }

    fn get_description(&self, _index: i32) -> &str {
        ""
    }

    fn get_name(&self, index: i32) -> fdo::Result<String> {
        self.0.get_action_name(index)
    }

    fn get_localized_name(&self, index: i32) -> fdo::Result<String> {
        self.0.get_action_name(index)
    }

    fn get_key_binding(&self, _index: i32) -> &str {
        ""
    }

    fn get_actions(&self) -> fdo::Result<Vec<Action>> {
        self.0.get_actions()
    }

    fn do_action(&self, index: i32) -> fdo::Result<bool> {
        self.0.do_action(index)
    }
}
