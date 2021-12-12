// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::sync::{Arc, Mutex};

use accesskit::{NodeId, TreeUpdate};
use accesskit_consumer::{Node, Tree, TreeChange};

use crate::atspi::{
    interfaces::{ObjectEvent, WindowEvent},
    Bus
};
use crate::node::{PlatformNode, RootPlatformNode};

lazy_static! {
    pub(crate) static ref CURRENT_ACTIVE_WINDOW: Arc<Mutex<Option<NodeId>>> = Arc::new(Mutex::new(None));
}

pub struct Adapter {
    atspi_bus: Bus,
    tree: Arc<Tree>,
}

impl Adapter {
    pub fn new(app_name: String, toolkit_name: String, toolkit_version: String, initial_state: TreeUpdate) -> Option<Self> {
        let mut atspi_bus = Bus::a11y_bus()?;
        let tree = Tree::new(initial_state);
        let app_node = RootPlatformNode::new(app_name, toolkit_name, toolkit_version, tree.clone());
        let mut objects_to_add = Vec::new();

        fn add_children(node: Node, to_add: &mut Vec<PlatformNode>) {
            for child in node.unignored_children() {
                to_add.push(PlatformNode::new(&child));
                add_children(child, to_add);
            }
        }

        objects_to_add.push(PlatformNode::new(&tree.read().root()));
        add_children(tree.read().root(), &mut objects_to_add);
        for node in objects_to_add {
            atspi_bus.register_accessible_interface(node);
        }
        atspi_bus.register_application_interface(app_node);
        Some(Self {
            atspi_bus,
            tree
        })
    }

    pub fn update(&self, update: TreeUpdate) {
        self.tree.update_and_process_changes(update, |change| {
            match change {
                TreeChange::FocusMoved {
                    old_node,
                    new_node,
                } => {
                    if let Some(old_node) = old_node {
                        let old_node = PlatformNode::new(&old_node);
                        self.atspi_bus.emit_object_event(&old_node, ObjectEvent::FocusLost).unwrap();
                    }
                    if let Some(new_node) = new_node {
                        let new_node = PlatformNode::new(&new_node);
                        self.atspi_bus.emit_object_event(&new_node, ObjectEvent::FocusGained).unwrap();
                        self.atspi_bus.emit_focus_event(&new_node).unwrap();
                    }
                }
                TreeChange::NodeUpdated { old_node, new_node } => {
                    //let old_node = ResolvedPlatformNode::new(old_node, self.hwnd);
                    //let new_node = ResolvedPlatformNode::new(new_node, self.hwnd);
                    //new_node.raise_property_changes(&old_node);
                }
                // TODO: handle other events
                _ => (),
            };
        });
    }

    pub fn window_activated(&self, window_id: NodeId) {
        let reader = self.tree.read();
        let node = PlatformNode::new(&reader.node_by_id(window_id).unwrap());
        if let Ok(mut current_active) = CURRENT_ACTIVE_WINDOW.lock() {
            *current_active = Some(window_id);
        }
        self.atspi_bus.emit_window_event(&node, WindowEvent::Activated);
        self.atspi_bus.emit_object_event(&node, ObjectEvent::Activated);
    }

    pub fn window_deactivated(&self, window_id: NodeId) {
        let reader = self.tree.read();
        let node = PlatformNode::new(&reader.node_by_id(window_id).unwrap());
        if let Ok(mut current_active) = CURRENT_ACTIVE_WINDOW.lock() {
            if *current_active == Some(window_id) {
                *current_active = None;
            }
        }
        self.atspi_bus.emit_object_event(&node, ObjectEvent::Deactivated);
        self.atspi_bus.emit_window_event(&node, WindowEvent::Deactivated);
    }

    fn root_platform_node(&self) -> PlatformNode {
        let reader = self.tree.read();
        let node = reader.root();
        PlatformNode::new(&node)
    }
}
