// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::sync::Arc;

use accesskit_consumer::{Tree, TreeChange};
use accesskit_provider::InitTree;
use accesskit_schema::TreeUpdate;
use lazy_init::LazyTransform;
use windows::Win32::{
    Foundation::*,
    UI::{Accessibility::*, WindowsAndMessaging::*},
};

use crate::node::{PlatformNode, ResolvedPlatformNode};

pub struct Manager<Init: InitTree = TreeUpdate> {
    hwnd: HWND,
    tree: LazyTransform<Init, Arc<Tree>>,
}

impl<Init: InitTree> Manager<Init> {
    pub fn new(hwnd: HWND, init: Init) -> Self {
        // It's unfortunate that we have to force UIA to initialize early;
        // it would be more optimal to let UIA lazily initialize itself
        // when we receive the first `WM_GETOBJECT`. But if we don't do this,
        // then on a thread that's using a COM STA, we can get a race condition
        // that leads to nested WM_GETOBJECT messages and, in some cases,
        // ATs not realizing that our window natively implements UIA. See #37.
        force_init_uia();

        Self {
            hwnd,
            tree: LazyTransform::new(init),
        }
    }

    fn get_or_create_tree(&self) -> &Arc<Tree> {
        self.tree
            .get_or_create(|init| Tree::new(init.init_accesskit_tree()))
    }

    pub fn update(&self, update: TreeUpdate) {
        let tree = self.get_or_create_tree();
        self.update_internal(tree, update);
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        let tree = match self.tree.get() {
            Some(tree) => tree,
            None => {
                return;
            }
        };
        self.update_internal(tree, updater());
    }

    fn update_internal(&self, tree: &Arc<Tree>, update: TreeUpdate) {
        tree.update_and_process_changes(update, |change| {
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
        let tree = self.get_or_create_tree();
        let reader = tree.read();
        let node = reader.root();
        PlatformNode::new(&node, self.hwnd)
    }

    /// Handle the `WM_GETOBJECT` window message.
    ///
    /// This returns an `Option` so the caller can pass the message
    /// to `DefWindowProc` if AccessKit decides not to handle it.
    /// The optional value is an `Into<LRESULT>` rather than simply an `LRESULT`
    /// so the necessary call to UIA, which may lead to a nested `WM_GETOBJECT`
    /// message, can be done outside of any lock that the caller might hold
    /// on the `Manager` or window state, while still abstracting away
    /// the details of that call to UIA.
    ///
    /// Callers must avoid a second deadlock scenario. The tree is lazily
    /// initialized on the first call to this method. So if the caller
    /// holds a lock while calling this method, it must be careful to ensure
    /// that running its tree initialization function while holding that lock
    /// doesn't lead to deadlock.
    pub fn handle_wm_getobject(
        &self,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<impl Into<LRESULT>> {
        // Don't bother with MSAA object IDs that are asking for something other
        // than the client area of the window. DefWindowProc can handle those.
        // First, cast the lparam to i32, to handle inconsistent conversion
        // behavior in senders.
        let objid: i32 = (lparam.0 & 0xFFFFFFFF) as _;
        if objid < 0 && objid != UiaRootObjectId && objid != OBJID_CLIENT.0 {
            return None;
        }

        let el: IRawElementProviderSimple = self.root_platform_node().into();
        Some(WmGetObjectResult {
            hwnd: self.hwnd,
            wparam,
            lparam,
            el,
        })
    }
}

struct WmGetObjectResult {
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    el: IRawElementProviderSimple,
}

impl From<WmGetObjectResult> for LRESULT {
    fn from(this: WmGetObjectResult) -> Self {
        unsafe { UiaReturnRawElementProvider(this.hwnd, this.wparam, this.lparam, this.el) }
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
