// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Live, NodeId, Role};
use accesskit_consumer::{DetachedNode, FilterResult, Node, TreeChangeHandler, TreeState};
use objc2::{
    foundation::{NSInteger, NSMutableDictionary, NSNumber, NSObject, NSString},
    msg_send, Message,
};
use std::{collections::HashSet, rc::Rc};

use crate::{
    appkit::*,
    context::Context,
    filters::{filter, filter_detached},
    node::NodeWrapper,
};

// Workaround for https://github.com/madsmtm/objc2/issues/306
fn set_object_for_key<K: Message, V: Message>(
    dictionary: &mut NSMutableDictionary<K, V>,
    value: &V,
    key: &K,
) {
    let _: () = unsafe { msg_send![dictionary, setObject: value, forKey: key] };
}

// This type is designed to be safe to create on a non-main thread
// and send to the main thread. This ability isn't yet used though.
pub(crate) enum QueuedEvent {
    Generic {
        node_id: NodeId,
        notification: &'static NSString,
    },
    NodeDestroyed(NodeId),
    Announcement {
        text: String,
        priority: NSInteger,
    },
}

impl QueuedEvent {
    fn live_region_announcement(node: &Node) -> Self {
        Self::Announcement {
            text: node.name().unwrap(),
            priority: if node.live() == Live::Assertive {
                NSAccessibilityPriorityHigh
            } else {
                NSAccessibilityPriorityMedium
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

                let mut user_info = NSMutableDictionary::<_, NSObject>::new();
                let text = NSString::from_str(&text);
                set_object_for_key(&mut user_info, &*text, unsafe {
                    NSAccessibilityAnnouncementKey
                });
                let priority = NSNumber::new_isize(priority);
                set_object_for_key(&mut user_info, &*priority, unsafe {
                    NSAccessibilityPriorityKey
                });

                unsafe {
                    NSAccessibilityPostNotificationWithUserInfo(
                        &window,
                        NSAccessibilityAnnouncementRequestedNotification,
                        &user_info,
                    )
                };
            }
        }
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

    pub(crate) fn raise_events(self) {
        for event in self.events {
            event.raise(&self.context);
        }
    }
}

impl TreeChangeHandler for EventGenerator {
    fn node_added(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
        if filter(node) != FilterResult::Include {
            return;
        }
        if node.name().is_some() && node.live() != Live::Off {
            self.events
                .push(QueuedEvent::live_region_announcement(node));
        }
    }

    fn node_updated(&mut self, old_node: &DetachedNode, new_node: &Node) {
        if old_node.raw_value() != new_node.raw_value() {
            self.insert_text_change_if_needed(new_node);
        }
        if filter(new_node) != FilterResult::Include {
            return;
        }
        let node_id = new_node.id();
        let old_wrapper = NodeWrapper::DetachedNode(old_node);
        let new_wrapper = NodeWrapper::Node(new_node);
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
        if new_node.name().is_some()
            && new_node.live() != Live::Off
            && (new_node.name() != old_node.name()
                || new_node.live() != old_node.live()
                || filter_detached(old_node) != FilterResult::Include)
        {
            self.events
                .push(QueuedEvent::live_region_announcement(new_node));
        }
    }

    fn focus_moved(
        &mut self,
        _old_node: Option<&DetachedNode>,
        new_node: Option<&Node>,
        _current_state: &TreeState,
    ) {
        if let Some(new_node) = new_node {
            if filter(new_node) != FilterResult::Include {
                return;
            }
            self.events.push(QueuedEvent::Generic {
                node_id: new_node.id(),
                notification: unsafe { NSAccessibilityFocusedUIElementChangedNotification },
            });
        }
    }

    fn node_removed(&mut self, node: &DetachedNode, current_state: &TreeState) {
        self.insert_text_change_if_needed_for_removed_node(node, current_state);
        self.events.push(QueuedEvent::NodeDestroyed(node.id()));
    }
}
