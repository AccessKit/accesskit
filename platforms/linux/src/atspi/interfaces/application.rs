// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::{Accessible, Interface, Interfaces},
    ObjectId, ObjectRef, OwnedObjectAddress, Role, StateSet
};
use parking_lot::RwLock;
use std::sync::Arc;

pub trait Application: Accessible {
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

impl<T: Application> Accessible for T {
    fn name(&self) -> String {
        Application::name(self)
    }

    fn description(&self) -> String {
        String::new()
    }

    fn parent(&self) -> Option<ObjectRef> {
        self.desktop().map(|desktop| desktop.into())
    }

    fn child_count(&self) -> usize {
        Application::child_count(self)
    }

    fn locale(&self) -> String {
        String::new()
    }

    fn id(&self) -> ObjectId<'static> {
        ObjectId::root()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        Application::child_at_index(self, index)
    }

    fn children(&self) -> Vec<ObjectRef> {
        Application::children(self)
    }

    fn index_in_parent(&self) -> Option<usize> {
        None
    }

    fn role(&self) -> Role {
        Role::Application
    }

    fn state(&self) -> StateSet {
        StateSet::empty()
    }

    fn interfaces(&self) -> Interfaces {
        Interface::Accessible | Interface::Application
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationInterfaceWrapper<T>(pub Arc<RwLock<T>>);

impl<T: Application> Application for ApplicationInterfaceWrapper<T> {
    fn name(&self) -> String {
        Application::name(&*self.0.read())
    }

    fn child_count(&self) -> usize {
        Application::child_count(&*self.0.read())
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        Application::child_at_index(&*self.0.read(), index)
    }

    fn children(&self) -> Vec<ObjectRef> {
        Application::children(&*self.0.read())
    }

    fn toolkit_name(&self) -> String {
        self.0.read().toolkit_name()
    }

    fn toolkit_version(&self) -> String {
        self.0.read().toolkit_version()
    }

    fn id(&self) -> Option<i32> {
        Application::id(&*self.0.read())
    }

    fn set_id(&mut self, id: i32) {
        self.0.write().set_id(id);
    }

    fn locale(&self, lctype: u32) -> String {
        Application::locale(&*self.0.read(), lctype)
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

pub struct ApplicationInterface<T>(pub T);

#[dbus_interface(name = "org.a11y.atspi.Application")]
impl<T: Application> ApplicationInterface<T> {
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
        Application::id(&self.0).unwrap_or(-1)
    }

    #[dbus_interface(property)]
    fn set_id(&mut self, id: i32) {
        self.0.set_id(id)
    }

    fn get_locale(&self, lctype: u32) -> String {
        Application::locale(&self.0, lctype)
    }

    fn register_event_listener(&self, _event: String) {}

    fn deregister_event_listener(&self, _event: String) {}
}
