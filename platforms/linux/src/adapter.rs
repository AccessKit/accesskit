// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::sync::{Arc, Mutex};

use accesskit::{ActionHandler, NodeId, Role, TreeUpdate};
use accesskit_consumer::{Node, Tree, TreeChange};

use crate::atspi::{
    interfaces::{Accessible, ObjectEvent, WindowEvent},
    Bus, State,
};
use crate::node::{PlatformNode, RootPlatformNode};

lazy_static! {
    pub(crate) static ref CURRENT_ACTIVE_WINDOW: Arc<Mutex<Option<NodeId>>> =
        Arc::new(Mutex::new(None));
}

pub struct Adapter<'a> {
    atspi_bus: Bus<'a>,
    tree: Arc<Tree>,
}

impl<'a> Adapter<'a> {
    pub fn new(
        app_name: String,
        toolkit_name: String,
        toolkit_version: String,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Option<Self> {
        let mut atspi_bus = Bus::a11y_bus()?;
        let tree = Tree::new(initial_state, action_handler);
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
        Some(Self { atspi_bus, tree })
    }

    pub fn update(&self, update: TreeUpdate) {
        self.tree.update_and_process_changes(update, |change| {
            match change {
                TreeChange::FocusMoved { old_node, new_node } => {
                    if let Some(old_node) = old_node {
                        let old_node = PlatformNode::new(&old_node);
                        self.atspi_bus
                            .emit_object_event(
                                &old_node,
                                ObjectEvent::StateChanged(State::Focused, false),
                            )
                            .unwrap();
                    }
                    if let Ok(mut active_window) = CURRENT_ACTIVE_WINDOW.lock() {
                        let node_window = new_node.map(|node| containing_window(node)).flatten();
                        let node_window_id = node_window.map(|node| node.id());
                        if let Some(active_window_id) = *active_window {
                            if node_window_id != Some(active_window_id) {
                                self.window_deactivated(active_window_id);
                            }
                        }
                        *active_window = node_window_id;
                        if let Some(new_node) = new_node {
                            if let Some(node_window_id) = node_window_id {
                                self.window_activated(node_window_id);
                            }
                            let new_node = PlatformNode::new(&new_node);
                            self.atspi_bus
                                .emit_object_event(
                                    &new_node,
                                    ObjectEvent::StateChanged(State::Focused, true),
                                )
                                .unwrap();
                            self.atspi_bus.emit_focus_event(&new_node).unwrap();
                        }
                    }
                }
                TreeChange::NodeUpdated { old_node, new_node } => {
                    let old_state = PlatformNode::new(&old_node).state();
                    let new_platform_node = PlatformNode::new(&new_node);
                    let new_state = new_platform_node.state();
                    let changed_states = old_state ^ new_state;
                    let mut events = Vec::new();
                    for state in changed_states.iter() {
                        events.push(ObjectEvent::StateChanged(state, new_state.contains(state)));
                    }
                    if let Some(name) = new_node.name() {
                        if old_node.name().as_ref() != Some(&name) {
                            events.push(ObjectEvent::NameChanged(name));
                        }
                    }
                    self.atspi_bus
                        .emit_object_events(&new_platform_node, events);
                }
                // TODO: handle other events
                _ => (),
            };
        });
    }

    fn window_activated(&self, window_id: NodeId) {
        let reader = self.tree.read();
        let node = PlatformNode::new(&reader.node_by_id(window_id).unwrap());
        self.atspi_bus
            .emit_window_event(&node, WindowEvent::Activated);
        self.atspi_bus
            .emit_object_event(&node, ObjectEvent::StateChanged(State::Active, true));
    }

    fn window_deactivated(&self, window_id: NodeId) {
        let reader = self.tree.read();
        let node = PlatformNode::new(&reader.node_by_id(window_id).unwrap());
        self.atspi_bus
            .emit_object_event(&node, ObjectEvent::StateChanged(State::Active, false));
        self.atspi_bus
            .emit_window_event(&node, WindowEvent::Deactivated);
    }

    fn root_platform_node(&self) -> PlatformNode {
        let reader = self.tree.read();
        let node = reader.root();
        PlatformNode::new(&node)
    }
}

fn containing_window(node: Node) -> Option<Node> {
    const WINDOW_ROLES: &[Role] = &[Role::AlertDialog, Role::Dialog, Role::Window];
    if WINDOW_ROLES.contains(&node.role()) {
        Some(node)
    } else {
        while let Some(node) = node.parent() {
            if WINDOW_ROLES.contains(&node.role()) {
                return Some(node);
            }
        }
        None
    }
}
