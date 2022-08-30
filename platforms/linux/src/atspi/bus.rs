// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::*,
    object_address::*,
    proxies::{BusProxy, SocketProxy},
    ObjectId, ObjectRef,
};
use crate::PlatformRootNode;
use std::{
    collections::HashMap,
    convert::{AsRef, TryInto},
    env::var,
};
use x11rb::{
    connection::Connection as _,
    protocol::xproto::{AtomEnum, ConnectionExt},
};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    names::{BusName, InterfaceName, MemberName, OwnedUniqueName},
    Address, Result,
};
use zvariant::{OwnedValue, Str, Value};

#[derive(Clone)]
pub(crate) struct Bus<'a> {
    conn: Connection,
    socket_proxy: SocketProxy<'a>,
}

impl<'a> Bus<'a> {
    pub fn a11y_bus() -> Option<Self> {
        let conn = a11y_bus()?;
        let socket_proxy = SocketProxy::new(&conn).ok()?;
        Some(Bus { conn, socket_proxy })
    }

    pub fn unique_name(&self) -> &OwnedUniqueName {
        self.conn.unique_name().unwrap()
    }

    pub fn register_interface<T>(&self, path: &str, interface: T) -> Result<bool>
    where
        T: zbus::Interface,
    {
        self.conn.object_server().at(path, interface)
    }

    pub fn unregister_interface<T>(&self, path: &str) -> Result<bool>
    where
        T: zbus::Interface,
    {
        self.conn.object_server().remove::<T, _>(path)
    }

    pub fn register_root_node(&mut self, node: PlatformRootNode) -> Result<bool> {
        println!("Registering on {:?}", self.unique_name());
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, ObjectId::root().as_str());
        let registered = self
            .conn
            .object_server()
            .at(path.clone(), ApplicationInterface(node.clone()))?
            && self.conn.object_server().at(
                path,
                AccessibleInterface::new(self.unique_name().to_owned(), node.clone()),
            )?;
        if registered {
            let desktop_address = self
                .socket_proxy
                .embed(ObjectAddress::root(self.unique_name().as_ref()))?;
            node.state
                .upgrade()
                .map(|state| state.write().desktop_address = Some(desktop_address));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn emit_object_event(&self, target: &ObjectId, event: &ObjectEvent) -> Result<()> {
        let interface = "org.a11y.atspi.Event.Object";
        let signal = event.as_ref();
        let properties = HashMap::new();
        match event {
            ObjectEvent::StateChanged(state, value) => self.emit_event(
                target,
                interface,
                signal,
                EventData {
                    minor: state.as_ref(),
                    detail1: *value as i32,
                    detail2: 0,
                    any_data: 0i32.into(),
                    properties,
                },
            ),
            ObjectEvent::PropertyChanged(property) => self.emit_event(
                target,
                interface,
                signal,
                EventData {
                    minor: property.as_ref(),
                    detail1: 0,
                    detail2: 0,
                    any_data: match property {
                        Property::Name(value) => Str::from(value).into(),
                        Property::Description(value) => Str::from(value).into(),
                        Property::Parent(Some(ObjectRef::Managed(parent))) => {
                            OwnedObjectAddress::from(ObjectAddress::accessible(
                                self.unique_name().into(),
                                parent,
                            ))
                            .into()
                        }
                        Property::Parent(Some(ObjectRef::Unmanaged(parent))) => {
                            parent.clone().into()
                        }
                        Property::Parent(None) => {
                            OwnedObjectAddress::from(ObjectAddress::root(self.unique_name().into()))
                                .into()
                        }
                        Property::Role(value) => OwnedValue::from(*value as u32),
                    }
                    .into(),
                    properties,
                },
            ),
        }
    }

    pub fn emit_window_event(
        &self,
        target: &ObjectId,
        window_name: &str,
        event: &WindowEvent,
    ) -> Result<()> {
        self.emit_event(
            target,
            "org.a11y.atspi.Event.Window",
            event.as_ref(),
            EventData {
                minor: "",
                detail1: 0,
                detail2: 0,
                any_data: Value::from(window_name).into(),
                properties: HashMap::new(),
            },
        )
    }

    fn emit_event(
        &self,
        id: &ObjectId,
        interface: &str,
        signal_name: &str,
        body: EventData,
    ) -> Result<()> {
        let path = format!("{}{}", ACCESSIBLE_PATH_PREFIX, id.as_str());
        self.conn.emit_signal(
            Option::<BusName>::None,
            path,
            InterfaceName::from_str_unchecked(interface),
            MemberName::from_str_unchecked(signal_name),
            &body,
        )
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
    ConnectionBuilder::address(address).ok()?.build().ok()
}
