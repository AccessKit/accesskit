// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{atspi::ObjectId, unknown_object, PlatformRootNode};
use zbus::fdo;

pub(crate) struct ApplicationInterface(pub PlatformRootNode);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl ApplicationInterface {
    #[dbus_interface(property)]
    fn toolkit_name(&self) -> String {
        self.0
            .context
            .upgrade()
            .map(|context| context.read_app_context().toolkit_name.clone())
            .unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn version(&self) -> String {
        self.0
            .context
            .upgrade()
            .map(|context| context.read_app_context().toolkit_version.clone())
            .unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn atspi_version(&self) -> &str {
        "2.1"
    }

    #[dbus_interface(property)]
    fn id(&self) -> i32 {
        self.0
            .context
            .upgrade()
            .and_then(|context| context.read_app_context().id)
            .unwrap_or(-1)
    }

    #[dbus_interface(property)]
    fn set_id(&mut self, id: i32) -> fdo::Result<()> {
        self.0
            .context
            .upgrade()
            .map(|context| context.app_context.write().unwrap().id = Some(id))
            .ok_or_else(|| unknown_object(&ObjectId::root()))
    }
}
