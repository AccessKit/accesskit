// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{ObjectId, OwnedObjectAddress},
    PlatformNode, PlatformRootNode,
};
use atspi::{Interface, InterfaceSet, Role, StateSet};
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
    fn name(&self) -> fdo::Result<String> {
        self.node.name()
    }

    #[dbus_interface(property)]
    fn description(&self) -> fdo::Result<String> {
        self.node.description()
    }

    #[dbus_interface(property)]
    fn parent(&self) -> fdo::Result<OwnedObjectAddress> {
        self.node
            .parent()
            .map(|parent| parent.to_address(self.bus_name.clone()))
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> fdo::Result<i32> {
        self.node.child_count()
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
            .map(|child| child.to_address(self.bus_name.clone()))
            .collect())
    }

    fn get_index_in_parent(&self) -> fdo::Result<i32> {
        self.node.index_in_parent()
    }

    fn get_role(&self) -> fdo::Result<Role> {
        self.node.role()
    }

    fn get_localized_role_name(&self) -> fdo::Result<String> {
        self.node.localized_role_name()
    }

    fn get_state(&self) -> fdo::Result<StateSet> {
        self.node.state()
    }

    fn get_application(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        super::object_address(hdr.destination()?, Some(ObjectId::Root))
    }

    fn get_interfaces(&self) -> fdo::Result<InterfaceSet> {
        self.node.interfaces()
    }
}

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl AccessibleInterface<PlatformRootNode> {
    #[dbus_interface(property)]
    fn name(&self) -> fdo::Result<String> {
        self.node.name()
    }

    #[dbus_interface(property)]
    fn description(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn parent(&self) -> fdo::Result<OwnedObjectAddress> {
        Ok(self
            .node
            .parent()?
            .unwrap_or_else(|| OwnedObjectAddress::null(self.bus_name.clone())))
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> fdo::Result<i32> {
        self.node.child_count()
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
        let child = self.node.child_at_index(index)?;
        super::object_address(hdr.destination()?, child)
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        let children = self
            .node
            .children()?
            .drain(..)
            .map(|child| child.to_address(self.bus_name.clone()))
            .collect();
        Ok(children)
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
        super::object_address(hdr.destination()?, Some(ObjectId::Root))
    }

    fn get_interfaces(&self) -> InterfaceSet {
        InterfaceSet::new(Interface::Accessible | Interface::Application)
    }
}
