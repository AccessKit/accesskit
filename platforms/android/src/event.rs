// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use jni::{objects::JObject, sys::jint, JNIEnv};

use crate::util::*;

fn new_event<'local>(
    env: &mut JNIEnv<'local>,
    host: &JObject,
    virtual_view_id: jint,
    event_type: jint,
) -> JObject<'local> {
    let event_class = env
        .find_class("android/view/accessibility/AccessibilityEvent")
        .unwrap();
    let event = env
        .call_static_method(
            &event_class,
            "obtain",
            "(I)Landroid/view/accessibility/AccessibilityEvent;",
            &[event_type.into()],
        )
        .unwrap()
        .l()
        .unwrap();
    let package_name = get_package_name(env, host);
    env.call_method(
        &event,
        "setPackageName",
        "(Ljava/lang/CharSequence;)V",
        &[(&package_name).into()],
    )
    .unwrap();
    env.call_method(
        &event,
        "setSource",
        "(Landroid/view/View;I)V",
        &[host.into(), virtual_view_id.into()],
    )
    .unwrap();
    event
}

fn send_completed_event(env: &mut JNIEnv, host: &JObject, event: JObject) {
    let parent = env
        .call_method(host, "getParent", "()Landroid/view/ViewParent;", &[])
        .unwrap()
        .l()
        .unwrap();
    env.call_method(
        &parent,
        "requestSendAccessibilityEvent",
        "(Landroid/view/View;Landroid/view/accessibility/AccessibilityEvent;)Z",
        &[host.into(), (&event).into()],
    )
    .unwrap();
}

fn send_simple_event(env: &mut JNIEnv, host: &JObject, virtual_view_id: jint, event_type: jint) {
    let event = new_event(env, host, virtual_view_id, event_type);
    send_completed_event(env, host, event);
}

fn send_window_content_changed(env: &mut JNIEnv, host: &JObject, virtual_view_id: jint) {
    let event = new_event(env, host, virtual_view_id, EVENT_WINDOW_CONTENT_CHANGED);
    env.call_method(
        &event,
        "setContentChangeTypes",
        "(I)V",
        &[CONTENT_CHANGE_TYPE_SUBTREE.into()],
    )
    .unwrap();
    send_completed_event(env, host, event);
}

fn send_text_changed(
    env: &mut JNIEnv,
    host: &JObject,
    virtual_view_id: jint,
    old: String,
    new: String,
) {
    let old_u16 = old.encode_utf16().collect::<Vec<u16>>();
    let new_u16 = new.encode_utf16().collect::<Vec<u16>>();
    let mut i = 0usize;
    while i < old_u16.len() && i < new_u16.len() {
        if old_u16[i] != new_u16[i] {
            break;
        }
        i += 1;
    }
    if i == old_u16.len() && i == new_u16.len() {
        // The text didn't change.
        return;
    }
    let event = new_event(env, host, virtual_view_id, EVENT_VIEW_TEXT_CHANGED);
    let old = env.new_string(old).unwrap();
    env.call_method(
        &event,
        "setBeforeText",
        "(Ljava/lang/CharSequence;)V",
        &[(&old).into()],
    )
    .unwrap();
    let text_list = env
        .call_method(&event, "getText", "()Ljava/util/List;", &[])
        .unwrap()
        .l()
        .unwrap();
    let new = env.new_string(new).unwrap();
    env.call_method(&text_list, "add", "(Ljava/lang/Object;)Z", &[(&new).into()])
        .unwrap();
    let first_difference = i;
    env.call_method(
        &event,
        "setFromIndex",
        "(I)V",
        &[(first_difference as jint).into()],
    )
    .unwrap();
    let mut old_index = old_u16.len() - 1;
    let mut new_index = new_u16.len() - 1;
    while old_index >= first_difference && new_index >= first_difference {
        if old_u16[old_index] != new_u16[new_index] {
            break;
        }
        old_index -= 1;
        new_index -= 1;
    }
    env.call_method(
        &event,
        "setRemovedCount",
        "(I)V",
        &[((old_index - first_difference + 1) as jint).into()],
    )
    .unwrap();
    env.call_method(
        &event,
        "setAddedCount",
        "(I)V",
        &[((new_index - first_difference + 1) as jint).into()],
    )
    .unwrap();
    send_completed_event(env, host, event);
}

fn send_text_selection_changed(
    env: &mut JNIEnv,
    host: &JObject,
    virtual_view_id: jint,
    text: String,
    start: jint,
    end: jint,
) {
    let text_u16_len = text.encode_utf16().count();
    let event = new_event(
        env,
        host,
        virtual_view_id,
        EVENT_VIEW_TEXT_SELECTION_CHANGED,
    );
    let text_list = env
        .call_method(&event, "getText", "()Ljava/util/List;", &[])
        .unwrap()
        .l()
        .unwrap();
    let text = env.new_string(text).unwrap();
    env.call_method(
        &text_list,
        "add",
        "(Ljava/lang/Object;)Z",
        &[(&text).into()],
    )
    .unwrap();
    env.call_method(&event, "setFromIndex", "(I)V", &[(start as jint).into()])
        .unwrap();
    env.call_method(&event, "setToIndex", "(I)V", &[(end as jint).into()])
        .unwrap();
    env.call_method(
        &event,
        "setItemCount",
        "(I)V",
        &[(text_u16_len as jint).into()],
    )
    .unwrap();
    send_completed_event(env, host, event);
}

fn send_text_traversed(
    env: &mut JNIEnv,
    host: &JObject,
    virtual_view_id: jint,
    granularity: jint,
    forward: bool,
    segment_start: jint,
    segment_end: jint,
) {
    let event = new_event(
        env,
        host,
        virtual_view_id,
        EVENT_VIEW_TEXT_TRAVERSED_AT_MOVEMENT_GRANULARITY,
    );
    env.call_method(
        &event,
        "setMovementGranularity",
        "(I)V",
        &[granularity.into()],
    )
    .unwrap();
    let action = if forward {
        ACTION_NEXT_AT_MOVEMENT_GRANULARITY
    } else {
        ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY
    };
    env.call_method(&event, "setAction", "(I)V", &[action.into()])
        .unwrap();
    env.call_method(&event, "setFromIndex", "(I)V", &[segment_start.into()])
        .unwrap();
    env.call_method(&event, "setToIndex", "(I)V", &[segment_end.into()])
        .unwrap();
    send_completed_event(env, host, event);
}

pub(crate) enum QueuedEvent {
    Simple {
        virtual_view_id: jint,
        event_type: jint,
    },
    WindowContentChanged {
        virtual_view_id: jint,
    },
    TextChanged {
        virtual_view_id: jint,
        old: String,
        new: String,
    },
    TextSelectionChanged {
        virtual_view_id: jint,
        text: String,
        start: jint,
        end: jint,
    },
    TextTraversed {
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        segment_start: jint,
        segment_end: jint,
    },
    InvalidateHost,
}

/// Events generated by a tree update or accessibility action.
#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents(pub(crate) Vec<QueuedEvent>);

impl QueuedEvents {
    /// Raise all queued events.
    ///
    /// The `host` parameter is the Android view for the adapter that
    /// returned this struct. It must be an instance of `android.view.View`
    /// or a subclass.
    ///
    /// This function must be called on the Android UI thread, while not holding
    /// any locks required by the host view's implementations of Android
    /// framework callbacks.
    pub fn raise(self, env: &mut JNIEnv, host: &JObject) {
        for event in self.0 {
            match event {
                QueuedEvent::Simple {
                    virtual_view_id,
                    event_type,
                } => {
                    send_simple_event(env, host, virtual_view_id, event_type);
                }
                QueuedEvent::WindowContentChanged { virtual_view_id } => {
                    send_window_content_changed(env, host, virtual_view_id);
                }
                QueuedEvent::TextChanged {
                    virtual_view_id,
                    old,
                    new,
                } => {
                    send_text_changed(env, host, virtual_view_id, old, new);
                }
                QueuedEvent::TextSelectionChanged {
                    virtual_view_id,
                    text,
                    start,
                    end,
                } => {
                    send_text_selection_changed(env, host, virtual_view_id, text, start, end);
                }
                QueuedEvent::TextTraversed {
                    virtual_view_id,
                    granularity,
                    forward,
                    segment_start,
                    segment_end,
                } => {
                    send_text_traversed(
                        env,
                        host,
                        virtual_view_id,
                        granularity,
                        forward,
                        segment_start,
                        segment_end,
                    );
                }
                QueuedEvent::InvalidateHost => {
                    env.call_method(host, "invalidate", "()V", &[]).unwrap();
                }
            }
        }
    }
}
