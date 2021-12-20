// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectAddress, OwnedObjectAddress};
use zbus::{dbus_proxy, Result};

#[dbus_proxy(
    default_service = "org.a11y.Bus",
    default_path = "/org/a11y/bus",
    interface = "org.a11y.Bus",
    gen_async = false
)]
pub trait Bus {
    fn get_address(&self) -> Result<String>;
}

#[dbus_proxy(
    default_path = "/org/a11y/atspi/accessible/root",
    default_service = "org.a11y.atspi.Registry",
    gen_async = false,
    interface = "org.a11y.atspi.Socket"
)]
trait Socket {
    fn embed<'a>(&self, plug: ObjectAddress<'a>) -> Result<OwnedObjectAddress>;

    fn unembed<'a>(&self, plug: ObjectAddress<'a>) -> Result<()>;

    #[dbus_proxy(signal)]
    fn available(&self, socket: ObjectAddress<'_>) -> Result<()>;
}

#[dbus_proxy(interface = "org.a11y.Status")]
pub trait Status {
    #[dbus_proxy(property)]
    fn is_enabled(&self) -> Result<bool>;

    #[DBusProxy(property)]
    fn set_is_enabled(&self, value: bool) -> Result<()>;

    #[dbus_proxy(property)]
    fn screen_reader_enabled(&self) -> Result<bool>;

    #[DBusProxy(property)]
    fn set_screen_reader_enabled(&self, value: bool) -> Result<()>;
}
