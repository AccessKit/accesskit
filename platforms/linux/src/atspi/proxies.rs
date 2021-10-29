use crate::atspi::{ObjectAddress, OwnedObjectAddress};
use zbus::{Result, dbus_proxy};

#[dbus_proxy(
    default_service = "org.a11y.Bus",
    default_path = "/org/a11y/bus",
    interface = "org.a11y.Bus",
    gen_async = false
)]
pub trait Bus {
    fn get_address(&self) -> Result<String>;
}

#[dbus_proxy(interface = "org.a11y.atspi.Socket")]
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
