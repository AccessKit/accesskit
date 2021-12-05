// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{Node, Tree, WeakNode};
use accesskit_schema::Role;
use crate::atspi::{
    interfaces::{Accessible, Application, Interface, Interfaces},
    ObjectId, ObjectRef, OwnedObjectAddress, Role as AtspiRole, State, StateSet
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PlatformNode(WeakNode);

impl PlatformNode {
    pub(crate) fn new(node: &Node) -> Self {
        Self(node.downgrade())
    }
}

impl Accessible for PlatformNode {
    fn name(&self) -> String {
        self.0.map(|node| node.name().map(|name| name.to_string())).flatten().unwrap_or(String::new())
    }

    fn description(&self) -> String {
        String::new()
    }

    fn parent(&self) -> Option<ObjectRef> {
        Some(self
        .0
        .map(|node| node.parent().map(|parent| parent.id().into()))
        .flatten()
        .unwrap_or(ObjectId::root().into()))
    }

    fn child_count(&self) -> usize {
        self.0.map(|node| node.children().count()).unwrap_or(0)
    }

    fn locale(&self) -> String {
        String::new()
    }

    fn id(&self) -> ObjectId<'static> {
        self.0.map(|node| node.id().into()).unwrap()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        self.0.map(|node| node.children().nth(index).map(|child| child.id().into())).flatten()
    }

    fn children(&self) -> Vec<ObjectRef> {
        self.0.map(|node| node.children().map(|child| child.id().into()).collect()).unwrap_or(Vec::new())
    }

    fn index_in_parent(&self) -> Option<usize> {
        self.0.map(|node| node.parent_and_index().map(|(_, index)| index)).flatten()
    }

    fn role(&self) -> AtspiRole {
        self.0.map(|node| {
            match node.role() {
                Role::Button => AtspiRole::PushButton,
                Role::Window => AtspiRole::Frame,
                _ => unimplemented!()
            }
        }).unwrap_or(AtspiRole::Invalid)
    }

    fn state(&self) -> StateSet {
        self.0.map(|node| {
            if node.role() == Role::Window {
                (State::Active | State::Sensitive | State::Showing | State::Visible).into()
            } else {
                let mut state: StateSet = (State::Enabled | State::Sensitive | State::Showing | State::Visible).into();
                if node.data().focusable {
                    state.insert(State::Focusable);
                    if node.is_focused() {
                        state.insert(State::Focused);
                    }
                }
                state
            }
        }).unwrap()
    }

    fn interfaces(&self) -> Interfaces {
        self.0.map(|node| {
            let mut interfaces: Interfaces = Interface::Accessible | Interface::ObjectEvents;
            if node.role() == Role::Window {
                interfaces.insert(Interface::WindowEvents);
            }
            if node.data().focusable {
                interfaces.insert(Interface::FocusEvents);
            }
            interfaces
        }).unwrap()
    }
}

#[derive(Clone)]
pub struct RootPlatformNode {
    app_name: String,
    app_id: Option<i32>,
    desktop_address: Option<OwnedObjectAddress>,
    tree: Arc<Tree>,
    toolkit_name: String,
    toolkit_version: String,
}

impl RootPlatformNode {
    pub fn new(app_name: String, toolkit_name: String, toolkit_version: String, tree: Arc<Tree>) -> Self {
        Self {
            app_name,
            app_id: None,
            desktop_address: None,
            tree,
            toolkit_name,
            toolkit_version
        }
    }
}

impl Application for RootPlatformNode {
    fn name(&self) -> String {
        self.app_name.clone()
    }

    fn child_count(&self) -> usize {
        1
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        if index == 0 {
            Some(self.tree.read().root().id().into())
        } else {
            None
        }
    }

    fn children(&self) -> Vec<ObjectRef> {
        vec![self.tree.read().root().id().into()]
    }

    fn toolkit_name(&self) -> String {
        self.toolkit_name.clone()
    }

    fn toolkit_version(&self) -> String {
        self.toolkit_version.clone()
    }

    fn id(&self) -> Option<i32> {
        self.app_id
    }

    fn set_id(&mut self, id: i32) {
        self.app_id = Some(id);
    }

    fn locale(&self, lctype: u32) -> String {
        String::new()
    }

    fn desktop(&self) -> Option<OwnedObjectAddress> {
        self.desktop_address.clone()
    }

    fn set_desktop(&mut self, address: OwnedObjectAddress) {
        self.desktop_address = Some(address);
    }

    fn register_event_listener(&mut self, _: String) {
    }

    fn deregister_event_listener(&mut self, _: String) {
    }
}
