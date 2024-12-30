// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, Node as NodeData, NodeId,
    Point, Role, TextSelection, Tree as TreeData, TreeUpdate,
};
use accesskit_consumer::{FilterResult, Node, TextPosition, Tree, TreeChangeHandler};
use jni::{
    errors::Result,
    objects::{JClass, JObject},
    sys::{jfloat, jint},
    JNIEnv,
};

use crate::{filters::filter, node::NodeWrapper, util::*};

fn send_event(
    env: &mut JNIEnv,
    callback_class: &JClass,
    host: &JObject,
    virtual_view_id: jint,
    event_type: jint,
) {
    env.call_static_method(
        callback_class,
        "sendEvent",
        "(Landroid/view/View;II)V",
        &[host.into(), virtual_view_id.into(), event_type.into()],
    )
    .unwrap();
}

fn send_window_content_changed(env: &mut JNIEnv, callback_class: &JClass, host: &JObject) {
    send_event(
        env,
        callback_class,
        host,
        HOST_VIEW_ID,
        EVENT_WINDOW_CONTENT_CHANGED,
    );
}

fn send_focus_event_if_applicable(
    env: &mut JNIEnv,
    callback_class: &JClass,
    host: &JObject,
    node_id_map: &mut NodeIdMap,
    node: &Node,
) {
    if node.is_root() && node.role() == Role::Window {
        return;
    }
    let id = node_id_map.get_or_create_java_id(node);
    send_event(env, callback_class, host, id, EVENT_VIEW_FOCUSED);
}

struct AdapterChangeHandler<'a, 'b, 'c, 'd> {
    env: &'a mut JNIEnv<'b>,
    callback_class: &'a JClass<'c>,
    host: &'a JObject<'d>,
    node_id_map: &'a mut NodeIdMap,
    sent_window_content_changed: bool,
}

impl<'a, 'b, 'c, 'd> AdapterChangeHandler<'a, 'b, 'c, 'd> {
    fn new(
        env: &'a mut JNIEnv<'b>,
        callback_class: &'a JClass<'c>,
        host: &'a JObject<'d>,
        node_id_map: &'a mut NodeIdMap,
    ) -> Self {
        Self {
            env,
            callback_class,
            host,
            node_id_map,
            sent_window_content_changed: false,
        }
    }
}

impl AdapterChangeHandler<'_, '_, '_, '_> {
    fn send_window_content_changed_if_needed(&mut self) {
        if self.sent_window_content_changed {
            return;
        }
        send_window_content_changed(self.env, self.callback_class, self.host);
        self.sent_window_content_changed = true;
    }
}

impl TreeChangeHandler for AdapterChangeHandler<'_, '_, '_, '_> {
    fn node_added(&mut self, _node: &Node) {
        self.send_window_content_changed_if_needed();
        // TODO: live regions?
    }

    fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
        self.send_window_content_changed_if_needed();
        if filter(new_node) != FilterResult::Include {
            return;
        }
        let old_wrapper = NodeWrapper(old_node);
        let new_wrapper = NodeWrapper(new_node);
        let old_text = old_wrapper.text();
        let new_text = new_wrapper.text();
        if old_text != new_text {
            let id = self.node_id_map.get_or_create_java_id(new_node);
            let old_text = self
                .env
                .new_string(old_text.unwrap_or_else(String::new))
                .unwrap();
            let new_text = self
                .env
                .new_string(new_text.clone().unwrap_or_else(String::new))
                .unwrap();
            self.env
                .call_static_method(
                    self.callback_class,
                    "sendTextChanged",
                    "(Landroid/view/View;ILjava/lang/String;Ljava/lang/String;)V",
                    &[
                        self.host.into(),
                        id.into(),
                        (&old_text).into(),
                        (&new_text).into(),
                    ],
                )
                .unwrap();
        }
        if old_node.raw_text_selection() != new_node.raw_text_selection()
            || (new_node.raw_text_selection().is_some()
                && old_node.is_focused() != new_node.is_focused())
        {
            if let Some((start, end)) = new_wrapper.text_selection() {
                if let Some(text) = new_text {
                    let id = self.node_id_map.get_or_create_java_id(new_node);
                    let text = self.env.new_string(text).unwrap();
                    self.env
                        .call_static_method(
                            self.callback_class,
                            "sendTextSelectionChanged",
                            "(Landroid/view/View;ILjava/lang/String;II)V",
                            &[
                                self.host.into(),
                                id.into(),
                                (&text).into(),
                                (start as jint).into(),
                                (end as jint).into(),
                            ],
                        )
                        .unwrap();
                }
            }
        }
        // TODO: other events
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            send_focus_event_if_applicable(
                self.env,
                self.callback_class,
                self.host,
                self.node_id_map,
                new_node,
            );
        }
    }

    fn node_removed(&mut self, _node: &Node) {
        self.send_window_content_changed_if_needed();
        // TODO: other events?
    }
}

const PLACEHOLDER_ROOT_ID: NodeId = NodeId(0);

#[derive(Default)]
enum State {
    #[default]
    Inactive,
    Placeholder(Tree),
    Active(Tree),
}

impl State {
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

    fn get_full_tree(&mut self) -> Option<&mut Tree> {
        match self {
            Self::Inactive => None,
            Self::Placeholder(_) => None,
            Self::Active(tree) => Some(tree),
        }
    }
}

fn update_tree(
    env: &mut JNIEnv,
    callback_class: &JClass,
    host: &JObject,
    node_id_map: &mut NodeIdMap,
    tree: &mut Tree,
    update: TreeUpdate,
) {
    let mut handler = AdapterChangeHandler::new(env, callback_class, host, node_id_map);
    tree.update_and_process_changes(update, &mut handler);
}

#[derive(Default)]
pub struct Adapter {
    node_id_map: NodeIdMap,
    state: State,
}

/// Low-level AccessKit adapter for Android. This layer provides maximum
/// flexibility in the application threading model, the interface between
/// Java and native code, and the implementation of action callbacks,
/// at the expense of requiring its caller to provide glue code. For a
/// higher-level implementation built on this type, see [`InjectingAdapter`].
///
/// Several of this type's functions have a `callback_class` parameter.
/// The reference implementation of the duck-typed contract for this Java class
/// is `dev.accesskit.android.Delegate`, the source code for which is in the
/// `java` directory of this crate. The methods that are called from native
/// code are all marked `public static`, and so far, all of them that are
/// called by this type (rather than [`InjectingAdapter`]) are for sending
/// events. Other implementations may differ by, for example, sending those
/// events synchronously rather than posting them to the UI thread for
/// asynchronous handling.
///
/// Several of this type's functions have a `host` parameter. This is always
/// a Java object whose class must derive from `android.view.View`.
///
/// [`InjectingAdapter`]: crate::InjectingAdapter
impl Adapter {
    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    pub fn update_if_active(
        &mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
        env: &mut JNIEnv,
        callback_class: &JClass,
        host: &JObject,
    ) {
        match &mut self.state {
            State::Inactive => (),
            State::Placeholder(_) => {
                let tree = Tree::new(update_factory(), true);
                send_window_content_changed(env, callback_class, host);
                let state = tree.state();
                if let Some(focus) = state.focus() {
                    send_focus_event_if_applicable(
                        env,
                        callback_class,
                        host,
                        &mut self.node_id_map,
                        &focus,
                    );
                }
                self.state = State::Active(tree);
            }
            State::Active(tree) => {
                update_tree(
                    env,
                    callback_class,
                    host,
                    &mut self.node_id_map,
                    tree,
                    update_factory(),
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn populate_node_info<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        env: &mut JNIEnv,
        host: &JObject,
        host_screen_x: jint,
        host_screen_y: jint,
        virtual_view_id: jint,
        jni_node: &JObject,
    ) -> Result<bool> {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let node = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root()
        } else {
            let Some(accesskit_id) = self.node_id_map.get_accesskit_id(virtual_view_id) else {
                return Ok(false);
            };
            let Some(node) = tree_state.node_by_id(accesskit_id) else {
                return Ok(false);
            };
            node
        };

        let wrapper = NodeWrapper(&node);
        wrapper.populate_node_info(
            env,
            host,
            host_screen_x,
            host_screen_y,
            &mut self.node_id_map,
            jni_node,
        )?;
        Ok(true)
    }

    pub fn input_focus<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> jint {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let node = tree_state.focus_in_tree();
        self.node_id_map.get_or_create_java_id(&node)
    }

    pub fn virtual_view_at_point<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
        x: jfloat,
        y: jfloat,
    ) -> jint {
        let tree = self.state.get_or_init_tree(activation_handler);
        let tree_state = tree.state();
        let root = tree_state.root();
        let point = Point::new(x.into(), y.into());
        let point = root.transform().inverse() * point;
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        self.node_id_map.get_or_create_java_id(&node)
    }

    pub fn perform_action<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        virtual_view_id: jint,
        action: jint,
    ) -> bool {
        let Some(tree) = self.state.get_full_tree() else {
            return false;
        };
        let tree_state = tree.state();
        let target = if virtual_view_id == HOST_VIEW_ID {
            tree_state.root_id()
        } else {
            let Some(accesskit_id) = self.node_id_map.get_accesskit_id(virtual_view_id) else {
                return false;
            };
            accesskit_id
        };
        let request = match action {
            ACTION_CLICK => ActionRequest {
                action: {
                    let node = tree_state.node_by_id(target).unwrap();
                    if node.is_focusable() && !node.is_focused() && !node.is_clickable() {
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
            _ => {
                return false;
            }
        };
        action_handler.do_action(request);
        true
    }

    fn set_text_selection_common<H: ActionHandler + ?Sized, F, Extra>(
        &mut self,
        action_handler: &mut H,
        env: &mut JNIEnv,
        callback_class: &JClass,
        host: &JObject,
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
        if !node.supports_action(Action::SetTextSelection) {
            return None;
        }
        let (anchor, focus, extra) = selection_factory(&node)?;
        let selection = TextSelection {
            anchor: anchor.to_raw(),
            focus: focus.to_raw(),
        };
        let mut new_node = NodeData::from(node.data());
        new_node.set_text_selection(selection);
        let update = TreeUpdate {
            nodes: vec![(node.id(), new_node)],
            tree: None,
            focus: tree_state.focus_id_in_tree(),
        };
        update_tree(
            env,
            callback_class,
            host,
            &mut self.node_id_map,
            tree,
            update,
        );
        let request = ActionRequest {
            target,
            action: Action::SetTextSelection,
            data: Some(ActionData::SetTextSelection(selection)),
        };
        action_handler.do_action(request);
        Some(extra)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_text_selection<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        env: &mut JNIEnv,
        callback_class: &JClass,
        host: &JObject,
        virtual_view_id: jint,
        anchor: jint,
        focus: jint,
    ) -> bool {
        self.set_text_selection_common(
            action_handler,
            env,
            callback_class,
            host,
            virtual_view_id,
            |node| {
                let anchor = usize::try_from(anchor).ok()?;
                let anchor = node.text_position_from_global_utf16_index(anchor)?;
                let focus = usize::try_from(focus).ok()?;
                let focus = node.text_position_from_global_utf16_index(focus)?;
                Some((anchor, focus, ()))
            },
        )
        .is_some()
    }

    pub fn collapse_text_selection<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        env: &mut JNIEnv,
        callback_class: &JClass,
        host: &JObject,
        virtual_view_id: jint,
    ) -> bool {
        self.set_text_selection_common(
            action_handler,
            env,
            callback_class,
            host,
            virtual_view_id,
            |node| node.text_selection_focus().map(|pos| (pos, pos, ())),
        )
        .is_some()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn traverse_text<H: ActionHandler + ?Sized>(
        &mut self,
        action_handler: &mut H,
        env: &mut JNIEnv,
        callback_class: &JClass,
        host: &JObject,
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    ) -> bool {
        let Some((segment_start, segment_end)) = self.set_text_selection_common(
            action_handler,
            env,
            callback_class,
            host,
            virtual_view_id,
            |node| {
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
            },
        ) else {
            return false;
        };
        env.call_static_method(
            callback_class,
            "sendTextTraversed",
            "(Landroid/view/View;IIZII)V",
            &[
                host.into(),
                virtual_view_id.into(),
                granularity.into(),
                forward.into(),
                (segment_start as jint).into(),
                (segment_end as jint).into(),
            ],
        )
        .unwrap();
        true
    }
}
