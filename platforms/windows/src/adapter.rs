// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, Live, NodeId, Role, TreeUpdate};
use accesskit_consumer::{DetachedNode, FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use std::{collections::HashSet, marker::PhantomData, sync::Arc};
use windows::Win32::{
    Foundation::*,
    UI::{Accessibility::*, WindowsAndMessaging::*},
};

use crate::{
    context::Context,
    filters::{filter, filter_detached},
    init::UiaInitMarker,
    node::{NodeWrapper, PlatformNode},
    util::QueuedEvent,
};

struct AdapterChangeHandler<'a> {
    context: &'a Arc<Context>,
    queue: Vec<QueuedEvent>,
    text_changed: HashSet<NodeId>,
}

impl AdapterChangeHandler<'_> {
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

    fn raise_events(self) {
        for event in self.queue {
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

impl TreeChangeHandler for AdapterChangeHandler<'_> {
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
        if old_node.raw_value() != new_node.raw_value() {
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

/// A Windows platform adapter.
///
/// This must be owned by the thread that created the associated window handle,
/// and this struct cannot be accessed on any other thread. However, this adapter
/// doesn't use UI Automation's "COM threading" flag, so UI Automation
/// will call most of the provider methods implemented by this adapter
/// on another thread, meaning that the thread that created the window
/// (often called the UI thread) will mostly not be a bottleneck for
/// accessibility queries.
pub struct Adapter {
    context: Arc<Context>,
    _not_send: PhantomData<*const ()>,
}

impl Adapter {
    /// Creates a new Windows platform adapter.
    ///
    /// The action handler may or may not be called on the thread that owns
    /// the window.
    pub fn new(
        hwnd: HWND,
        initial_state: TreeUpdate,
        is_window_focused: bool,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
        _uia_init_marker: UiaInitMarker,
    ) -> Self {
        let context = Context::new(
            hwnd,
            Tree::new(initial_state, is_window_focused),
            action_handler,
        );
        Self {
            context,
            _not_send: PhantomData,
        }
    }

    fn change_handler(&self) -> AdapterChangeHandler {
        AdapterChangeHandler {
            context: &self.context,
            queue: Vec::new(),
            text_changed: HashSet::new(),
        }
    }

    /// Apply the provided update to the tree.
    ///
    /// This method synchronously raises all events generated by the update.
    /// The window may receive the `WM_GETOBJECT` message while these events
    /// are being raised. This means that any mutexes, mutable borrows, or
    /// the like that are required to handle the `WM_GETOBJECT` message must not
    /// be held while this method is called.
    pub fn update(&self, update: TreeUpdate) {
        let mut handler = self.change_handler();
        let mut tree = self.context.tree.write().unwrap();
        tree.update_and_process_changes(update, &mut handler);
        drop(tree);
        handler.raise_events();
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// This method synchronously raises all events generated by the update.
    /// The window may receive the `WM_GETOBJECT` message while these events
    /// are being raised. This means that any mutexes, mutable borrows, or
    /// the like that are required to handle the `WM_GETOBJECT` message must not
    /// be held while this method is called.
    pub fn update_window_focus_state(&self, is_focused: bool) {
        let mut handler = self.change_handler();
        let mut tree = self.context.tree.write().unwrap();
        tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
        drop(tree);
        handler.raise_events();
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
    ///
    /// The window may receive a second `WM_GETOBJECT` message while this method
    /// is running. This means that the caller must not hold any mutex,
    /// mutable borrow, or the like that is necessary to handle that message
    /// while calling this method. The adapter itself can correctly handle
    /// such reentrancy.
    pub fn handle_wm_getobject(&self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        // Don't bother with MSAA object IDs that are asking for something other
        // than the client area of the window. DefWindowProc can handle those.
        // First, cast the lparam to i32, to handle inconsistent conversion
        // behavior in senders.
        let objid = normalize_objid(lparam);
        if objid < 0 && objid != UiaRootObjectId && objid != OBJID_CLIENT.0 {
            return None;
        }

        let el: IRawElementProviderSimple = self.root_platform_node().into();
        let result = unsafe { UiaReturnRawElementProvider(self.context.hwnd, wparam, lparam, &el) };
        Some(result)
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
