// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::*,
    object_address::*,
    proxies::{BusProxy, SocketProxy},
};
use async_io::block_on;
use parking_lot::RwLock;
use std::{
    convert::TryInto,
    env::var,
    sync::Arc,
};
use x11rb::{
    connection::Connection as _,
    protocol::xproto::{AtomEnum, ConnectionExt},
};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    Address, Result,
};

pub struct Bus<'a> {
    conn: Connection,
    socket_proxy: SocketProxy<'a>,
}

impl<'a> Bus<'a> {
    pub fn a11y_bus() -> Option<Self> {
        let conn = a11y_bus()?;
        let socket_proxy = SocketProxy::new(&conn).ok()?;
        Some(Bus { conn, socket_proxy })
    }

    pub fn register_accessible_interface<T>(&mut self, object: T) -> Result<bool>
    where
        T: Accessible,
    {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, object.id().as_str());
        if self.conn.object_server().at(
            path.clone(),
            AccessibleInterface::new(self.conn.unique_name().unwrap().to_owned(), object.clone()),
        )? {
            let interfaces = object.interfaces();
            if interfaces.contains(Interface::FocusEvents) {
                self.register_focus_events_interface(&path)?;
            }
            if interfaces.contains(Interface::ObjectEvents) {
                self.register_object_events_interface(&path)?;
            }
            if interfaces.contains(Interface::WindowEvents) {
                self.register_window_events_interface(&path, object)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    pub fn register_application_interface<T>(&mut self, root: T) -> Result<bool>
    where
        T: Application,
    {
        println!("Registering on {:?}", self.conn.unique_name().unwrap());
        let path = format!(
            "{}{}",
            ACCESSIBLE_PATH_PREFIX,
            Accessible::id(&root).as_str()
        );
        let root = Arc::new(RwLock::new(root));
        let registered = self.conn.object_server().at(
            path,
            ApplicationInterface(ApplicationInterfaceWrapper(root.clone())),
        )?;
        if registered
            && self.register_accessible_interface(ApplicationInterfaceWrapper(root.clone()))?
        {
            let desktop_address = self.socket_proxy.embed(ObjectAddress::root(
                self.conn.unique_name().unwrap().as_ref(),
            ))?;
            root.write().set_desktop(desktop_address);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn register_focus_events_interface(&mut self, path: &str) -> Result<bool> {
        self.conn
            .object_server()
            .at(path, FocusEventsInterface {})
    }

    fn register_object_events_interface(&mut self, path: &str) -> Result<bool> {
        self.conn
            .object_server()
            .at(path, ObjectEventsInterface {})
    }

    fn register_window_events_interface<T>(&mut self, path: &str, object: T) -> Result<bool>
    where
        T: Accessible,
    {
        self.conn
            .object_server()
            .at(path, WindowEventsInterface(object))
    }

    pub fn emit_focus_event<T>(&self, target: &T) -> Result<()>
    where
        T: Accessible,
    {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, target.id().as_str());
        let iface_ref = self.conn.object_server()
            .interface::<_, FocusEventsInterface>(path)
            .unwrap();
        let iface = iface_ref.get();
        block_on(iface.focused(iface_ref.signal_context()))
    }

    pub fn emit_object_event<T>(&self, target: &T, event: ObjectEvent) -> Result<()>
    where
        T: Accessible,
    {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, target.id().as_str());
        let iface_ref = self.conn.object_server()
            .interface::<_, ObjectEventsInterface>(path)
            .unwrap();
        let iface = iface_ref.get();
        block_on(iface.emit(event, iface_ref.signal_context()))
    }

    pub fn emit_object_events<T>(&self, target: &T, events: Vec<ObjectEvent>) -> Result<()>
    where
        T: Accessible,
    {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, target.id().as_str());
        let iface_ref = self.conn.object_server()
            .interface::<_, ObjectEventsInterface>(path)
            .unwrap();
        block_on(async {
            for event in events {
                iface_ref.get().emit(event, iface_ref.signal_context()).await?;
            }
            Ok(())
        })
    }

    pub fn emit_window_event<T>(&self, target: &T, event: WindowEvent) -> Result<()>
    where
        T: Accessible,
    {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, target.id().as_str());
        let iface_ref = self.conn.object_server()
            .interface::<_, WindowEventsInterface<T>>(path)
            .unwrap();
        let iface = iface_ref.get();
        block_on(iface.emit(event, iface_ref.signal_context()))
    }
}

impl Drop for Bus<'_> {
    fn drop(&mut self) {
        let _ = self.socket_proxy.unembed(ObjectAddress::root(
            self.conn.unique_name().unwrap().as_ref(),
        ));
    }
}

fn spi_display_name() -> Option<String> {
    var("AT_SPI_DISPLAY").ok().or(match var("DISPLAY") {
        Ok(mut display_env) if display_env.len() > 0 => {
            match (display_env.rfind(':'), display_env.rfind('.')) {
                (Some(i), Some(j)) if j > i => {
                    display_env.truncate(j);
                    Some(display_env)
                }
                _ => Some(display_env),
            }
        }
        _ => None,
    })
}

fn a11y_bus_address_from_x11() -> Option<String> {
    let (bridge_display, screen_num) = x11rb::connect(Some(&spi_display_name()?)).ok()?;
    let root_window = &bridge_display.setup().roots[screen_num].root;
    let reply = bridge_display.intern_atom(false, b"AT_SPI_BUS").ok()?;
    let at_spi_bus = reply.reply().ok()?;
    let address = bridge_display
        .get_property(
            false,
            *root_window,
            at_spi_bus.atom,
            AtomEnum::STRING,
            0,
            1024,
        )
        .ok()?
        .reply()
        .map_or(None, |data| String::from_utf8(data.value).ok());
    address
}

fn a11y_bus_address_from_dbus() -> Option<String> {
    let session_bus = Connection::session().ok()?;
    BusProxy::new(&session_bus).ok()?.get_address().ok()
}

fn a11y_bus() -> Option<Connection> {
    let mut address = match var("AT_SPI_BUS_ADDRESS") {
        Ok(address) if address.len() > 0 => Some(address),
        _ => None,
    };
    if address.is_none() && var("WAYLAND_DISPLAY").is_err() {
        address = a11y_bus_address_from_x11();
    }
    if address.is_none() {
        address = a11y_bus_address_from_dbus();
    }
    let address: Address = address?.as_str().try_into().ok()?;
    ConnectionBuilder::address(address)
        .ok()?
        .build()
        .ok()
}
