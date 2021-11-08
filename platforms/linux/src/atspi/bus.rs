use crate::atspi::{
    interfaces::*,
    proxies::{BusProxy, SocketProxy},
    ObjectAddress
};
use std::{
    env::var,
    os::unix::net::{SocketAddr, UnixStream},
    str::FromStr
};
use x11rb::{
    connection::Connection as _,
    protocol::xproto::{AtomEnum, ConnectionExt}
};
use zbus::{
    blocking::{ConnectionBuilder, Connection},
    Address
};

pub struct Bus(Connection);

impl Bus {
    pub fn a11y_bus() -> Option<Self> {
        Some(Bus(a11y_bus()?))
    }

    pub fn register_root<T>(&mut self, root: T)
    where T: AccessibleInterface + ApplicationInterface + Send + Sync + 'static
    {
        self.0.object_server_mut().at("/org/a11y/atspi/accessible/root", ApplicationInterfaceObject(root)).unwrap();
        SocketProxy::new(&self.0)
        .unwrap()
        .embed(ObjectAddress::root(
            self.0.unique_name().unwrap().as_ref()));
    }
}

fn spi_display_name() -> Option<String> {
    var("AT_SPI_DISPLAY").ok().or(
        match var("DISPLAY") {
            Ok(mut display_env) if display_env.len() > 0 => {
                match (display_env.rfind(':'), display_env.rfind('.')) {
                    (Some(i), Some(j)) if j > i => {
                        display_env.truncate(j);
                        Some(display_env)
                    },
                    _ => Some(display_env)
                }
            },
            _ => None
        })
}

fn a11y_bus_address_from_x11() -> Option<String> {
    let (bridge_display, screen_num) = x11rb::connect(Some(&spi_display_name()?)).ok()?;
    let root_window = &bridge_display.setup().roots[screen_num].root;
    let reply = bridge_display.intern_atom(false, b"AT_SPI_BUS").ok()?;
    let at_spi_bus = reply.reply().ok()?;
    let address = bridge_display.get_property(false, *root_window, at_spi_bus.atom, AtomEnum::STRING, 0, 1024).ok()?
        .reply().map_or(None, |data| String::from_utf8(data.value).ok());
    address
}

fn a11y_bus_address_from_dbus() -> Option<String> {
    let session_bus = Connection::session().ok()?;
    BusProxy::new(&session_bus).ok()?
        .get_address().ok()
}

fn a11y_bus() -> Option<Connection> {
    let mut address = match var("AT_SPI_BUS_ADDRESS") {
        Ok(address) if address.len() > 0 => Some(address),
        _ => None
    };
    if address.is_none() && var("WAYLAND_DISPLAY").is_err() {
        address = a11y_bus_address_from_x11();
    }
    if address.is_none() {
        address = a11y_bus_address_from_dbus();
    }
    let address = address?;
    let guid_index = address.find(',').unwrap_or(address.len());
    if address.starts_with("unix:abstract=") {
        ConnectionBuilder::unix_stream(UnixStream::connect_addr(&SocketAddr::from_abstract_namespace(address[14..guid_index].as_bytes()).ok()?).ok()?).build().ok()
    } else {
        ConnectionBuilder::address(Address::from_str(address.get(0..guid_index)?).ok()?).ok()?.build().ok()
    }
}
