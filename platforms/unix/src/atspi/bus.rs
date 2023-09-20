// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{interfaces::*, ObjectId},
    context::AppContext,
    PlatformRootNode,
};
use async_once_cell::OnceCell;
use atspi::{
    events::EventBody,
    proxy::{bus::BusProxy, socket::SocketProxy},
};
use serde::Serialize;
use std::{
    collections::HashMap,
    env::var,
    sync::{Arc, RwLock},
};
use zbus::{
    names::{BusName, InterfaceName, MemberName, OwnedUniqueName},
    zvariant::{Str, Value},
    Address, Connection, ConnectionBuilder, Result,
};

#[derive(Clone)]
pub(crate) struct Bus {
    conn: Connection,
    socket_proxy: SocketProxy<'static>,
}

impl Bus {
    pub async fn a11y_bus() -> Option<Self> {
        let conn = a11y_bus().await?;
        let socket_proxy = SocketProxy::new(&conn).await.ok()?;
        Some(Bus { conn, socket_proxy })
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn unique_name(&self) -> &OwnedUniqueName {
        self.conn.unique_name().unwrap()
    }

    pub async fn register_interface<T>(&self, path: &str, interface: T) -> Result<bool>
    where
        T: zbus::Interface,
    {
        self.conn.object_server().at(path, interface).await
    }

    pub async fn unregister_interface<T>(&self, path: &str) -> Result<bool>
    where
        T: zbus::Interface,
    {
        self.conn.object_server().remove::<T, _>(path).await
    }

    pub async fn register_root_node(
        &mut self,
        app_context: &Arc<RwLock<AppContext>>,
    ) -> Result<bool> {
        let node = PlatformRootNode::new(app_context);
        let path = ObjectId::Root.path();

        let app_node_added = self
            .conn
            .object_server()
            .at(path.clone(), ApplicationInterface(node.clone()))
            .await?
            && self
                .conn
                .object_server()
                .at(
                    path,
                    AccessibleInterface::new(self.unique_name().to_owned(), node),
                )
                .await?;

        if app_node_added {
            let desktop = self
                .socket_proxy
                .embed(&(self.unique_name().as_str(), ObjectId::Root.path().into()))
                .await?;
            app_context.write().unwrap().desktop_address = Some(desktop.into());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) async fn emit_object_event(
        &self,
        target: ObjectId,
        event: ObjectEvent,
    ) -> Result<()> {
        let interface = "org.a11y.atspi.Event.Object";
        let signal = match event {
            ObjectEvent::ActiveDescendantChanged(_) => "ActiveDescendantChanged",
            ObjectEvent::Announcement(_, _) => "Announcement",
            ObjectEvent::BoundsChanged(_) => "BoundsChanged",
            ObjectEvent::ChildAdded(_, _) | ObjectEvent::ChildRemoved(_) => "ChildrenChanged",
            ObjectEvent::PropertyChanged(_) => "PropertyChange",
            ObjectEvent::StateChanged(_, _) => "StateChanged",
        };
        let properties = HashMap::new();
        match event {
            ObjectEvent::ActiveDescendantChanged(child) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "",
                        detail1: 0,
                        detail2: 0,
                        any_data: child.to_address(self.unique_name().clone()).into(),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::Announcement(message, politeness) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "",
                        detail1: politeness as i32,
                        detail2: 0,
                        any_data: message.into(),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::BoundsChanged(bounds) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "",
                        detail1: 0,
                        detail2: 0,
                        any_data: Value::from(bounds),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::ChildAdded(index, child) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "add",
                        detail1: index as i32,
                        detail2: 0,
                        any_data: child.to_address(self.unique_name().clone()).into(),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::ChildRemoved(child) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: "remove",
                        detail1: -1,
                        detail2: 0,
                        any_data: child.to_address(self.unique_name().clone()).into(),
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::PropertyChanged(property) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: match property {
                            Property::Name(_) => "accessible-name",
                            Property::Description(_) => "accessible-description",
                            Property::Parent(_) => "accessible-parent",
                            Property::Role(_) => "accessible-role",
                            Property::Value(_) => "accessible-value",
                        },
                        detail1: 0,
                        detail2: 0,
                        any_data: match property {
                            Property::Name(value) => Str::from(value).into(),
                            Property::Description(value) => Str::from(value).into(),
                            Property::Parent(parent) => {
                                parent.to_address(self.unique_name().clone()).into()
                            }
                            Property::Role(value) => Value::U32(value as u32),
                            Property::Value(value) => Value::F64(value),
                        },
                        properties,
                    },
                )
                .await
            }
            ObjectEvent::StateChanged(state, value) => {
                self.emit_event(
                    target,
                    interface,
                    signal,
                    EventBody {
                        kind: state,
                        detail1: value as i32,
                        detail2: 0,
                        any_data: 0i32.into(),
                        properties,
                    },
                )
                .await
            }
        }
    }

    pub(crate) async fn emit_window_event(
        &self,
        target: ObjectId,
        window_name: String,
        event: WindowEvent,
    ) -> Result<()> {
        let signal = match event {
            WindowEvent::Activated => "Activate",
            WindowEvent::Deactivated => "Deactivate",
        };
        self.emit_event(
            target,
            "org.a11y.atspi.Event.Window",
            signal,
            EventBody {
                kind: "",
                detail1: 0,
                detail2: 0,
                any_data: window_name.into(),
                properties: HashMap::new(),
            },
        )
        .await
    }

    async fn emit_event<T: Serialize>(
        &self,
        target: ObjectId,
        interface: &str,
        signal_name: &str,
        body: EventBody<'_, T>,
    ) -> Result<()> {
        self.conn
            .emit_signal(
                Option::<BusName>::None,
                target.path(),
                InterfaceName::from_str_unchecked(interface),
                MemberName::from_str_unchecked(signal_name),
                &body,
            )
            .await
    }
}

static A11Y_BUS: OnceCell<Option<Connection>> = OnceCell::new();

async fn a11y_bus() -> Option<Connection> {
    A11Y_BUS
        .get_or_init(async {
            let address = match var("AT_SPI_BUS_ADDRESS") {
                Ok(address) if !address.is_empty() => address,
                _ => {
                    let session_bus = Connection::session().await.ok()?;
                    BusProxy::new(&session_bus)
                        .await
                        .ok()?
                        .get_address()
                        .await
                        .ok()?
                }
            };
            let address: Address = address.as_str().try_into().ok()?;
            ConnectionBuilder::address(address).ok()?.build().await.ok()
        })
        .await
        .clone()
}
