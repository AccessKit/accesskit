// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::PlatformNode;

pub(crate) struct ValueInterface {
    node: PlatformNode,
}

impl ValueInterface {
    pub fn new(node: PlatformNode) -> Self {
        Self { node }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Value")]
impl ValueInterface {
    #[dbus_interface(property)]
    fn minimum_value(&self) -> f64 {
        self.node.minimum_value().unwrap()
    }

    #[dbus_interface(property)]
    fn maximum_value(&self) -> f64 {
        self.node.maximum_value().unwrap()
    }

    #[dbus_interface(property)]
    fn minimum_increment(&self) -> f64 {
        self.node.minimum_increment().unwrap()
    }

    #[dbus_interface(property)]
    fn current_value(&self) -> f64 {
        self.node.current_value().unwrap()
    }

    #[dbus_interface(property)]
    fn set_current_value(&self, value: f64) {
        self.node.set_current_value(value).unwrap();
    }
}
