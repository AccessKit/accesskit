// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::PlatformRootNode;
use zbus::fdo;

pub struct ApplicationInterface(pub(crate) PlatformRootNode);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl ApplicationInterface {
    #[dbus_interface(property)]
    fn toolkit_name(&self) -> String {
        self.0.state.upgrade()
            .map(|state| state.read().toolkit_name.clone())
            .unwrap_or(String::new())
    }

    #[dbus_interface(property)]
    fn version(&self) -> String {
        self.0.state.upgrade()
            .map(|state| state.read().toolkit_version.clone())
            .unwrap_or(String::new())
    }

    #[dbus_interface(property)]
    fn atspi_version(&self) -> &str {
        "2.1"
    }

    #[dbus_interface(property)]
    fn id(&self) -> i32 {
        self.0.state.upgrade()
            .and_then(|state| state.read().id)
            .unwrap_or(-1)
    }

    #[dbus_interface(property)]
    fn set_id(&mut self, id: i32) -> fdo::Result<()> {
        self.0.state.upgrade()
            .map(|state| state.write().id = Some(id))
            .ok_or(fdo::Error::UnknownObject("".into()))
    }

    fn get_locale(&self, lctype: u32) -> fdo::Result<String> {
        Ok(String::new())
    }

    fn register_event_listener(&self, _event: String) {}

    fn deregister_event_listener(&self, _event: String) {}
}
