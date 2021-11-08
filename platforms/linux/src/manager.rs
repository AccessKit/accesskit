// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::sync::Arc;

use accesskit_consumer::{Tree, TreeChange};
use accesskit_schema::TreeUpdate;

use crate::atspi::Bus;
use crate::node::{PlatformNode, RootPlatformNode};

pub struct Manager {
    atspi_bus: Bus,
    tree: Arc<Tree>,
}

impl Manager {
    pub fn new(app_name: String, toolkit_name: String, toolkit_version: String, initial_state: TreeUpdate) -> Option<Self> {
        let mut atspi_bus = Bus::a11y_bus()?;
        let app_node = RootPlatformNode::new(app_name, toolkit_name, toolkit_version);
        atspi_bus.register_root(app_node);
        Some(Self {
            atspi_bus,
            tree: Tree::new(initial_state),
        })
    }

    pub fn update(&self, update: TreeUpdate) {
        self.tree.update_and_process_changes(update, |change| {
            match change {
                TreeChange::FocusMoved {
                    old_node: _,
                    new_node,
                } => {
                    if let Some(new_node) = new_node {
                        //let platform_node = PlatformNode::new(&new_node, self.hwnd);
                        //let el: IRawElementProviderSimple = platform_node.into();
                        //unsafe { UiaRaiseAutomationEvent(el, UIA_AutomationFocusChangedEventId) }
                        //    .unwrap();
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

    fn root_platform_node(&self) -> PlatformNode {
        let reader = self.tree.read();
        let node = reader.root();
        PlatformNode::new(&node)
    }
}
