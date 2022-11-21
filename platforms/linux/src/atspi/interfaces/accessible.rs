// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectAddress, ObjectId, ObjectRef, OwnedObjectAddress};
use crate::{PlatformNode, PlatformRootNode};
use atspi::{accessible::Role, Interface, InterfaceSet, StateSet};
use std::convert::TryInto;
use zbus::{fdo, names::OwnedUniqueName};

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
            Ok(ObjectRef::Managed(id)) => {
                ObjectAddress::accessible(self.bus_name.as_ref(), &id).into()
            }
            Ok(ObjectRef::Unmanaged(address)) => address,
            _ => OwnedObjectAddress::null(self.bus_name.clone()),
        }
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        self.node.child_count().unwrap_or(0)
    }

    #[dbus_interface(property)]
    fn locale(&self) -> String {
        self.node.locale().unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        self.node
            .accessible_id()
            .unwrap_or_else(|_| unsafe { ObjectId::from_str_unchecked("") })
    }

    fn get_child_at_index(&self, index: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        let index = index
            .try_into()
            .map_err(|_| fdo::Error::InvalidArgs("Index can't be negative.".into()))?;
        Ok(match self.node.child_at_index(index)? {
            ObjectRef::Managed(id) => {
                (ObjectAddress::accessible(self.bus_name.as_ref(), &id).into(),)
            }
            ObjectRef::Unmanaged(address) => (address,),
        })
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        Ok(self
            .node
            .children()?
            .into_iter()
            .map(|child| match child {
                ObjectRef::Managed(id) => {
                    ObjectAddress::accessible(self.bus_name.as_ref(), &id).into()
                }
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

    fn get_application(&self) -> (OwnedObjectAddress,) {
        (ObjectAddress::root(self.bus_name.as_ref()).into(),)
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
            .state
            .upgrade()
            .map(|state| state.read().name.clone())
            .unwrap_or_default()
    }

    #[dbus_interface(property)]
    fn description(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        self.node
            .state
            .upgrade()
            .and_then(|state| state.read().desktop_address.clone())
            .unwrap_or_else(|| OwnedObjectAddress::null(self.bus_name.clone()))
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
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

    fn get_child_at_index(&self, index: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        if index != 0 {
            return Ok((OwnedObjectAddress::null(self.bus_name.clone()),));
        }
        self.node
            .tree
            .upgrade()
            .map(|tree| {
                (
                    ObjectAddress::accessible(
                        self.bus_name.as_ref(),
                        &tree.read().root().id().into(),
                    )
                    .into(),
                )
            })
            .ok_or_else(|| fdo::Error::UnknownObject("".into()))
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        self.node
            .tree
            .upgrade()
            .map(|tree| {
                vec![ObjectAddress::accessible(
                    self.bus_name.as_ref(),
                    &tree.read().root().id().into(),
                )
                .into()]
            })
            .ok_or_else(|| fdo::Error::UnknownObject("".into()))
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

    fn get_application(&self) -> (OwnedObjectAddress,) {
        (ObjectAddress::root(self.bus_name.as_ref()).into(),)
    }

    fn get_interfaces(&self) -> InterfaceSet {
        InterfaceSet::new(Interface::Accessible | Interface::Application)
    }
}
