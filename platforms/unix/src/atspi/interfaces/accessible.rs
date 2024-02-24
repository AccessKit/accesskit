// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::{NodeIdOrRoot, PlatformNode, PlatformRoot};
use atspi::{Interface, InterfaceSet, Role, StateSet};
use zbus::{fdo, names::OwnedUniqueName};

use super::map_root_error;
use crate::atspi::{ObjectId, OwnedObjectAddress};

pub(crate) struct NodeAccessibleInterface {
    bus_name: OwnedUniqueName,
    node: PlatformNode,
}

impl NodeAccessibleInterface {
    pub fn new(bus_name: OwnedUniqueName, node: PlatformNode) -> Self {
        Self { bus_name, node }
    }

    fn map_error(&self) -> impl '_ + FnOnce(accesskit_atspi_common::Error) -> fdo::Error {
        |error| crate::util::map_error_from_node(&self.node, error)
    }
}

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl NodeAccessibleInterface {
    #[dbus_interface(property)]
    fn name(&self) -> fdo::Result<String> {
        self.node.name().map_err(self.map_error())
    }

    #[dbus_interface(property)]
    fn description(&self) -> fdo::Result<String> {
        self.node.description().map_err(self.map_error())
    }

    #[dbus_interface(property)]
    fn parent(&self) -> fdo::Result<OwnedObjectAddress> {
        self.node.parent().map_err(self.map_error()).map(|parent| {
            match parent {
                NodeIdOrRoot::Node(node) => ObjectId::Node {
                    adapter: self.node.adapter_id(),
                    node,
                },
                NodeIdOrRoot::Root => ObjectId::Root,
            }
            .to_address(self.bus_name.clone())
        })
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> fdo::Result<i32> {
        self.node.child_count().map_err(self.map_error())
    }

    #[dbus_interface(property)]
    fn locale(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        ObjectId::from(&self.node)
    }

    fn get_child_at_index(&self, index: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        let index = index
            .try_into()
            .map_err(|_| fdo::Error::InvalidArgs("Index can't be negative.".into()))?;
        let child = self
            .node
            .child_at_index(index)
            .map_err(self.map_error())?
            .map(|child| ObjectId::Node {
                adapter: self.node.adapter_id(),
                node: child,
            });
        Ok(super::optional_object_address(&self.bus_name, child))
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        self.node
            .map_children(|child| {
                ObjectId::Node {
                    adapter: self.node.adapter_id(),
                    node: child,
                }
                .to_address(self.bus_name.clone())
            })
            .map_err(self.map_error())
    }

    fn get_index_in_parent(&self) -> fdo::Result<i32> {
        self.node.index_in_parent().map_err(self.map_error())
    }

    fn get_role(&self) -> fdo::Result<Role> {
        self.node.role().map_err(self.map_error())
    }

    fn get_localized_role_name(&self) -> fdo::Result<String> {
        self.node.localized_role_name().map_err(self.map_error())
    }

    fn get_state(&self) -> StateSet {
        self.node.state()
    }

    fn get_application(&self) -> (OwnedObjectAddress,) {
        (ObjectId::Root.to_address(self.bus_name.clone()),)
    }

    fn get_interfaces(&self) -> fdo::Result<InterfaceSet> {
        self.node.interfaces().map_err(self.map_error())
    }
}

pub(crate) struct RootAccessibleInterface {
    bus_name: OwnedUniqueName,
    desktop_address: OwnedObjectAddress,
    root: PlatformRoot,
}

impl RootAccessibleInterface {
    pub fn new(
        bus_name: OwnedUniqueName,
        desktop_address: OwnedObjectAddress,
        root: PlatformRoot,
    ) -> Self {
        Self {
            bus_name,
            desktop_address,
            root,
        }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Accessible")]
impl RootAccessibleInterface {
    #[dbus_interface(property)]
    fn name(&self) -> fdo::Result<String> {
        self.root.name().map_err(map_root_error)
    }

    #[dbus_interface(property)]
    fn description(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn parent(&self) -> OwnedObjectAddress {
        self.desktop_address.clone()
    }

    #[dbus_interface(property)]
    fn child_count(&self) -> fdo::Result<i32> {
        self.root.child_count().map_err(map_root_error)
    }

    #[dbus_interface(property)]
    fn locale(&self) -> &str {
        ""
    }

    #[dbus_interface(property)]
    fn accessible_id(&self) -> ObjectId {
        ObjectId::Root
    }

    fn get_child_at_index(&self, index: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        let index = index
            .try_into()
            .map_err(|_| fdo::Error::InvalidArgs("Index can't be negative.".into()))?;
        let child = self
            .root
            .child_id_at_index(index)
            .map_err(map_root_error)?
            .map(|(adapter, node)| ObjectId::Node { adapter, node });
        Ok(super::optional_object_address(&self.bus_name, child))
    }

    fn get_children(&self) -> fdo::Result<Vec<OwnedObjectAddress>> {
        self.root
            .map_child_ids(|(adapter, node)| {
                ObjectId::Node { adapter, node }.to_address(self.bus_name.clone())
            })
            .map_err(map_root_error)
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
        (ObjectId::Root.to_address(self.bus_name.clone()),)
    }

    fn get_interfaces(&self) -> InterfaceSet {
        InterfaceSet::new(Interface::Accessible | Interface::Application)
    }
}
