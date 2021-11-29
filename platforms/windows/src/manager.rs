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

use crate::{
    node::{PlatformNode, ResolvedPlatformNode},
    util::Event,
};

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

    /// Initialize the tree if it hasn't been initialized already, then apply
    /// the provided update.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let tree = self.get_or_create_tree();
        self.update_internal(tree, update)
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) -> QueuedEvents {
        let tree = match self.tree.get() {
            Some(tree) => tree,
            None => {
                return QueuedEvents(Vec::new());
            }
        };
        self.update_internal(tree, updater())
    }

    fn update_internal(&self, tree: &Arc<Tree>, update: TreeUpdate) -> QueuedEvents {
        let mut queue = Vec::new();
        tree.update_and_process_changes(update, |change| {
            match change {
                TreeChange::FocusMoved {
                    old_node: _,
                    new_node: Some(new_node),
                } => {
                    let platform_node = PlatformNode::new(&new_node, self.hwnd);
                    let element: IRawElementProviderSimple = platform_node.into();
                    queue.push(Event::Simple {
                        element,
                        event_id: UIA_AutomationFocusChangedEventId,
                    });
                }
                TreeChange::NodeUpdated { old_node, new_node } => {
                    let old_node = ResolvedPlatformNode::new(old_node, self.hwnd);
                    let new_node = ResolvedPlatformNode::new(new_node, self.hwnd);
                    new_node.enqueue_property_changes(&mut queue, &old_node);
                }
                // TODO: handle other events (#20)
                _ => (),
            };
        });
        QueuedEvents(queue)
    }

    fn root_platform_node(&self) -> PlatformNode {
        let tree = self.get_or_create_tree();
        let reader = tree.read();
        let node = reader.root();
        PlatformNode::new(&node, self.hwnd)
    }

    pub fn handle_wm_getobject(&self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        // Don't bother with MSAA object IDs that are asking for something other
        // than the client area of the window. DefWindowProc can handle those.
        // First, cast the lparam to i32, to handle inconsistent conversion
        // behavior in senders.
        let objid: i32 = (lparam.0 & 0xFFFFFFFF) as _;
        if objid < 0 && objid != UiaRootObjectId && objid != OBJID_CLIENT.0 {
            return None;
        }

        let el: IRawElementProviderSimple = self.root_platform_node().into();
        Some(unsafe { UiaReturnRawElementProvider(self.hwnd, wparam, lparam, el) })
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

/// Events generated by a tree update.
#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents(Vec<Event>);

impl QueuedEvents {
    /// Raise all queued events synchronously.
    ///
    /// The window may receive `WM_GETOBJECT` messages during this call.
    /// This means that any locks required by the `WM_GETOBJECT` handler
    /// must not be held when this method is called.
    ///
    /// This method should be called on the thread that owns the window.
    /// It's not clear whether this is a strict requirement of UIA itself,
    /// but based on the known behavior of UIA, MSAA, and some ATs,
    /// it's strongly recommended.
    pub fn raise(self) {
        for event in self.0 {
            match event {
                Event::Simple { element, event_id } => {
                    unsafe { UiaRaiseAutomationEvent(element, event_id) }.unwrap();
                }
                Event::PropertyChanged {
                    element,
                    property_id,
                    old_value,
                    new_value,
                } => {
                    unsafe {
                        UiaRaiseAutomationPropertyChangedEvent(
                            element,
                            property_id,
                            old_value,
                            new_value,
                        )
                    }
                    .unwrap();
                }
            }
        }
    }
}

// We explicitly want to allow the queued events to be sent to the UI thread,
// so implement Send even though windows-rs doesn't implement it for all
// contained types. This is safe because we're not using COM threading.
unsafe impl Send for QueuedEvents {}
