// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::PlatformRootNode;
use zbus::fdo;

pub(crate) struct ApplicationInterface(pub PlatformRootNode);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl ApplicationInterface {
    #[dbus_interface(property)]
    fn toolkit_name(&self) -> String {
        self.0.toolkit_name()
    }

    #[dbus_interface(property)]
    fn version(&self) -> String {
        self.0.toolkit_version()
    }

    #[dbus_interface(property)]
    fn atspi_version(&self) -> &str {
        "2.1"
    }

    #[dbus_interface(property)]
    fn id(&self) -> i32 {
        self.0.id()
    }

    #[dbus_interface(property)]
    fn set_id(&mut self, id: i32) -> fdo::Result<()> {
        self.0.set_id(id)
    }
}
