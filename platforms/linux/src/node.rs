// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{Node, Tree, WeakNode};
use accesskit_schema::Role;
use crate::atspi::{
    interfaces::{AccessibleInterface, ApplicationInterface},
    ObjectId, ObjectRef, OwnedObjectAddress, Role as AtspiRole
};
use std::sync::Arc;

pub struct PlatformNode(WeakNode);

impl PlatformNode {
    pub(crate) fn new(node: &Node) -> Self {
        Self(node.downgrade())
    }
}

impl AccessibleInterface for PlatformNode {
    fn name(&self) -> String {
        self.0.map(|node| node.name().map(|name| name.to_string())).flatten().unwrap_or(String::new())
    }

    fn description(&self) -> String {
        String::new()
    }

    fn parent(&self) -> Option<ObjectRef> {
        Some(self
        .0
        .map(|node| node.parent().map(|parent| ObjectId::from(parent.id().0).to_owned().into()))
        .flatten()
        .unwrap_or(ObjectId::root().into()))
    }

    fn child_count(&self) -> usize {
        self.0.map(|node| node.unignored_children().count()).unwrap_or(0)
    }

    fn locale(&self) -> String {
        String::new()
    }

    fn id(&self) -> ObjectId<'static> {
        self.0.map(|node| ObjectId::from(node.id().0).to_owned()).unwrap()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        self.0.map(|node| node.unignored_children().nth(index).map(|child| ObjectId::from(child.id().0).to_owned().into())).flatten()
    }

    fn children(&self) -> Vec<ObjectRef> {
        todo!()
    }

    fn index_in_parent(&self) -> Option<usize> {
        todo!()
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
}

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

impl ApplicationInterface for RootPlatformNode {
    fn name(&self) -> String {
        self.app_name.clone()
    }

    fn child_count(&self) -> usize {
        1
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        if index == 0 {
            Some(ObjectId::from(self.tree.read().root().id().0).to_owned().into())
        } else {
            None
        }
    }

    fn children(&self) -> Vec<ObjectRef> {
        vec![ObjectId::from(self.tree.read().root().id().0).to_owned().into()]
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
