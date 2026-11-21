// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{Action, Live, Role, Toggled};
use accesskit_consumer::NodeRef;
use jni::{Env, errors::Result, jni_sig, jni_str, objects::JObject, sys::jint};

use crate::{filters::filter, util::*};

pub(crate) fn add_action(env: &mut Env, node_info: &JObject, action: jint) -> Result<()> {
    // Note: We're using the deprecated addAction signature.
    // But this one is much easier to call from JNI since it uses
    // a simple integer constant. Revisit if Android ever gets strict
    // about prohibiting deprecated methods for applications targeting
    // newer SDKs.
    env.call_method(
        node_info,
        jni_str!("addAction"),
        jni_sig!("(I)V"),
        &[action.into()],
    )?;
    Ok(())
}

pub(crate) struct NodeWrapper<'a>(pub(crate) &'a NodeRef<'a>);

impl NodeWrapper<'_> {
    fn is_editable(&self) -> bool {
        self.0.is_text_input() && !self.0.is_read_only()
    }

    fn is_enabled(&self) -> bool {
        !self.0.is_disabled()
    }

    fn is_focusable(&self) -> bool {
        self.0.is_focusable(&filter) && self.0.role() != Role::ScrollView
    }

    fn is_focused(&self) -> bool {
        self.0.is_focused()
    }

    fn is_password(&self) -> bool {
        self.0.is_password_input()
    }

    fn is_checkable(&self) -> bool {
        self.0.toggled().is_some()
    }

    fn is_checked(&self) -> bool {
        match self.0.toggled().unwrap() {
            Toggled::False => false,
            Toggled::True => true,
            Toggled::Mixed => true,
        }
    }

    fn is_scrollable(&self) -> bool {
        self.0.supports_action(Action::ScrollDown, &filter)
            || self.0.supports_action(Action::ScrollLeft, &filter)
            || self.0.supports_action(Action::ScrollRight, &filter)
            || self.0.supports_action(Action::ScrollUp, &filter)
    }

    fn is_selected(&self) -> bool {
        self.0.is_selected().unwrap_or(false)
    }

    pub(crate) fn range(&self) -> Option<Range> {
        Range::from_node(self.0)
    }

    fn supports_set_progress(&self) -> bool {
        self.range().is_some() && self.0.supports_action(Action::SetValue, &filter)
    }

    fn is_range_control(&self) -> bool {
        self.0.supports_increment(&filter) || self.0.supports_decrement(&filter)
    }

    pub(crate) fn content_description(&self) -> Option<String> {
        if self.0.label_comes_from_value() {
            self.0.value()
        } else {
            self.0.label()
        }
    }

    fn url(&self) -> Option<&str> {
        if self.0.supports_url() || self.0.role() == Role::Image {
            self.0.url()
        } else {
            None
        }
    }

    pub(crate) fn text(&self) -> Option<String> {
        if !self.0.label_comes_from_value() {
            if let Some(value) = self.0.value() {
                return Some(value);
            }
        }
        self.0
            .supports_text_ranges()
            .then(|| self.0.document_range().text())
    }

    pub(crate) fn text_selection(&self) -> Option<(usize, usize)> {
        if !self.is_focused() {
            return None;
        }
        self.0.text_selection().map(|range| {
            (
                range.start().to_global_utf16_index(),
                range.end().to_global_utf16_index(),
            )
        })
    }

    fn class_name(&self) -> &str {
        match self.0.role() {
            Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::EmailInput
            | Role::NumberInput
            | Role::PasswordInput
            | Role::PhoneNumberInput
            | Role::UrlInput => "android.widget.EditText",
            Role::Slider => "android.widget.SeekBar",
            Role::ColorWell
            | Role::ComboBox
            | Role::EditableComboBox
            | Role::DateInput
            | Role::DateTimeInput
            | Role::WeekInput
            | Role::MonthInput
            | Role::TimeInput => "android.widget.Spinner",
            Role::Button => {
                if self.0.supports_toggle() {
                    "android.widget.ToggleButton"
                } else {
                    "android.widget.Button"
                }
            }
            Role::PdfActionableHighlight => "android.widget.Button",
            Role::CheckBox => "android.widget.CheckBox",
            Role::RadioButton => "android.widget.RadioButton",
            Role::RadioGroup => "android.widget.RadioGroup",
            Role::Switch => "android.widget.ToggleButton",
            Role::Canvas | Role::Image | Role::SvgRoot => "android.widget.ImageView",
            Role::Meter | Role::ProgressIndicator => "android.widget.ProgressBar",
            Role::TabList => "android.widget.TabWidget",
            Role::Grid | Role::Table | Role::TreeGrid => "android.widget.GridView",
            Role::DescriptionList | Role::List | Role::ListBox | Role::ScrollView => {
                "android.widget.ListView"
            }
            Role::Dialog => "android.app.Dialog",
            Role::RootWebArea => "android.webkit.WebView",
            Role::MenuItem | Role::MenuItemCheckBox | Role::MenuItemRadio => {
                "android.view.MenuItem"
            }
            Role::Label => "android.widget.TextView",
            _ => "android.view.View",
        }
    }

    pub(crate) fn scroll_x(&self) -> Option<jint> {
        self.0
            .scroll_x()
            .map(|value| (value - self.0.scroll_x_min().unwrap_or(0.0)) as jint)
    }

    pub(crate) fn max_scroll_x(&self) -> Option<jint> {
        self.0
            .scroll_x_max()
            .map(|value| (value - self.0.scroll_x_min().unwrap_or(0.0)) as jint)
    }

    pub(crate) fn scroll_y(&self) -> Option<jint> {
        self.0
            .scroll_y()
            .map(|value| (value - self.0.scroll_y_min().unwrap_or(0.0)) as jint)
    }

    pub(crate) fn max_scroll_y(&self) -> Option<jint> {
        self.0
            .scroll_y_max()
            .map(|value| (value - self.0.scroll_y_min().unwrap_or(0.0)) as jint)
    }

    pub(crate) fn populate_node_info(
        &self,
        env: &mut Env,
        host: &JObject,
        id_map: &mut NodeIdMap,
        node_info: &JObject,
    ) -> Result<()> {
        for child in self.0.filtered_children(&filter) {
            env.call_method(
                node_info,
                jni_str!("addChild"),
                jni_sig!("(Landroid/view/View;I)V"),
                &[host.into(), id_map.get_or_create_java_id(&child).into()],
            )?;
        }
        if let Some(parent) = self.0.filtered_parent(&filter) {
            if parent.is_root() {
                env.call_method(
                    node_info,
                    jni_str!("setParent"),
                    jni_sig!("(Landroid/view/View;)V"),
                    &[host.into()],
                )?;
            } else {
                env.call_method(
                    node_info,
                    jni_str!("setParent"),
                    jni_sig!("(Landroid/view/View;I)V"),
                    &[host.into(), id_map.get_or_create_java_id(&parent).into()],
                )?;
            }
        }

        if let Some(rect) = self.0.bounding_box() {
            let location = env.new_int_array(2)?;
            env.call_method(
                host,
                jni_str!("getLocationOnScreen"),
                jni_sig!("([I)V"),
                &[(&location).into()],
            )?;
            let mut location_buf = [0; 2];
            location.get_region(env, 0, &mut location_buf)?;
            let host_screen_x = location_buf[0];
            let host_screen_y = location_buf[1];
            let android_rect_class = env.find_class(jni_str!("android/graphics/Rect"))?;
            let android_rect = env.new_object(
                &android_rect_class,
                jni_sig!("(IIII)V"),
                &[
                    ((rect.x0 as jint) + host_screen_x).into(),
                    ((rect.y0 as jint) + host_screen_y).into(),
                    ((rect.x1 as jint) + host_screen_x).into(),
                    ((rect.y1 as jint) + host_screen_y).into(),
                ],
            )?;
            env.call_method(
                node_info,
                jni_str!("setBoundsInScreen"),
                jni_sig!("(Landroid/graphics/Rect;)V"),
                &[(&android_rect).into()],
            )?;
        }

        if self.is_checkable() {
            env.call_method(
                node_info,
                jni_str!("setCheckable"),
                jni_sig!("(Z)V"),
                &[true.into()],
            )?;
            env.call_method(
                node_info,
                jni_str!("setChecked"),
                jni_sig!("(Z)V"),
                &[self.is_checked().into()],
            )?;
        }
        env.call_method(
            node_info,
            jni_str!("setEditable"),
            jni_sig!("(Z)V"),
            &[self.is_editable().into()],
        )?;
        env.call_method(
            node_info,
            jni_str!("setEnabled"),
            jni_sig!("(Z)V"),
            &[self.is_enabled().into()],
        )?;
        env.call_method(
            node_info,
            jni_str!("setFocusable"),
            jni_sig!("(Z)V"),
            &[self.is_focusable().into()],
        )?;
        env.call_method(
            node_info,
            jni_str!("setFocused"),
            jni_sig!("(Z)V"),
            &[self.is_focused().into()],
        )?;
        env.call_method(
            node_info,
            jni_str!("setPassword"),
            jni_sig!("(Z)V"),
            &[self.is_password().into()],
        )?;
        env.call_method(
            node_info,
            jni_str!("setScrollable"),
            jni_sig!("(Z)V"),
            &[self.is_scrollable().into()],
        )?;
        env.call_method(
            node_info,
            jni_str!("setSelected"),
            jni_sig!("(Z)V"),
            &[self.is_selected().into()],
        )?;
        // TBD: When, if ever, should the visible-to-user property be false?
        env.call_method(
            node_info,
            jni_str!("setVisibleToUser"),
            jni_sig!("(Z)V"),
            &[true.into()],
        )?;
        if let Some(desc) = self.content_description() {
            let desc = env.new_string(desc)?;
            env.call_method(
                node_info,
                jni_str!("setContentDescription"),
                jni_sig!("(Ljava/lang/CharSequence;)V"),
                &[(&desc).into()],
            )?;
        }

        if let Some(text) = self.text() {
            let text = env.new_string(text)?;
            env.call_method(
                node_info,
                jni_str!("setText"),
                jni_sig!("(Ljava/lang/CharSequence;)V"),
                &[(&text).into()],
            )?;
        }
        if let Some((start, end)) = self.text_selection() {
            env.call_method(
                node_info,
                jni_str!("setTextSelection"),
                jni_sig!("(II)V"),
                &[(start as jint).into(), (end as jint).into()],
            )?;
        }

        if let Some(url) = self.url() {
            let extras = env
                .call_method(
                    node_info,
                    jni_str!("getExtras"),
                    jni_sig!("()Landroid/os/Bundle;"),
                    &[],
                )?
                .l()?;
            let key = env.new_string("AccessibilityNodeInfo.targetUrl")?;
            let value = env.new_string(url)?;
            env.call_method(
                &extras,
                jni_str!("putString"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;)V"),
                &[(&key).into(), (&value).into()],
            )?;
        }

        let class_name = env.new_string(self.class_name())?;
        env.call_method(
            node_info,
            jni_str!("setClassName"),
            jni_sig!("(Ljava/lang/CharSequence;)V"),
            &[(&class_name).into()],
        )?;

        let can_focus = self.is_focusable() && !self.0.is_focused();
        // Without a click action, TalkBack handles a double-tap on a range
        // control by tapping its center which can be useful to quickly
        // set a slider value in the middle.
        if self.0.is_clickable(&filter) || (can_focus && !self.is_range_control()) {
            add_action(env, node_info, ACTION_CLICK)?;
        }
        if can_focus {
            add_action(env, node_info, ACTION_FOCUS)?;
        }
        if self.0.supports_text_ranges() {
            add_action(env, node_info, ACTION_SET_SELECTION)?;
            add_action(env, node_info, ACTION_NEXT_AT_MOVEMENT_GRANULARITY)?;
            add_action(env, node_info, ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY)?;
            env.call_method(
                node_info,
                jni_str!("setMovementGranularities"),
                jni_sig!("(I)V"),
                &[(MOVEMENT_GRANULARITY_CHARACTER
                    | MOVEMENT_GRANULARITY_WORD
                    | MOVEMENT_GRANULARITY_LINE
                    | MOVEMENT_GRANULARITY_PARAGRAPH)
                    .into()],
            )?;
        }
        if self.0.supports_action(Action::ScrollLeft, &filter)
            || self.0.supports_action(Action::ScrollUp, &filter)
            || self.0.supports_decrement(&filter)
        {
            add_action(env, node_info, ACTION_SCROLL_BACKWARD)?;
        }
        if self.0.supports_action(Action::ScrollRight, &filter)
            || self.0.supports_action(Action::ScrollDown, &filter)
            || self.0.supports_increment(&filter)
        {
            add_action(env, node_info, ACTION_SCROLL_FORWARD)?;
        }

        if let Some(range) = self.range() {
            let range_info = range.to_java(env)?;
            env.call_method(
                node_info,
                jni_str!("setRangeInfo"),
                jni_sig!("(Landroid/view/accessibility/AccessibilityNodeInfo$RangeInfo;)V"),
                &[(&range_info).into()],
            )?;
        }
        if self.supports_set_progress() {
            let action_class = env.find_class(jni_str!(
                "android/view/accessibility/AccessibilityNodeInfo$AccessibilityAction"
            ))?;
            let action = env
                .get_static_field(
                    &action_class,
                    jni_str!("ACTION_SET_PROGRESS"),
                    jni_sig!(
                        "Landroid/view/accessibility/AccessibilityNodeInfo$AccessibilityAction;"
                    ),
                )?
                .l()?;
            env.call_method(
                node_info,
                jni_str!("addAction"),
                jni_sig!(
                    "(Landroid/view/accessibility/AccessibilityNodeInfo$AccessibilityAction;)V"
                ),
                &[(&action).into()],
            )?;
        }

        let live = match self.0.live() {
            Live::Off => LIVE_REGION_NONE,
            Live::Polite => LIVE_REGION_POLITE,
            Live::Assertive => LIVE_REGION_ASSERTIVE,
        };
        env.call_method(
            node_info,
            jni_str!("setLiveRegion"),
            jni_sig!("(I)V"),
            &[live.into()],
        )?;
        Ok(())
    }
}
