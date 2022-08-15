// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::{Interface, Interfaces},
    ObjectAddress, ObjectId, ObjectRef, OwnedObjectAddress, Role, State, StateSet,
};
use crate::{PlatformNode, PlatformRootNode};
use std::convert::TryInto;
use zbus::{fdo, names::OwnedUniqueName};

pub struct AccessibleInterface<T> {
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
        self.node
            .resolve(|node| node.name())
            .unwrap_or(String::new())
    }

    #[dbus_interface(property)]
    fn description(&self) -> String {
        self.node
            .resolve(|node| node.description())
            .unwrap_or(String::new())
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        match self.node.resolve(|node| node.parent()).ok().flatten() {
            Some(ObjectRef::Managed(id)) => {
                ObjectAddress::accessible(self.bus_name.as_ref(), id).into()
            }
            Some(ObjectRef::Unmanaged(address)) => address,
            None => ObjectAddress::null(self.bus_name.as_ref()).into(),
        }
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        self.node
            .resolve(|node| node.child_count())
            .map_or(0, |count| count.try_into().unwrap_or(0))
    }

    #[dbus_interface(property)]
    fn locale(&self) -> String {
        self.node
            .resolve(|node| node.locale())
            .unwrap_or(String::new())
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        self.node
            .resolve(|node| node.id())
            .unwrap_or(unsafe { ObjectId::from_str_unchecked("") })
    }

    fn get_child_at_index(&self, index: i32) -> fdo::Result<OwnedObjectAddress> {
        let index = index
            .try_into()
            .map_err(|_| fdo::Error::InvalidArgs("Index can't be negative.".into()))?;
        self.node.resolve(|node| match node.child_at_index(index) {
            Some(ObjectRef::Managed(id)) => {
                ObjectAddress::accessible(self.bus_name.as_ref(), id).into()
            }
            Some(ObjectRef::Unmanaged(address)) => address,
            _ => ObjectAddress::null(self.bus_name.as_ref()).into(),
        })
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        self.node.resolve(|node| {
            node.children()
                .into_iter()
                .map(|child| match child {
                    ObjectRef::Managed(id) => {
                        ObjectAddress::accessible(self.bus_name.as_ref(), id).into()
                    }
                    ObjectRef::Unmanaged(address) => address,
                })
                .collect()
        })
    }

    fn get_index_in_parent(&self) -> fdo::Result<i32> {
        let index = self.node.resolve(|node| node.index_in_parent())?;
        if let Some(index) = index {
            index
                .try_into()
                .map_err(|_| fdo::Error::Failed("Index is too big.".into()))
        } else {
            Ok(-1)
        }
    }

    fn get_role(&self) -> fdo::Result<Role> {
        self.node.resolve(|node| node.role())
    }

    fn get_state(&self) -> fdo::Result<StateSet> {
        self.node.resolve(|node| node.state())
    }

    fn get_interfaces(&self) -> fdo::Result<Interfaces> {
        self.node.resolve(|node| node.interfaces())
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
            .unwrap_or(String::new())
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
            .unwrap_or_else(|| ObjectAddress::null(self.bus_name.as_ref()).into())
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        1
    }

    #[dbus_interface(property)]
    fn locale(&self) -> String {
        String::new()
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        ObjectId::root()
    }

    fn get_child_at_index(&self, index: i32) -> fdo::Result<OwnedObjectAddress> {
        if index != 0 {
            return Ok(ObjectAddress::null(self.bus_name.as_ref()).into());
        }
        self.node
            .tree
            .upgrade()
            .map(|tree| {
                ObjectAddress::accessible(self.bus_name.as_ref(), tree.read().root().id().into())
                    .into()
            })
            .ok_or(fdo::Error::UnknownObject("".into()))
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        self.node
            .tree
            .upgrade()
            .map(|tree| {
                vec![ObjectAddress::accessible(
                    self.bus_name.as_ref(),
                    tree.read().root().id().into(),
                )
                .into()]
            })
            .ok_or(fdo::Error::UnknownObject("".into()))
    }

    fn get_index_in_parent(&self) -> i32 {
        -1
    }

    fn get_role(&self) -> Role {
        Role::Application
    }

    fn get_state(&self) -> StateSet {
        let mut state = StateSet::empty();
        state.insert(State::Showing | State::Visible);
        state
    }

    fn get_interfaces(&self) -> Interfaces {
        Interfaces::new(Interface::Accessible | Interface::Application)
    }
}
