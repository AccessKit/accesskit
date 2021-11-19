// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::sync::Arc;

use accesskit_consumer::{Tree, TreeChange};
use accesskit_schema::TreeUpdate;
use windows::Win32::{Foundation::*, UI::Accessibility::*};

use crate::node::{PlatformNode, ResolvedPlatformNode};

pub struct Manager {
    hwnd: HWND,
    tree: Arc<Tree>,
}

impl Manager {
    pub fn new(hwnd: HWND, initial_state: TreeUpdate) -> Self {
        // It's unfortunate that we have to force UIA to initialize early;
        // it would be more optimal to let UIA lazily initialize itself
        // when we receive the first `WM_GETOBJECT`. But if we don't do this,
        // then on a thread that's using a COM STA, we can get a race condition
        // that leads to nested WM_GETOBJECT messages and, in some cases,
        // ATs not realizing that our window natively implements UIA. See #37.
        force_init_uia();

        Self {
            hwnd,
            tree: Tree::new(initial_state),
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.tree.update_and_process_changes(update, |change| {
            match change {
                TreeChange::FocusMoved {
                    old_node: _,
                    new_node: Some(new_node),
                } => {
                    let platform_node = PlatformNode::new(&new_node, self.hwnd);
                    let el: IRawElementProviderSimple = platform_node.into();
                    unsafe { UiaRaiseAutomationEvent(el, UIA_AutomationFocusChangedEventId) }
                        .unwrap();
                }
                TreeChange::NodeUpdated { old_node, new_node } => {
                    let old_node = ResolvedPlatformNode::new(old_node, self.hwnd);
                    let new_node = ResolvedPlatformNode::new(new_node, self.hwnd);
                    new_node.raise_property_changes(&old_node);
                }
                // TODO: handle other events (#20)
                _ => (),
            };
        });
    }

    fn root_platform_node(&self) -> PlatformNode {
        let reader = self.tree.read();
        let node = reader.root();
        PlatformNode::new(&node, self.hwnd)
    }

    pub fn handle_wm_getobject(&self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let el: IRawElementProviderSimple = self.root_platform_node().into();
        unsafe { UiaReturnRawElementProvider(self.hwnd, wparam, lparam, el) }
    }
}

fn force_init_uia() {
    // `UiaLookupId` is a cheap way of forcing UIA to initialize itself.
    unsafe {
        UiaLookupId(
            AutomationIdentifierType_Property,
            &ControlType_Property_GUID,
        )
    };
}
