// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, Live, NodeId, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use std::{collections::HashSet, sync::Arc};
use windows::Win32::{
    Foundation::*,
    UI::{Accessibility::*, WindowsAndMessaging::*},
};

use crate::{
    context::Context,
    init::UiaInitMarker,
    node::{filter, filter_detached, NodeWrapper, PlatformNode},
    util::QueuedEvent,
};

pub struct Adapter {
    context: Arc<Context>,
}

impl Adapter {
    /// Creates a new Windows platform adapter.
    ///
    /// The action handler may or may not be called on the thread that owns
    /// the window.
    pub fn new(
        hwnd: HWND,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
        _uia_init_marker: UiaInitMarker,
    ) -> Self {
        let context = Context::new(hwnd, Tree::new(initial_state), action_handler);
        Self { context }
    }

    /// Apply the provided update to the tree.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        struct Handler<'a> {
            context: &'a Arc<Context>,
            queue: Vec<QueuedEvent>,
            text_changed: HashSet<NodeId>,
        }
        impl Handler<'_> {
            fn insert_text_change_if_needed_parent(&mut self, node: Node) {
                if !node.supports_text_ranges() {
                    return;
                }
                let id = node.id();
                if self.text_changed.contains(&id) {
                    return;
                }
                let platform_node = PlatformNode::new(self.context, node.id());
                let element: IRawElementProviderSimple = platform_node.into();
                // Text change events must come before selection change
                // events. It doesn't matter if text change events come
                // before other events.
                self.queue.insert(
                    0,
                    QueuedEvent::Simple {
                        element,
                        event_id: UIA_Text_TextChangedEventId,
                    },
                );
                self.text_changed.insert(id);
            }
            fn insert_text_change_if_needed(&mut self, node: &Node) {
                if node.role() != Role::InlineTextBox {
                    return;
                }
                if let Some(node) = node.filtered_parent(&filter) {
                    self.insert_text_change_if_needed_parent(node);
                }
            }
            fn insert_text_change_if_needed_for_removed_node(
                &mut self,
                node: &DetachedNode,
                current_state: &TreeState,
            ) {
                if node.role() != Role::InlineTextBox {
                    return;
                }
                if let Some(id) = node.parent_id() {
                    if let Some(node) = current_state.node_by_id(id) {
                        self.insert_text_change_if_needed_parent(node);
                    }
                }
            }
        }
        impl TreeChangeHandler for Handler<'_> {
            fn node_added(&mut self, node: &Node) {
                self.insert_text_change_if_needed(node);
                if filter(node) != FilterResult::Include {
                    return;
                }
                if node.name().is_some() && node.live() != Live::Off {
                    let platform_node = PlatformNode::new(self.context, node.id());
                    let element: IRawElementProviderSimple = platform_node.into();
                    self.queue.push(QueuedEvent::Simple {
                        element,
                        event_id: UIA_LiveRegionChangedEventId,
                    });
                }
            }
            fn node_updated(&mut self, old_node: &DetachedNode, new_node: &Node) {
                if old_node.value() != new_node.value() {
                    self.insert_text_change_if_needed(new_node);
                }
                if filter(new_node) != FilterResult::Include {
                    return;
                }
                let platform_node = PlatformNode::new(self.context, new_node.id());
                let element: IRawElementProviderSimple = platform_node.into();
                let old_wrapper = NodeWrapper::DetachedNode(old_node);
                let new_wrapper = NodeWrapper::Node(new_node);
                new_wrapper.enqueue_property_changes(&mut self.queue, &element, &old_wrapper);
                if new_node.name().is_some()
                    && new_node.live() != Live::Off
                    && (new_node.name() != old_node.name()
                        || new_node.live() != old_node.live()
                        || filter_detached(old_node) != FilterResult::Include)
                {
                    self.queue.push(QueuedEvent::Simple {
                        element,
                        event_id: UIA_LiveRegionChangedEventId,
                    });
                }
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&DetachedNode>,
                new_node: Option<&Node>,
                _current_state: &TreeState,
            ) {
                if let Some(new_node) = new_node {
                    let platform_node = PlatformNode::new(self.context, new_node.id());
                    let element: IRawElementProviderSimple = platform_node.into();
                    self.queue.push(QueuedEvent::Simple {
                        element,
                        event_id: UIA_AutomationFocusChangedEventId,
                    });
                }
            }
            fn node_removed(&mut self, node: &DetachedNode, current_state: &TreeState) {
                self.insert_text_change_if_needed_for_removed_node(node, current_state);
            }
            // TODO: handle other events (#20)
        }
        let mut handler = Handler {
            context: &self.context,
            queue: Vec::new(),
            text_changed: HashSet::new(),
        };
        let mut tree = self.context.tree.write().unwrap();
        tree.update_and_process_changes(update, &mut handler);
        QueuedEvents(handler.queue)
    }

    fn root_platform_node(&self) -> PlatformNode {
        let tree = self.context.read_tree();
        let node_id = tree.state().root_id();
        PlatformNode::new(&self.context, node_id)
    }

    /// Handle the `WM_GETOBJECT` window message.
    ///
    /// This returns an `Option` so the caller can pass the message
    /// to `DefWindowProc` if AccessKit decides not to handle it.
    /// The optional value is an `Into<LRESULT>` rather than simply an `LRESULT`
    /// so the necessary call to UIA, which may lead to a nested `WM_GETOBJECT`
    /// message, can be done outside of any lock that the caller might hold
    /// on the `Adapter` or window state, while still abstracting away
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
        let objid = normalize_objid(lparam);
        if objid < 0 && objid != UiaRootObjectId && objid != OBJID_CLIENT.0 {
            return None;
        }

        let el: IRawElementProviderSimple = self.root_platform_node().into();
        Some(WmGetObjectResult {
            hwnd: self.context.hwnd,
            wparam,
            lparam,
            el,
        })
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn normalize_objid(lparam: LPARAM) -> i32 {
    (lparam.0 & 0xFFFFFFFF) as _
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn normalize_objid(lparam: LPARAM) -> i32 {
    lparam.0 as _
}

struct WmGetObjectResult {
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    el: IRawElementProviderSimple,
}

impl From<WmGetObjectResult> for LRESULT {
    fn from(this: WmGetObjectResult) -> Self {
        unsafe { UiaReturnRawElementProvider(this.hwnd, this.wparam, this.lparam, &this.el) }
    }
}

/// Events generated by a tree update.
#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents(Vec<QueuedEvent>);

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
                QueuedEvent::Simple { element, event_id } => {
                    unsafe { UiaRaiseAutomationEvent(&element, event_id) }.unwrap();
                }
                QueuedEvent::PropertyChanged {
                    element,
                    property_id,
                    old_value,
                    new_value,
                } => {
                    unsafe {
                        UiaRaiseAutomationPropertyChangedEvent(
                            &element,
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
