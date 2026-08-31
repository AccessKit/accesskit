// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Live, Role};
use accesskit_consumer::{FilterResult, FullNodeId, NodeRef, TreeChangeHandler, TreeState};
use hashbrown::HashSet;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::*;
use objc2_foundation::{NSMutableDictionary, NSNumber, NSString};
use std::{collections::VecDeque, rc::Rc};

use crate::{
    context::Context,
    filters::filter,
    node::{NodeWrapper, Value},
};

// This type is designed to be safe to create on a non-main thread
// and send to the main thread. This ability isn't yet used though.
pub(crate) enum QueuedEvent {
    Generic {
        node_id: FullNodeId,
        notification: &'static NSAccessibilityNotificationName,
    },
    NodeDestroyed(FullNodeId),
    /// A single, batched notification that at least one node changed
    /// geometry during one tree update. The target element is resolved when
    /// the event is raised, not when it is queued, because the tree is still
    /// mutably borrowed while the event generator runs.
    LayoutChanged,
    Announcement {
        text: String,
        priority: NSAccessibilityPriorityLevel,
    },
}

impl QueuedEvent {
    fn live_region_announcement(text: String, live: Live) -> Self {
        Self::Announcement {
            text,
            priority: if live == Live::Assertive {
                NSAccessibilityPriorityLevel::NSAccessibilityPriorityHigh
            } else {
                NSAccessibilityPriorityLevel::NSAccessibilityPriorityMedium
            },
        }
    }

    fn raise(self, context: &Rc<Context>) {
        match self {
            Self::Generic {
                node_id,
                notification,
            } => {
                let platform_node = context.get_or_create_platform_node(node_id);
                unsafe { NSAccessibilityPostNotification(&platform_node, notification) };
            }
            Self::NodeDestroyed(node_id) => {
                if let Some(platform_node) = context.remove_platform_node(node_id) {
                    unsafe {
                        NSAccessibilityPostNotification(
                            &platform_node,
                            NSAccessibilityUIElementDestroyedNotification,
                        )
                    };
                }
            }
            Self::LayoutChanged => {
                // Post on an element rather than on the view, and without
                // user info. `NSAccessibilityUIElementsKey` marks elements
                // that the assistive technology should treat as added or
                // changed, which makes VoiceOver move its cursor and
                // re-announce -- the opposite of the silent "this element's
                // frame is stale" hint we want here.
                //
                // The tree borrow is released before posting, because posting
                // can synchronously re-enter the platform node, which borrows
                // the same tree.
                let target = {
                    let tree = context.tree.borrow();
                    layout_changed_target(tree.state())
                };
                let platform_node = context.get_or_create_platform_node(target);
                unsafe {
                    NSAccessibilityPostNotification(
                        &platform_node,
                        NSAccessibilityLayoutChangedNotification,
                    )
                };
            }
            Self::Announcement { text, priority } => {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return;
                    }
                };

                let window = match view.window() {
                    Some(window) => window,
                    None => {
                        return;
                    }
                };

                let mut user_info = NSMutableDictionary::<_, AnyObject>::new();
                let text = NSString::from_str(&text);
                unsafe {
                    user_info.setObject_forKey(
                        &*text,
                        ProtocolObject::from_ref(NSAccessibilityAnnouncementKey),
                    )
                };
                let priority = NSNumber::new_isize(priority.0);
                unsafe {
                    user_info.setObject_forKey(
                        &*priority,
                        ProtocolObject::from_ref(NSAccessibilityPriorityKey),
                    )
                };

                unsafe {
                    NSAccessibilityPostNotificationWithUserInfo(
                        &window,
                        NSAccessibilityAnnouncementRequestedNotification,
                        Some(&**user_info),
                    )
                };
            }
        }
    }
}

/// Events generated by a tree update.
#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents {
    context: Rc<Context>,
    events: Vec<QueuedEvent>,
}

impl QueuedEvents {
    pub(crate) fn new(context: Rc<Context>, events: Vec<QueuedEvent>) -> Self {
        Self { context, events }
    }

    /// Raise all queued events synchronously.
    ///
    /// It is unknown whether accessibility methods on the view may be
    /// called while events are being raised. This means that any locks
    /// or runtime borrows required to access the adapter must not
    /// be held while this method is called.
    pub fn raise(self) {
        for event in self.events {
            event.raise(&self.context);
        }
    }
}

/// The element that a batched layout-changed notification is posted on: the
/// focused node, or the root if the host window isn't focused.
///
/// A layout-changed notification has no way to name the nodes that moved, so
/// the target only needs to be a stable element in the affected tree. The
/// focused node is preferred because it is the one whose frame the assistive
/// technology is most likely to be relying on.
pub(crate) fn layout_changed_target(state: &TreeState) -> FullNodeId {
    state
        .focus()
        .map_or_else(|| state.root().id(), |node| node.id())
}

pub(crate) fn focus_event(node_id: FullNodeId) -> QueuedEvent {
    QueuedEvent::Generic {
        node_id,
        notification: unsafe { NSAccessibilityFocusedUIElementChangedNotification },
    }
}

#[derive(Default)]
pub(crate) struct EventGenerator {
    events: Vec<QueuedEvent>,
    text_changed: HashSet<FullNodeId>,
    selected_rows_changed: HashSet<FullNodeId>,
    /// How many platform-exposed nodes changed their bounding box during this
    /// update. Only the zero/non-zero distinction matters; see
    /// [`EventGenerator::into_events`].
    moved: usize,
}

impl EventGenerator {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn into_events(mut self) -> Vec<QueuedEvent> {
        // Every geometry change in one update is collapsed into one
        // notification. Posting one per moved node would fire at scroll rate,
        // and VoiceOver responds to a flood of layout-changed notifications by
        // re-announcing repeatedly, which is worse for the user than the stale
        // frame this is meant to fix.
        if self.moved > 0 {
            self.events.push(QueuedEvent::LayoutChanged);
        }
        self.events
    }

    fn remove_subtree(&mut self, node: &NodeRef) {
        let mut to_remove = VecDeque::new();
        to_remove.push_back(*node);

        while let Some(node) = to_remove.pop_front() {
            for child in node.filtered_children(&filter) {
                to_remove.push_back(child);
            }

            self.events.push(QueuedEvent::NodeDestroyed(node.id()));
        }
    }

    fn insert_text_change_if_needed_parent(&mut self, node: NodeRef) {
        if !node.supports_text_ranges() {
            return;
        }
        let id = node.id();
        if self.text_changed.contains(&id) {
            return;
        }
        // Text change events must come before selection change
        // events. It doesn't matter if text change events come
        // before other events.
        self.events.insert(
            0,
            QueuedEvent::Generic {
                node_id: id,
                notification: unsafe { NSAccessibilityValueChangedNotification },
            },
        );
        self.text_changed.insert(id);
    }

    fn insert_text_change_if_needed(&mut self, node: &NodeRef) {
        if node.role() != Role::TextRun {
            return;
        }
        if let Some(node) = node.filtered_parent(&filter) {
            self.insert_text_change_if_needed_parent(node);
        }
    }

    fn enqueue_selected_rows_change_if_needed_parent(&mut self, node: NodeRef) {
        let id = node.id();
        if self.selected_rows_changed.contains(&id) {
            return;
        }
        self.events.push(QueuedEvent::Generic {
            node_id: id,
            notification: unsafe { NSAccessibilitySelectedRowsChangedNotification },
        });
        self.selected_rows_changed.insert(id);
    }

    fn enqueue_selected_rows_change_if_needed(&mut self, node: &NodeRef) {
        let wrapper = NodeWrapper(node);
        if !wrapper.is_item_like() {
            return;
        }
        if let Some(node) = node.selection_container(&filter) {
            self.enqueue_selected_rows_change_if_needed_parent(node);
        }
    }
}

impl TreeChangeHandler for EventGenerator {
    fn node_added(&mut self, node: &NodeRef) {
        self.insert_text_change_if_needed(node);
        if filter(node) != FilterResult::Include {
            return;
        }
        if let Some(true) = node.is_selected() {
            self.enqueue_selected_rows_change_if_needed(node);
        }
        if let Some(label) = NodeWrapper(node).label() {
            if node.live() != Live::Off {
                self.events
                    .push(QueuedEvent::live_region_announcement(label, node.live()));
            }
        }
    }

    fn node_updated(&mut self, old_node: &NodeRef, new_node: &NodeRef) {
        if old_node.raw_value() != new_node.raw_value() {
            self.insert_text_change_if_needed(new_node);
        }
        let old_filter_result = filter(old_node);
        let new_filter_result = filter(new_node);
        if new_filter_result != FilterResult::Include {
            if old_filter_result == FilterResult::Include && old_node.is_selected() == Some(true) {
                self.enqueue_selected_rows_change_if_needed(old_node);
            }
            if new_filter_result == FilterResult::ExcludeSubtree {
                self.remove_subtree(old_node);
            } else {
                self.events.push(QueuedEvent::NodeDestroyed(new_node.id()));
            }
            return;
        }
        let node_id = new_node.id();
        // A node that moved or was resized -- by scrolling, a window reflow or
        // an animation -- reports a new frame, but assistive technologies do
        // not poll geometry. Count it so that `into_events` emits one batched
        // layout-changed notification for this update. Only included nodes get
        // here; the early return above has already handled the rest.
        if old_node.bounding_box() != new_node.bounding_box() {
            self.moved += 1;
        }
        let old_wrapper = NodeWrapper(old_node);
        let new_wrapper = NodeWrapper(new_node);
        if old_wrapper.title() != new_wrapper.title() {
            self.events.push(QueuedEvent::Generic {
                node_id,
                notification: unsafe { NSAccessibilityTitleChangedNotification },
            });
        }
        let new_value = new_wrapper.value();
        if old_wrapper.value() != new_value {
            if !new_node.is_focused() && new_value.is_some_and(|v| matches!(v, Value::Bool(_))) {
                // Bool value changed event for the focused node must come last
                // in order for VoiceOver to announce it. Otherwise, if we raise
                // bool value changed events for other nodes after this one, VoiceOver
                // will announce them instead.
                self.events.insert(
                    0,
                    QueuedEvent::Generic {
                        node_id,
                        notification: unsafe { NSAccessibilityValueChangedNotification },
                    },
                );
            } else {
                self.events.push(QueuedEvent::Generic {
                    node_id,
                    notification: unsafe { NSAccessibilityValueChangedNotification },
                });
            }
        }
        if old_wrapper.supports_text_ranges()
            && new_wrapper.supports_text_ranges()
            && old_wrapper.raw_text_selection() != new_wrapper.raw_text_selection()
        {
            self.events.push(QueuedEvent::Generic {
                node_id,
                notification: unsafe { NSAccessibilitySelectedTextChangedNotification },
            });
        }
        if let Some(new_label) = new_wrapper.label() {
            if new_node.live() != Live::Off
                && (Some(&new_label) != old_wrapper.label().as_ref()
                    || new_node.live() != old_node.live()
                    || old_filter_result != FilterResult::Include)
            {
                self.events.push(QueuedEvent::live_region_announcement(
                    new_label,
                    new_node.live(),
                ));
            }
        }
        if new_node.is_selected() != old_node.is_selected()
            || (old_filter_result != FilterResult::Include && new_node.is_selected() == Some(true))
        {
            self.enqueue_selected_rows_change_if_needed(new_node);
        }
    }

    fn focus_moved(&mut self, _old_node: Option<&NodeRef>, new_node: Option<&NodeRef>) {
        if let Some(new_node) = new_node {
            if filter(new_node) != FilterResult::Include {
                return;
            }
            self.events.push(focus_event(new_node.id()));
        }
    }

    fn node_removed(&mut self, node: &NodeRef) {
        self.insert_text_change_if_needed(node);
        if let Some(true) = node.is_selected() {
            self.enqueue_selected_rows_change_if_needed(node);
        }
        self.events.push(QueuedEvent::NodeDestroyed(node.id()));
    }
}
