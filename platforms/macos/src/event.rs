// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Live, NodeId, Role};
use accesskit_consumer::{FilterResult, Node, TreeChangeHandler};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::*;
use objc2_foundation::{NSMutableDictionary, NSNumber, NSString};
use std::{collections::HashSet, rc::Rc};

use crate::{context::Context, filters::filter, node::NodeWrapper};

// This type is designed to be safe to create on a non-main thread
// and send to the main thread. This ability isn't yet used though.
pub(crate) enum QueuedEvent {
    Generic {
        node_id: NodeId,
        notification: &'static NSAccessibilityNotificationName,
    },
    NodeDestroyed(NodeId),
    Announcement {
        text: String,
        priority: NSAccessibilityPriorityLevel,
    },
}

impl QueuedEvent {
    fn live_region_announcement(node: &Node) -> Self {
        Self::Announcement {
            text: node.value().unwrap(),
            priority: if node.live() == Live::Assertive {
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

pub(crate) fn focus_event(node_id: NodeId) -> QueuedEvent {
    QueuedEvent::Generic {
        node_id,
        notification: unsafe { NSAccessibilityFocusedUIElementChangedNotification },
    }
}

pub(crate) struct EventGenerator {
    context: Rc<Context>,
    events: Vec<QueuedEvent>,
    text_changed: HashSet<NodeId>,
}

impl EventGenerator {
    pub(crate) fn new(context: Rc<Context>) -> Self {
        Self {
            context,
            events: Vec::new(),
            text_changed: HashSet::new(),
        }
    }

    pub(crate) fn into_result(self) -> QueuedEvents {
        QueuedEvents::new(self.context, self.events)
    }

    fn insert_text_change_if_needed_parent(&mut self, node: Node) {
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

    fn insert_text_change_if_needed(&mut self, node: &Node) {
        if node.role() != Role::TextRun {
            return;
        }
        if let Some(node) = node.filtered_parent(&filter) {
            self.insert_text_change_if_needed_parent(node);
        }
    }
}

impl TreeChangeHandler for EventGenerator {
    fn node_added(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
        if filter(node) != FilterResult::Include {
            return;
        }
        if node.value().is_some() && node.live() != Live::Off {
            self.events
                .push(QueuedEvent::live_region_announcement(node));
        }
    }

    fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
        if old_node.raw_value() != new_node.raw_value() {
            self.insert_text_change_if_needed(new_node);
        }
        if filter(new_node) != FilterResult::Include {
            return;
        }
        let node_id = new_node.id();
        let old_wrapper = NodeWrapper(old_node);
        let new_wrapper = NodeWrapper(new_node);
        if old_wrapper.title() != new_wrapper.title() {
            self.events.push(QueuedEvent::Generic {
                node_id,
                notification: unsafe { NSAccessibilityTitleChangedNotification },
            });
        }
        if old_wrapper.value() != new_wrapper.value() {
            self.events.push(QueuedEvent::Generic {
                node_id,
                notification: unsafe { NSAccessibilityValueChangedNotification },
            });
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
        if new_node.value().is_some()
            && new_node.live() != Live::Off
            && (new_node.value() != old_node.value()
                || new_node.live() != old_node.live()
                || filter(old_node) != FilterResult::Include)
        {
            self.events
                .push(QueuedEvent::live_region_announcement(new_node));
        }
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            if filter(new_node) != FilterResult::Include {
                return;
            }
            self.events.push(focus_event(new_node.id()));
        }
    }

    fn node_removed(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
        self.events.push(QueuedEvent::NodeDestroyed(node.id()));
    }
}
