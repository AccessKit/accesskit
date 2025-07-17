// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, Node as NodeData, NodeId,
    Orientation, Point, Role, ScrollUnit, TextSelection, Tree as TreeData, TreeUpdate,
};
use accesskit_consumer::{FilterResult, Node, TextPosition, Tree, TreeChangeHandler};
use jni::{
    objects::{JMethodID, JObject},
    sys::{jfloat, jint},
    JNIEnv,
};

use crate::{
    action::{PlatformAction, PlatformActionInner},
    event::{QueuedEvent, QueuedEvents, ScrollDimension},
    filters::filter,
    node::{add_action, NodeWrapper},
    util::*,
};

#[profiling::function]
fn enqueue_window_content_changed(events: &mut Vec<QueuedEvent>) {
    events.push(QueuedEvent::WindowContentChanged {
        virtual_view_id: HOST_VIEW_ID,
    });
}

fn enqueue_focus_event_if_applicable(
    events: &mut Vec<QueuedEvent>,
    node_id_map: &mut NodeIdMap,
    node: &Node,
) {
    if node.is_root() && node.role() == Role::Window {
        return;
    }
    let id = node_id_map.get_or_create_java_id(node);
    events.push(QueuedEvent::Simple {
        virtual_view_id: id,
        event_type: EVENT_VIEW_FOCUSED,
    });
}

struct AdapterChangeHandler<'a> {
    events: &'a mut Vec<QueuedEvent>,
    node_id_map: &'a mut NodeIdMap,
    enqueued_window_content_changed: bool,
}

impl<'a> AdapterChangeHandler<'a> {
    fn new(events: &'a mut Vec<QueuedEvent>, node_id_map: &'a mut NodeIdMap) -> Self {
        Self {
            events,
            node_id_map,
            enqueued_window_content_changed: false,
        }
    }
}

impl AdapterChangeHandler<'_> {
    fn enqueue_window_content_changed_if_needed(&mut self) {
        if self.enqueued_window_content_changed {
            return;
        }
        enqueue_window_content_changed(self.events);
        self.enqueued_window_content_changed = true;
    }
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
    fn node_added(&mut self, _node: &Node) {
        self.enqueue_window_content_changed_if_needed();
        // TODO: live regions?
    }

    #[profiling::function]
    fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
        self.enqueue_window_content_changed_if_needed();
        if filter(new_node) != FilterResult::Include {
            return;
        }
        let old_wrapper = NodeWrapper(old_node);
        let new_wrapper = NodeWrapper(new_node);
        let old_text = old_wrapper.text();
        let new_text = new_wrapper.text();
        if old_text != new_text {
            let id = self.node_id_map.get_or_create_java_id(new_node);
            self.events.push(QueuedEvent::TextChanged {
                virtual_view_id: id,
                old: old_text.unwrap_or_else(String::new),
                new: new_text.clone().unwrap_or_else(String::new),
            });
        }
        if old_node.raw_text_selection() != new_node.raw_text_selection()
            || (new_node.raw_text_selection().is_some()
                && old_node.is_focused() != new_node.is_focused())
        {
            if let Some((start, end)) = new_wrapper.text_selection() {
                if let Some(text) = new_text {
                    let id = self.node_id_map.get_or_create_java_id(new_node);
                    self.events.push(QueuedEvent::TextSelectionChanged {
                        virtual_view_id: id,
                        text,
                        start: start as jint,
                        end: end as jint,
                    });
                }
            }
        }
        let scroll_x = old_wrapper
            .scroll_x()
            .zip(new_wrapper.scroll_x())
            .and_then(|(old, new)| {
                (new != old).then(|| ScrollDimension {
                    current: new,
                    delta: new - old,
                    max: new_wrapper.max_scroll_x(),
                })
            });
        let scroll_y = old_wrapper
            .scroll_y()
            .zip(new_wrapper.scroll_y())
            .and_then(|(old, new)| {
                (new != old).then(|| ScrollDimension {
                    current: new,
                    delta: new - old,
                    max: new_wrapper.max_scroll_y(),
                })
            });
        if scroll_x.is_some() || scroll_y.is_some() {
            let id = self.node_id_map.get_or_create_java_id(new_node);
            self.events.push(QueuedEvent::Scrolled {
                virtual_view_id: id,
                x: scroll_x,
                y: scroll_y,
            });
        }
        // TODO: other events
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            enqueue_focus_event_if_applicable(self.events, self.node_id_map, new_node);
        }
    }

    fn node_removed(&mut self, _node: &Node) {
        self.enqueue_window_content_changed_if_needed();
        // TODO: other events?
    }
}

const PLACEHOLDER_ROOT_ID: NodeId = NodeId(0);

#[derive(Debug, Default)]
enum State {
    #[default]
    Inactive,
    Placeholder(Tree),
    Active(Tree),
}

impl State {
    #[profiling::function]
    fn get_or_init_tree<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> &Tree {
        match self {
            Self::Inactive => {
                *self = match activation_handler.request_initial_tree() {
                    Some(initial_state) => Self::Active(Tree::new(initial_state, true)),
                    None => {
                        let placeholder_update = TreeUpdate {
                            nodes: vec![(PLACEHOLDER_ROOT_ID, NodeData::new(Role::Window))],
                            tree: Some(TreeData::new(PLACEHOLDER_ROOT_ID)),
                            focus: PLACEHOLDER_ROOT_ID,
                        };
                        Self::Placeholder(Tree::new(placeholder_update, true))
                    }
                };
                self.get_or_init_tree(activation_handler)
            }
            Self::Placeholder(tree) => tree,
            Self::Active(tree) => tree,
        }
    }

    #[profiling::function]
    fn get_full_tree(&mut self) -> Option<&mut Tree> {
        match self {
            Self::Inactive => None,
            Self::Placeholder(_) => None,
            Self::Active(tree) => Some(tree),
        }
    }
}

#[profiling::function]
fn update_tree(
    events: &mut Vec<QueuedEvent>,
    node_id_map: &mut NodeIdMap,
    tree: &mut Tree,
    update: TreeUpdate,
) {
    let mut handler = AdapterChangeHandler::new(events, node_id_map);
    tree.update_and_process_changes(update, &mut handler);
}

/// Low-level AccessKit adapter for Android.
///
/// This layer provides maximum flexibility in the application threading
/// model, the interface between Java and native code, and the implementation
/// of action callbacks, at the expense of requiring its caller to provide
/// glue code. For a higher-level implementation built on this type, see
/// [`InjectingAdapter`].
///
/// [`InjectingAdapter`]: crate::InjectingAdapter
#[derive(Debug)]
pub struct Adapter {
    node_id_map: NodeIdMap,
    state: State,
    accessibility_focus: Option<jint>,
    hover_target: Option<jint>,
    add_child_method_id: JMethodID,
}

impl Adapter {
    pub fn new(env: &mut JNIEnv) -> Self {
        let class = env
            .find_class("android/view/accessibility/AccessibilityNodeInfo")
            .unwrap();
        let add_child_method_id = env
            .get_method_id(&class, "addChild", "(Landroid/view/View;I)V")
            .unwrap();
        Self {
            node_id_map: Default::default(),
            state: Default::default(),
            accessibility_focus: None,
            hover_target: None,
            add_child_method_id,
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    #[profiling::function]
    pub fn update_if_active(
        &mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        match &mut self.state {
            State::Inactive => None,
            State::Placeholder(_) => {
                let tree = Tree::new(update_factory(), true);
                let mut events = Vec::new();
                enqueue_window_content_changed(&mut events);
                let state = tree.state();
                if let Some(focus) = state.focus() {
                    enqueue_focus_event_if_applicable(&mut events, &mut self.node_id_map, &focus);
                }
                self.state = State::Active(tree);
                Some(QueuedEvents(events))
            }
            State::Active(tree) => {
                let mut events = Vec::new();
                update_tree(&mut events, &mut self.node_id_map, tree, update_factory());
                Some(QueuedEvents(events))
            }
        }
    }

    /// Create an `AccessibilityNodeInfo` for the AccessKit node
    /// corresponding to the given virtual view ID. Returns null if
    /// there is no such node.
    ///
    /// The `host` parameter is the Android view for this adapter.
    /// It must be an instance of `android.view.View` or a subclass.
    #[profiling::function]
    pub fn create_accessibility_node_info<'local, H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        env: &mut JNIEnv<'local>,
        host: &JObject,
        virtual_view_id: jint,
    ) -> JObject<'local> {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let node = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root()
        } else {
            let Some(accesskit_id) = self.node_id_map.get_accesskit_id(virtual_view_id) else {
                return JObject::null();
            };
            let Some(node) = tree_state.node_by_id(accesskit_id) else {
                return JObject::null();
            };
            node
        };

        let node_info_class = env
            .find_class("android/view/accessibility/AccessibilityNodeInfo")
            .unwrap();
        let node_info = env
            .call_static_method(
                &node_info_class,
                "obtain",
                "(Landroid/view/View;I)Landroid/view/accessibility/AccessibilityNodeInfo;",
                &[host.into(), virtual_view_id.into()],
            )
            .unwrap()
            .l()
            .unwrap();

        let package_name = get_package_name(env, host);
        env.call_method(
            &node_info,
            "setPackageName",
            "(Ljava/lang/CharSequence;)V",
            &[(&package_name).into()],
        )
        .unwrap();

        let wrapper = NodeWrapper(&node);
        wrapper.populate_node_info(
            env,
            self.add_child_method_id,
            host,
            &mut self.node_id_map,
            &node_info,
        );

        let is_accessibility_focus = self.accessibility_focus == Some(virtual_view_id);
        env.call_method(
            &node_info,
            "setAccessibilityFocused",
            "(Z)V",
            &[is_accessibility_focus.into()],
        )
        .unwrap();
        add_action(
            env,
            &node_info,
            if is_accessibility_focus {
                ACTION_CLEAR_ACCESSIBILITY_FOCUS
            } else {
                ACTION_ACCESSIBILITY_FOCUS
            },
        );

        node_info
    }

    /// Create an `AccessibilityNodeInfo` for the AccessKit node
    /// with the given focus type. Returns null if there is no such node.
    ///
    /// The `host` parameter is the Android view for this adapter.
    /// It must be an instance of `android.view.View` or a subclass.
    #[profiling::function]
    pub fn find_focus<'local, H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        env: &mut JNIEnv<'local>,
        host: &JObject,
        focus_type: jint,
    ) -> JObject<'local> {
        let virtual_view_id = match focus_type {
            FOCUS_INPUT => {
                let tree = self.state.get_or_init_tree(activation_handler);
                let tree_state = tree.state();
                let node = tree_state.focus_in_tree();
                self.node_id_map.get_or_create_java_id(&node)
            }
            FOCUS_ACCESSIBILITY => {
                let Some(id) = self.accessibility_focus else {
                    return JObject::null();
                };
                id
            }
            _ => return JObject::null(),
        };
        self.create_accessibility_node_info(activation_handler, env, host, virtual_view_id)
    }

    #[profiling::function]
    fn perform_simple_action<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        virtual_view_id: jint,
        action: jint,
    ) -> Option<QueuedEvents> {
        let tree = self.state.get_full_tree()?;
        let tree_state = tree.state();
        let target = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root_id()
        } else {
            self.node_id_map.get_accesskit_id(virtual_view_id)?
        };
        let mut events = Vec::new();
        let request = match action {
            ACTION_CLICK => ActionRequest {
                action: {
                    let node = tree_state.node_by_id(target).unwrap();
                    if node.is_focusable(&filter)
                        && !node.is_focused()
                        && !node.is_clickable(&filter)
                    {
                        Action::Focus
                    } else {
                        Action::Click
                    }
                },
                target,
                data: None,
            },
            ACTION_FOCUS => ActionRequest {
                action: Action::Focus,
                target,
                data: None,
            },
            ACTION_SCROLL_BACKWARD | ACTION_SCROLL_FORWARD => ActionRequest {
                action: {
                    let node = tree_state.node_by_id(target).unwrap();
                    if let Some(orientation) = node.orientation() {
                        match orientation {
                            Orientation::Horizontal => {
                                if action == ACTION_SCROLL_BACKWARD {
                                    Action::ScrollLeft
                                } else {
                                    Action::ScrollRight
                                }
                            }
                            Orientation::Vertical => {
                                if action == ACTION_SCROLL_BACKWARD {
                                    Action::ScrollUp
                                } else {
                                    Action::ScrollDown
                                }
                            }
                        }
                    } else if action == ACTION_SCROLL_BACKWARD {
                        if node.supports_action(Action::ScrollUp, &filter) {
                            Action::ScrollUp
                        } else {
                            Action::ScrollLeft
                        }
                    } else if node.supports_action(Action::ScrollDown, &filter) {
                        Action::ScrollDown
                    } else {
                        Action::ScrollRight
                    }
                },
                target,
                data: Some(ActionData::ScrollUnit(ScrollUnit::Page)),
            },
            ACTION_ACCESSIBILITY_FOCUS => {
                self.accessibility_focus = Some(virtual_view_id);
                events.push(QueuedEvent::InvalidateHost);
                events.push(QueuedEvent::Simple {
                    virtual_view_id,
                    event_type: EVENT_VIEW_ACCESSIBILITY_FOCUSED,
                });
                return Some(QueuedEvents(events));
            }
            ACTION_CLEAR_ACCESSIBILITY_FOCUS => {
                if self.accessibility_focus == Some(virtual_view_id) {
                    self.accessibility_focus = None;
                }
                events.push(QueuedEvent::InvalidateHost);
                events.push(QueuedEvent::Simple {
                    virtual_view_id,
                    event_type: EVENT_VIEW_ACCESSIBILITY_FOCUS_CLEARED,
                });
                return Some(QueuedEvents(events));
            }
            _ => {
                return None;
            }
        };
        action_handler.do_action(request);
        if action == ACTION_CLICK {
            events.push(QueuedEvent::Simple {
                virtual_view_id,
                event_type: EVENT_VIEW_CLICKED,
            });
        }
        Some(QueuedEvents(events))
    }

    fn set_text_selection_common<H: ActionHandler + ?Sized, F, Extra>(
        &mut self,
        action_handler: &mut H,
        events: &mut Vec<QueuedEvent>,
        virtual_view_id: jint,
        selection_factory: F,
    ) -> Option<Extra>
    where
        for<'a> F: FnOnce(&'a Node<'a>) -> Option<(TextPosition<'a>, TextPosition<'a>, Extra)>,
    {
        let tree = self.state.get_full_tree()?;
        let tree_state = tree.state();
        let node = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root()
        } else {
            let id = self.node_id_map.get_accesskit_id(virtual_view_id)?;
            tree_state.node_by_id(id).unwrap()
        };
        let target = node.id();
        // TalkBack expects the text selection change to take effect
        // immediately, so we optimistically update the node.
        // But don't be *too* optimistic.
        if !node.supports_action(Action::SetTextSelection, &filter) {
            return None;
        }
        let (anchor, focus, extra) = selection_factory(&node)?;
        let selection = TextSelection {
            anchor: anchor.to_raw(),
            focus: focus.to_raw(),
        };
        let mut new_node = node.data().clone();
        new_node.set_text_selection(selection);
        let update = TreeUpdate {
            nodes: vec![(node.id(), new_node)],
            tree: None,
            focus: tree_state.focus_id_in_tree(),
        };
        update_tree(events, &mut self.node_id_map, tree, update);
        let request = ActionRequest {
            target,
            action: Action::SetTextSelection,
            data: Some(ActionData::SetTextSelection(selection)),
        };
        action_handler.do_action(request);
        Some(extra)
    }

    fn set_text_selection<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        virtual_view_id: jint,
        anchor: jint,
        focus: jint,
    ) -> Option<QueuedEvents> {
        let mut events = Vec::new();
        self.set_text_selection_common(action_handler, &mut events, virtual_view_id, |node| {
            let anchor = usize::try_from(anchor).ok()?;
            let anchor = node.text_position_from_global_utf16_index(anchor)?;
            let focus = usize::try_from(focus).ok()?;
            let focus = node.text_position_from_global_utf16_index(focus)?;
            Some((anchor, focus, ()))
        })?;
        Some(QueuedEvents(events))
    }

    fn collapse_text_selection<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        virtual_view_id: jint,
    ) -> Option<QueuedEvents> {
        let mut events = Vec::new();
        self.set_text_selection_common(action_handler, &mut events, virtual_view_id, |node| {
            node.text_selection_focus().map(|pos| (pos, pos, ()))
        })?;
        Some(QueuedEvents(events))
    }

    fn traverse_text<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    ) -> Option<QueuedEvents> {
        let mut events = Vec::new();
        let (segment_start, segment_end) =
            self.set_text_selection_common(action_handler, &mut events, virtual_view_id, |node| {
                let current = node.text_selection_focus().unwrap_or_else(|| {
                    let range = node.document_range();
                    if forward {
                        range.start()
                    } else {
                        range.end()
                    }
                });
                if (forward && current.is_document_end())
                    || (!forward && current.is_document_start())
                {
                    return None;
                }
                let current = if forward {
                    current.biased_to_start()
                } else {
                    current.biased_to_end()
                };
                let (segment_start, segment_end) = match granularity {
                    MOVEMENT_GRANULARITY_CHARACTER => {
                        if forward {
                            (current, current.forward_to_character_end())
                        } else {
                            (current.backward_to_character_start(), current)
                        }
                    }
                    MOVEMENT_GRANULARITY_WORD => {
                        if forward {
                            let start = if current.is_word_start() {
                                current
                            } else {
                                let start = current.forward_to_word_start();
                                if start.is_document_end() {
                                    return None;
                                }
                                start
                            };
                            (start, start.forward_to_word_end())
                        } else {
                            let end = if current.is_line_end() || current.is_word_start() {
                                current
                            } else {
                                let end = current.backward_to_word_start().biased_to_end();
                                if end.is_document_start() {
                                    return None;
                                }
                                end
                            };
                            (end.backward_to_word_start(), end)
                        }
                    }
                    MOVEMENT_GRANULARITY_LINE => {
                        if forward {
                            let start = if current.is_line_start() {
                                current
                            } else {
                                let start = current.forward_to_line_start();
                                if start.is_document_end() {
                                    return None;
                                }
                                start
                            };
                            (start, start.forward_to_line_end())
                        } else {
                            let end = if current.is_line_end() {
                                current
                            } else {
                                let end = current.backward_to_line_start().biased_to_end();
                                if end.is_document_start() {
                                    return None;
                                }
                                end
                            };
                            (end.backward_to_line_start(), end)
                        }
                    }
                    MOVEMENT_GRANULARITY_PARAGRAPH => {
                        if forward {
                            let mut start = current;
                            while start.is_paragraph_separator() {
                                start = start.forward_to_paragraph_start();
                            }
                            if start.is_document_end() {
                                return None;
                            }
                            let mut end = start.forward_to_paragraph_end();
                            let prev = end.backward_to_character_start();
                            if prev.is_paragraph_separator() {
                                end = prev;
                            }
                            (start, end)
                        } else {
                            let mut end = current;
                            while !end.is_document_start()
                                && end.backward_to_character_start().is_paragraph_separator()
                            {
                                end = end.backward_to_character_start();
                            }
                            if end.is_document_start() {
                                return None;
                            }
                            (end.backward_to_paragraph_start(), end)
                        }
                    }
                    _ => {
                        return None;
                    }
                };
                if segment_start == segment_end {
                    return None;
                }
                let focus = if forward { segment_end } else { segment_start };
                let anchor = if extend_selection {
                    node.text_selection_anchor().unwrap_or({
                        if forward {
                            segment_start
                        } else {
                            segment_end
                        }
                    })
                } else {
                    focus
                };
                Some((
                    anchor,
                    focus,
                    (
                        segment_start.to_global_utf16_index(),
                        segment_end.to_global_utf16_index(),
                    ),
                ))
            })?;
        events.push(QueuedEvent::TextTraversed {
            virtual_view_id,
            granularity,
            forward,
            segment_start: segment_start as jint,
            segment_end: segment_end as jint,
        });
        Some(QueuedEvents(events))
    }

    /// Perform the specified accessibility action.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it, and the Java `performAction` method
    /// must return `true`. Otherwise, the Java `performAction` method
    /// must either handle the action some other way or return `false`.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    #[profiling::function]
    pub fn perform_action<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        virtual_view_id: jint,
        action: &PlatformAction,
    ) -> Option<QueuedEvents> {
        match action.0 {
            PlatformActionInner::Simple { action } => {
                self.perform_simple_action(action_handler, virtual_view_id, action)
            }
            PlatformActionInner::SetTextSelection { anchor, focus } => {
                self.set_text_selection(action_handler, virtual_view_id, anchor, focus)
            }
            PlatformActionInner::CollapseTextSelection => {
                self.collapse_text_selection(action_handler, virtual_view_id)
            }
            PlatformActionInner::TraverseText {
                granularity,
                forward,
                extend_selection,
            } => self.traverse_text(
                action_handler,
                virtual_view_id,
                granularity,
                forward,
                extend_selection,
            ),
        }
    }

    #[profiling::function]
    fn virtual_view_at_point<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        x: jfloat,
        y: jfloat,
    ) -> Option<jint> {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let root = tree_state.root();
        let point = Point::new(x.into(), y.into());
        let point = root.transform().inverse() * point;
        let node = root.node_at_point(point, &filter)?;
        Some(self.node_id_map.get_or_create_java_id(&node))
    }

    /// Handle the provided hover event.
    ///
    /// The `action`, `x`, and `y` parameters must be retrieved from
    /// the corresponding properties on an Android motion event. These
    /// parameters are passed individually so you can use either a Java
    /// or NDK event.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it, and if using Java, the event handler
    /// must return `true`. Otherwise, if using Java, the event handler
    /// must either handle the event some other way or return `false`.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    #[profiling::function]
    pub fn on_hover_event<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        action: jint,
        x: jfloat,
        y: jfloat,
    ) -> Option<QueuedEvents> {
        let mut events = Vec::new();
        match action {
            MOTION_ACTION_HOVER_ENTER | MOTION_ACTION_HOVER_MOVE => {
                let new_id = self.virtual_view_at_point(activation_handler, x, y);
                if self.hover_target != new_id {
                    if let Some(virtual_view_id) = new_id {
                        events.push(QueuedEvent::Simple {
                            virtual_view_id,
                            event_type: EVENT_VIEW_HOVER_ENTER,
                        });
                    }
                    if let Some(virtual_view_id) = self.hover_target {
                        events.push(QueuedEvent::Simple {
                            virtual_view_id,
                            event_type: EVENT_VIEW_HOVER_EXIT,
                        });
                    }
                    self.hover_target = new_id;
                }
            }
            MOTION_ACTION_HOVER_EXIT => {
                if let Some(virtual_view_id) = self.hover_target.take() {
                    events.push(QueuedEvent::Simple {
                        virtual_view_id,
                        event_type: EVENT_VIEW_HOVER_EXIT,
                    });
                }
            }
            _ => return None,
        }
        Some(QueuedEvents(events))
    }
}
