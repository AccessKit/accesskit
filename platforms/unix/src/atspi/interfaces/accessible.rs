// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{ObjectId, ObjectRef, OwnedObjectAddress},
    unknown_object, PlatformNode, PlatformRootNode,
};
use atspi::{accessible::Role, Interface, InterfaceSet, StateSet};
use std::convert::TryInto;
use zbus::{fdo, names::OwnedUniqueName, MessageHeader};

pub(crate) struct AccessibleInterface<T> {
    bus_name: OwnedUniqueName,
    node: T,
}

impl<T> AccessibleInterface<T> {
    pub fn new(bus_name: OwnedUniqueName, node: T) -> Self {
        Self { bus_name, node }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl AccessibleInterface<PlatformNode> {
    #[dbus_interface(property)]
    fn name(&self) -> String {
        self.node.name().unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn description(&self) -> String {
        self.node.description().unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        match self.node.parent() {
            Ok(ObjectRef::Managed(id)) => OwnedObjectAddress::accessible(self.bus_name.clone(), id),
            Ok(ObjectRef::Unmanaged(address)) => address,
            _ => OwnedObjectAddress::null(self.bus_name.clone()),
        }
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        self.node.child_count().unwrap_or(0)
    }

    #[dbus_interface(property)]
    fn locale(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        self.node.accessible_id()
    }

    fn get_child_at_index(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        index: i32,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        let index = index
            .try_into()
            .map_err(|_| fdo::Error::InvalidArgs("Index can't be negative.".into()))?;
        super::object_address(hdr.destination()?, self.node.child_at_index(index)?)
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        Ok(self
            .node
            .children()?
            .into_iter()
            .map(|child| match child {
                ObjectRef::Managed(id) => OwnedObjectAddress::accessible(self.bus_name.clone(), id),
                ObjectRef::Unmanaged(address) => address,
            })
            .collect())
    }

    fn get_index_in_parent(&self) -> fdo::Result<i32> {
        self.node.index_in_parent()
    }

    fn get_role(&self) -> fdo::Result<Role> {
        self.node.role()
    }

    fn get_state(&self) -> fdo::Result<StateSet> {
        self.node.state()
    }

    fn get_application(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        super::object_address(
            hdr.destination()?,
            Some(ObjectRef::Managed(ObjectId::root())),
        )
    }

    fn get_interfaces(&self) -> fdo::Result<InterfaceSet> {
        self.node.interfaces()
    }
}

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl AccessibleInterface<PlatformRootNode> {
    #[dbus_interface(property)]
    fn name(&self) -> String {
        self.node
            .context
            .upgrade()
            .map(|context| context.read_app_context().name.clone())
            .unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn description(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        self.node
            .context
            .upgrade()
            .and_then(|context| context.read_app_context().desktop_address.clone())
            .unwrap_or_else(|| OwnedObjectAddress::null(self.bus_name.clone()))
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        // TODO: Handle multiple top-level windows.
        1
    }

    #[dbus_interface(property)]
    fn locale(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        ObjectId::root()
    }

    fn get_child_at_index(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        index: i32,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        // TODO: Handle multiple top-level windows.
        if index != 0 {
            return super::object_address(hdr.destination()?, None);
        }
        let child = self
            .node
            .context
            .upgrade()
            .map(|context| ObjectRef::Managed(context.read_tree().state().root().id().into()));
        super::object_address(hdr.destination()?, child)
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        // TODO: Handle multiple top-level windows.
        self.node
            .context
            .upgrade()
            .map(|context| {
                vec![OwnedObjectAddress::accessible(
                    self.bus_name.clone(),
                    context.read_tree().state().root().id().into(),
                )]
            })
            .ok_or_else(|| unknown_object(&ObjectId::root()))
    }

    fn get_index_in_parent(&self) -> i32 {
        -1
    }

    fn get_role(&self) -> Role {
        Role::Application
    }

    fn get_state(&self) -> StateSet {
        StateSet::empty()
    }

    fn get_application(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        super::object_address(
            hdr.destination()?,
            Some(ObjectRef::Managed(ObjectId::root())),
        )
    }

    fn get_interfaces(&self) -> InterfaceSet {
        InterfaceSet::new(Interface::Accessible | Interface::Application)
    }
}
