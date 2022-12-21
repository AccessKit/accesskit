// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{interfaces::*, object_address::*, ObjectId, ObjectRef};
use crate::PlatformRootNode;
use atspi::{bus::BusProxyBlocking, socket::SocketProxyBlocking, EventBody};
use serde::Serialize;
use std::{
    collections::HashMap,
    convert::{AsRef, TryInto},
    env::var,
};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    names::{BusName, InterfaceName, MemberName, OwnedUniqueName},
    zvariant::{ObjectPath, Str, Value},
    Address, Result,
};

#[derive(Clone)]
pub(crate) struct Bus {
    conn: Connection,
    socket_proxy: SocketProxyBlocking<'static>,
}

impl Bus {
    pub fn a11y_bus() -> Option<Self> {
        let conn = a11y_bus()?;
        let socket_proxy = SocketProxyBlocking::new(&conn).ok()?;
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
            let desktop = self.socket_proxy.embed(&(
                self.unique_name().as_str(),
                ObjectPath::from_str_unchecked(ROOT_PATH),
            ))?;
            if let Some(context) = node.context.upgrade() {
                context.write().desktop_address = Some(desktop.into());
            }
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
                EventBody {
                    kind: state,
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
                EventBody {
                    kind: match property {
                        Property::Name(_) => "accessible-name",
                        Property::Description(_) => "accessible-description",
                        Property::Parent(_) => "accessible-parent",
                        Property::Role(_) => "accessible-role",
                    },
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
                        Property::Role(value) => Value::U32(*value as u32),
                    },
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
            EventBody {
                kind: "",
                detail1: 0,
                detail2: 0,
                any_data: window_name.into(),
                properties: HashMap::new(),
            },
        )
    }

    fn emit_event<T: Serialize>(
        &self,
        id: &ObjectId,
        interface: &str,
        signal_name: &str,
        body: EventBody<T>,
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

fn a11y_bus() -> Option<Connection> {
    let address = match var("AT_SPI_BUS_ADDRESS") {
        Ok(address) if !address.is_empty() => address,
        _ => {
            let session_bus = Connection::session().ok()?;
            BusProxyBlocking::new(&session_bus)
                .ok()?
                .get_address()
                .ok()?
        }
    };
    let address: Address = address.as_str().try_into().ok()?;
    ConnectionBuilder::address(address).ok()?.build().ok()
}
