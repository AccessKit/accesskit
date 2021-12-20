// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::{Interface, Interfaces},
    ObjectAddress, ObjectId, ObjectRef, OwnedObjectAddress, Role, StateSet,
};
use std::convert::TryInto;
use zbus::names::OwnedUniqueName;

pub trait Accessible: Clone + Send + Sync + 'static {
    fn name(&self) -> String;

    fn description(&self) -> String;

    fn parent(&self) -> Option<ObjectRef>;

    fn child_count(&self) -> usize;

    fn locale(&self) -> String;

    fn id(&self) -> ObjectId;

    fn child_at_index(&self, index: usize) -> Option<ObjectRef>;

    fn children(&self) -> Vec<ObjectRef>;

    fn index_in_parent(&self) -> Option<usize>;

    fn role(&self) -> Role;

    fn state(&self) -> StateSet;

    fn interfaces(&self) -> Interfaces;
}

pub struct AccessibleInterface<T> {
    bus_name: OwnedUniqueName,
    object: T,
}

impl<T> AccessibleInterface<T> {
    pub fn new(bus_name: OwnedUniqueName, object: T) -> Self {
        Self { bus_name, object }
    }
}

const INTERFACES: &[&'static str] = &["org.a11y.atspi.Accessible", "org.a11y.atspi.Application"];

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl<T: Accessible> AccessibleInterface<T> {
    #[dbus_interface(property)]
    fn name(&self) -> String {
        self.object.name()
    }

    #[dbus_interface(property)]
    fn description(&self) -> String {
        self.object.description()
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        match self.object.parent() {
            Some(ObjectRef::Managed(id)) => {
                ObjectAddress::accessible(self.bus_name.as_ref(), id).into()
            }
            Some(ObjectRef::Unmanaged(address)) => address,
            None => ObjectAddress::null(self.bus_name.as_ref()).into(),
        }
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> i32 {
        self.object.child_count().try_into().unwrap_or(0)
    }

    #[dbus_interface(property)]
    fn locale(&self) -> String {
        self.object.locale()
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        self.object.id()
    }

    fn get_child_at_index(&self, index: i32) -> (OwnedObjectAddress,) {
        (
            match index
                .try_into()
                .ok()
                .map(|index| self.object.child_at_index(index))
                .flatten()
            {
                Some(ObjectRef::Managed(id)) => {
                    ObjectAddress::accessible(self.bus_name.as_ref(), id).into()
                }
                Some(ObjectRef::Unmanaged(address)) => address,
                None => ObjectAddress::null(self.bus_name.as_ref()).into(),
            },
        )
    }

    fn get_children(&self) -> Vec<OwnedObjectAddress> {
        self.object
            .children()
            .into_iter()
            .map(|child| match child {
                ObjectRef::Managed(id) => {
                    ObjectAddress::accessible(self.bus_name.as_ref(), id).into()
                }
                ObjectRef::Unmanaged(address) => address,
            })
            .collect()
    }

    fn get_index_in_parent(&self) -> i32 {
        self.object
            .index_in_parent()
            .map(|index| index.try_into().ok())
            .flatten()
            .unwrap_or(-1)
    }

    fn get_role(&self) -> Role {
        self.object.role()
    }

    fn get_state(&self) -> StateSet {
        self.object.state()
    }

    fn get_interfaces(&self) -> Vec<&'static str> {
        let mut interfaces = Vec::with_capacity(INTERFACES.len());
        for interface in self.object.interfaces().iter() {
            if interface > Interface::Application {
                break;
            }
            interfaces.push(INTERFACES[(interface as u8).trailing_zeros() as usize]);
        }
        interfaces
    }
}
