// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use jni::{Env, errors::Result, jni_sig, jni_str, objects::JObject, sys::jint};

use crate::util::*;

fn new_event<'local>(
    env: &mut Env<'local>,
    host: &JObject,
    virtual_view_id: jint,
    event_type: jint,
) -> Result<JObject<'local>> {
    let event_class = env.find_class(jni_str!("android/view/accessibility/AccessibilityEvent"))?;
    let event = env
        .call_static_method(
            &event_class,
            jni_str!("obtain"),
            jni_sig!("(I)Landroid/view/accessibility/AccessibilityEvent;"),
            &[event_type.into()],
        )?
        .l()?;
    let package_name = get_package_name(env, host)?;
    env.call_method(
        &event,
        jni_str!("setPackageName"),
        jni_sig!("(Ljava/lang/CharSequence;)V"),
        &[(&package_name).into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setSource"),
        jni_sig!("(Landroid/view/View;I)V"),
        &[host.into(), virtual_view_id.into()],
    )?;
    Ok(event)
}

fn send_completed_event(env: &mut Env, host: &JObject, event: JObject) -> Result<()> {
    let parent = env
        .call_method(
            host,
            jni_str!("getParent"),
            jni_sig!("()Landroid/view/ViewParent;"),
            &[],
        )?
        .l()?;
    // The host may have been detached from its window by the time
    // a posted event is raised, in which case there's nothing to send it to.
    if parent.is_null() {
        return Ok(());
    }
    env.call_method(
        &parent,
        jni_str!("requestSendAccessibilityEvent"),
        jni_sig!("(Landroid/view/View;Landroid/view/accessibility/AccessibilityEvent;)Z"),
        &[host.into(), (&event).into()],
    )?;
    Ok(())
}

fn send_simple_event(
    env: &mut Env,
    host: &JObject,
    virtual_view_id: jint,
    event_type: jint,
) -> Result<()> {
    let event = new_event(env, host, virtual_view_id, event_type)?;
    send_completed_event(env, host, event)
}

fn send_window_content_changed(env: &mut Env, host: &JObject, virtual_view_id: jint) -> Result<()> {
    let event = new_event(env, host, virtual_view_id, EVENT_WINDOW_CONTENT_CHANGED)?;
    env.call_method(
        &event,
        jni_str!("setContentChangeTypes"),
        jni_sig!("(I)V"),
        &[CONTENT_CHANGE_TYPE_SUBTREE.into()],
    )?;
    send_completed_event(env, host, event)
}

fn send_text_changed(
    env: &mut Env,
    host: &JObject,
    virtual_view_id: jint,
    old: String,
    new: String,
) -> Result<()> {
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
        return Ok(());
    }
    let event = new_event(env, host, virtual_view_id, EVENT_VIEW_TEXT_CHANGED)?;
    let old = env.new_string(old)?;
    env.call_method(
        &event,
        jni_str!("setBeforeText"),
        jni_sig!("(Ljava/lang/CharSequence;)V"),
        &[(&old).into()],
    )?;
    let text_list = env
        .call_method(
            &event,
            jni_str!("getText"),
            jni_sig!("()Ljava/util/List;"),
            &[],
        )?
        .l()?;
    let new = env.new_string(new)?;
    env.call_method(
        &text_list,
        jni_str!("add"),
        jni_sig!("(Ljava/lang/Object;)Z"),
        &[(&new).into()],
    )?;
    // Note: This algorithm, translated from code in Flutter, assumes
    // that the indices are signed.
    let first_difference = i as jint;
    env.call_method(
        &event,
        jni_str!("setFromIndex"),
        jni_sig!("(I)V"),
        &[first_difference.into()],
    )?;
    let mut old_index = old_u16.len() as jint - 1;
    let mut new_index = new_u16.len() as jint - 1;
    while old_index >= first_difference && new_index >= first_difference {
        if old_u16[old_index as usize] != new_u16[new_index as usize] {
            break;
        }
        old_index -= 1;
        new_index -= 1;
    }
    env.call_method(
        &event,
        jni_str!("setRemovedCount"),
        jni_sig!("(I)V"),
        &[(old_index - first_difference + 1).into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setAddedCount"),
        jni_sig!("(I)V"),
        &[(new_index - first_difference + 1).into()],
    )?;
    send_completed_event(env, host, event)
}

fn send_text_selection_changed(
    env: &mut Env,
    host: &JObject,
    virtual_view_id: jint,
    text: String,
    start: jint,
    end: jint,
) -> Result<()> {
    let text_u16_len = text.encode_utf16().count();
    let event = new_event(
        env,
        host,
        virtual_view_id,
        EVENT_VIEW_TEXT_SELECTION_CHANGED,
    )?;
    let text_list = env
        .call_method(
            &event,
            jni_str!("getText"),
            jni_sig!("()Ljava/util/List;"),
            &[],
        )?
        .l()?;
    let text = env.new_string(text)?;
    env.call_method(
        &text_list,
        jni_str!("add"),
        jni_sig!("(Ljava/lang/Object;)Z"),
        &[(&text).into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setFromIndex"),
        jni_sig!("(I)V"),
        &[(start as jint).into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setToIndex"),
        jni_sig!("(I)V"),
        &[(end as jint).into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setItemCount"),
        jni_sig!("(I)V"),
        &[(text_u16_len as jint).into()],
    )?;
    send_completed_event(env, host, event)
}

fn send_text_traversed(
    env: &mut Env,
    host: &JObject,
    virtual_view_id: jint,
    granularity: jint,
    forward: bool,
    segment_start: jint,
    segment_end: jint,
) -> Result<()> {
    let event = new_event(
        env,
        host,
        virtual_view_id,
        EVENT_VIEW_TEXT_TRAVERSED_AT_MOVEMENT_GRANULARITY,
    )?;
    env.call_method(
        &event,
        jni_str!("setMovementGranularity"),
        jni_sig!("(I)V"),
        &[granularity.into()],
    )?;
    let action = if forward {
        ACTION_NEXT_AT_MOVEMENT_GRANULARITY
    } else {
        ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY
    };
    env.call_method(
        &event,
        jni_str!("setAction"),
        jni_sig!("(I)V"),
        &[action.into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setFromIndex"),
        jni_sig!("(I)V"),
        &[segment_start.into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setToIndex"),
        jni_sig!("(I)V"),
        &[segment_end.into()],
    )?;
    send_completed_event(env, host, event)
}

pub(crate) struct ScrollDimension {
    pub(crate) current: jint,
    pub(crate) delta: jint,
    pub(crate) max: Option<jint>,
}

fn send_range_value_changed(
    env: &mut Env,
    host: &JObject,
    virtual_view_id: jint,
    event_type: jint,
    range: Range,
) -> Result<()> {
    let event = new_event(env, host, virtual_view_id, event_type)?;
    env.call_method(
        &event,
        jni_str!("setItemCount"),
        jni_sig!("(I)V"),
        &[100i32.into()],
    )?;
    env.call_method(
        &event,
        jni_str!("setCurrentItemIndex"),
        jni_sig!("(I)V"),
        &[range.current_item_index().into()],
    )?;
    send_completed_event(env, host, event)
}

fn send_scrolled(
    env: &mut Env,
    host: &JObject,
    virtual_view_id: jint,
    x: Option<ScrollDimension>,
    y: Option<ScrollDimension>,
) -> Result<()> {
    let event = new_event(env, host, virtual_view_id, EVENT_VIEW_SCROLLED)?;
    env.call_method(
        &event,
        jni_str!("setScrollable"),
        jni_sig!("(Z)V"),
        &[true.into()],
    )?;
    if let Some(x) = x {
        env.call_method(
            &event,
            jni_str!("setScrollX"),
            jni_sig!("(I)V"),
            &[x.current.into()],
        )?;
        env.call_method(
            &event,
            jni_str!("setScrollDeltaX"),
            jni_sig!("(I)V"),
            &[x.delta.into()],
        )?;
        if let Some(max) = x.max {
            env.call_method(
                &event,
                jni_str!("setMaxScrollX"),
                jni_sig!("(I)V"),
                &[max.into()],
            )?;
        }
    }
    if let Some(y) = y {
        env.call_method(
            &event,
            jni_str!("setScrollY"),
            jni_sig!("(I)V"),
            &[y.current.into()],
        )?;
        env.call_method(
            &event,
            jni_str!("setScrollDeltaY"),
            jni_sig!("(I)V"),
            &[y.delta.into()],
        )?;
        if let Some(max) = y.max {
            env.call_method(
                &event,
                jni_str!("setMaxScrollY"),
                jni_sig!("(I)V"),
                &[max.into()],
            )?;
        }
    }
    send_completed_event(env, host, event)
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
    Scrolled {
        virtual_view_id: jint,
        x: Option<ScrollDimension>,
        y: Option<ScrollDimension>,
    },
    RangeValueChanged {
        virtual_view_id: jint,
        event_type: jint,
        range: Range,
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
    pub fn raise(self, env: &mut Env, host: &JObject) -> Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }
        // The framework throws an exception if events are sent from the
        // UI thread while accessibility is disabled.
        if !is_accessibility_enabled(env, host)? {
            return Ok(());
        }
        for event in self.0 {
            match event {
                QueuedEvent::Simple {
                    virtual_view_id,
                    event_type,
                } => {
                    send_simple_event(env, host, virtual_view_id, event_type)?;
                }
                QueuedEvent::WindowContentChanged { virtual_view_id } => {
                    send_window_content_changed(env, host, virtual_view_id)?;
                }
                QueuedEvent::TextChanged {
                    virtual_view_id,
                    old,
                    new,
                } => {
                    send_text_changed(env, host, virtual_view_id, old, new)?;
                }
                QueuedEvent::TextSelectionChanged {
                    virtual_view_id,
                    text,
                    start,
                    end,
                } => {
                    send_text_selection_changed(env, host, virtual_view_id, text, start, end)?;
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
                    )?;
                }
                QueuedEvent::Scrolled {
                    virtual_view_id,
                    x,
                    y,
                } => {
                    send_scrolled(env, host, virtual_view_id, x, y)?;
                }
                QueuedEvent::RangeValueChanged {
                    virtual_view_id,
                    event_type,
                    range,
                } => {
                    send_range_value_changed(env, host, virtual_view_id, event_type, range)?;
                }
                QueuedEvent::InvalidateHost => {
                    env.call_method(host, jni_str!("invalidate"), jni_sig!("()V"), &[])?;
                }
            }
        }
        Ok(())
    }
}
