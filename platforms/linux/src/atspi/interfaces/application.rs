// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::AccessibleInterface,
    ObjectId, ObjectRef, OwnedObjectAddress, Role
};
use parking_lot::RwLock;
use std::sync::Arc;

pub trait ApplicationInterface {
    fn name(&self) -> String;

    fn child_count(&self) -> usize;

    fn child_at_index(&self, index: usize) -> Option<ObjectRef>;

    fn children(&self) -> Vec<ObjectRef>;

    fn toolkit_name(&self) -> String;

    fn toolkit_version(&self) -> String;

    fn id(&self) -> Option<i32>;

    fn set_id(&mut self, id: i32);

    fn locale(&self, lctype: u32) -> String;

    fn desktop(&self) -> Option<OwnedObjectAddress>;

    fn set_desktop(&mut self, address: OwnedObjectAddress);

    fn register_event_listener(&mut self, event: String);

    fn deregister_event_listener(&mut self, event: String);
}

impl<T> AccessibleInterface for T
where T: ApplicationInterface
{
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        String::new()
    }

    fn parent(&self) -> Option<ObjectRef> {
        self.desktop().map(|desktop| desktop.into())
    }

    fn child_count(&self) -> usize {
        self.child_count()
    }

    fn locale(&self) -> String {
        String::new()
    }

    fn id(&self) -> ObjectId<'static> {
        ObjectId::root()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        self.child_at_index(index)
    }

    fn children(&self) -> Vec<ObjectRef> {
        self.children()
    }

    fn index_in_parent(&self) -> Option<usize> {
        None
    }

    fn role(&self) -> Role {
        Role::Application
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationObjectWrapper<T>(pub Arc<RwLock<T>>);

impl<T> ApplicationInterface for ApplicationObjectWrapper<T>
where T: ApplicationInterface
{
    fn name(&self) -> String {
        self.0.read().name()
    }

    fn child_count(&self) -> usize {
        self.0.read().child_count()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        self.0.read().child_at_index(index)
    }

    fn children(&self) -> Vec<ObjectRef> {
        self.0.read().children()
    }

    fn toolkit_name(&self) -> String {
        self.0.read().toolkit_name()
    }

    fn toolkit_version(&self) -> String {
        self.0.read().toolkit_version()
    }

    fn id(&self) -> Option<i32> {
        self.0.read().id()
    }

    fn set_id(&mut self, id: i32) {
        self.0.write().set_id(id);
    }

    fn locale(&self, lctype: u32) -> String {
        self.0.read().locale(lctype)
    }

    fn desktop(&self) -> Option<OwnedObjectAddress> {
        self.0.write().desktop()
    }

    fn set_desktop(&mut self, address: OwnedObjectAddress) {
        self.0.write().set_desktop(address)
    }

    fn register_event_listener(&mut self, event: String) {
        self.0.write().register_event_listener(event)
    }

    fn deregister_event_listener(&mut self, event: String) {
        self.0.write().deregister_event_listener(event)
    }
}

pub struct ApplicationInterfaceObject<T>(pub T);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl<T> ApplicationInterfaceObject<T>
where T: ApplicationInterface + Send + Sync + 'static
{
    #[dbus_interface(property)]
    fn toolkit_name(&self) -> String {
        self.0.toolkit_name()
    }

    #[dbus_interface(property)]
    fn version(&self) -> String {
        self.0.toolkit_version()
    }

    #[dbus_interface(property)]
    fn atspi_version(&self) -> &str {
        "2.1"
    }

    #[dbus_interface(property)]
    fn id(&self) -> i32 {
        self.0.id().unwrap_or(-1)
    }

    #[dbus_interface(property)]
    fn set_id(&mut self, id: i32) {
        self.0.set_id(id)
    }

    fn get_locale(&self, lctype: u32) -> String {
        self.0.locale(lctype)
    }

    fn register_event_listener(&self, _event: String) {}

    fn deregister_event_listener(&self, _event: String) {}
}
