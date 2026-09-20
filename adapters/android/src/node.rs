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
use jni::{JNIEnv, objects::JObject, sys::jint};

use crate::{filters::filter, util::*};

pub(crate) fn add_action(env: &mut JNIEnv, node_info: &JObject, action: jint) {
    // Note: We're using the deprecated addAction signature.
    // But this one is much easier to call from JNI since it uses
    // a simple integer constant. Revisit if Android ever gets strict
    // about prohibiting deprecated methods for applications targeting
    // newer SDKs.
    env.call_method(node_info, "addAction", "(I)V", &[action.into()])
        .unwrap();
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
        self.0.role() == Role::PasswordInput
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

    /// Returns the current, minimum, and maximum values to expose
    /// through `RangeInfo`, if this node has a numeric value
    /// but no textual one.
    fn range(&self) -> Option<(f64, f64, f64)> {
        if self.0.data().value().is_some() {
            return None;
        }
        let current = self.0.numeric_value()?;
        let min = self.0.min_numeric_value()?;
        let max = self.0.max_numeric_value()?;
        Some((current, min, max))
    }

    fn range_type(&self, current: f64, min: f64, max: f64) -> jint {
        let is_integral = |value: f64| value.fract() == 0.0;
        let step_is_integral = self.0.numeric_value_step().is_none_or(is_integral);
        if is_integral(current) && is_integral(min) && is_integral(max) && step_is_integral {
            RANGE_TYPE_INT
        } else {
            RANGE_TYPE_FLOAT
        }
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
        env: &mut JNIEnv,
        host: &JObject,
        id_map: &mut NodeIdMap,
        node_info: &JObject,
    ) {
        for child in self.0.filtered_children(&filter) {
            env.call_method(
                node_info,
                "addChild",
                "(Landroid/view/View;I)V",
                &[host.into(), id_map.get_or_create_java_id(&child).into()],
            )
            .unwrap();
        }
        if let Some(parent) = self.0.filtered_parent(&filter) {
            if parent.is_root() {
                env.call_method(
                    node_info,
                    "setParent",
                    "(Landroid/view/View;)V",
                    &[host.into()],
                )
                .unwrap();
            } else {
                env.call_method(
                    node_info,
                    "setParent",
                    "(Landroid/view/View;I)V",
                    &[host.into(), id_map.get_or_create_java_id(&parent).into()],
                )
                .unwrap();
            }
        }

        if let Some(rect) = self.0.bounding_box() {
            let location = env.new_int_array(2).unwrap();
            env.call_method(host, "getLocationOnScreen", "([I)V", &[(&location).into()])
                .unwrap();
            let mut location_buf = [0; 2];
            env.get_int_array_region(&location, 0, &mut location_buf)
                .unwrap();
            let host_screen_x = location_buf[0];
            let host_screen_y = location_buf[1];
            let android_rect_class = env.find_class("android/graphics/Rect").unwrap();
            let android_rect = env
                .new_object(
                    &android_rect_class,
                    "(IIII)V",
                    &[
                        ((rect.x0 as jint) + host_screen_x).into(),
                        ((rect.y0 as jint) + host_screen_y).into(),
                        ((rect.x1 as jint) + host_screen_x).into(),
                        ((rect.y1 as jint) + host_screen_y).into(),
                    ],
                )
                .unwrap();
            env.call_method(
                node_info,
                "setBoundsInScreen",
                "(Landroid/graphics/Rect;)V",
                &[(&android_rect).into()],
            )
            .unwrap();
        }

        if self.is_checkable() {
            env.call_method(node_info, "setCheckable", "(Z)V", &[true.into()])
                .unwrap();
            env.call_method(node_info, "setChecked", "(Z)V", &[self.is_checked().into()])
                .unwrap();
        }
        env.call_method(
            node_info,
            "setEditable",
            "(Z)V",
            &[self.is_editable().into()],
        )
        .unwrap();
        env.call_method(node_info, "setEnabled", "(Z)V", &[self.is_enabled().into()])
            .unwrap();
        env.call_method(
            node_info,
            "setFocusable",
            "(Z)V",
            &[self.is_focusable().into()],
        )
        .unwrap();
        env.call_method(node_info, "setFocused", "(Z)V", &[self.is_focused().into()])
            .unwrap();
        env.call_method(
            node_info,
            "setPassword",
            "(Z)V",
            &[self.is_password().into()],
        )
        .unwrap();
        env.call_method(
            node_info,
            "setScrollable",
            "(Z)V",
            &[self.is_scrollable().into()],
        )
        .unwrap();
        env.call_method(
            node_info,
            "setSelected",
            "(Z)V",
            &[self.is_selected().into()],
        )
        .unwrap();
        // TBD: When, if ever, should the visible-to-user property be false?
        env.call_method(node_info, "setVisibleToUser", "(Z)V", &[true.into()])
            .unwrap();
        if let Some(desc) = self.content_description() {
            let desc = env.new_string(desc).unwrap();
            env.call_method(
                node_info,
                "setContentDescription",
                "(Ljava/lang/CharSequence;)V",
                &[(&desc).into()],
            )
            .unwrap();
        }

        if let Some(text) = self.text() {
            let text = env.new_string(text).unwrap();
            env.call_method(
                node_info,
                "setText",
                "(Ljava/lang/CharSequence;)V",
                &[(&text).into()],
            )
            .unwrap();
        }
        if let Some((start, end)) = self.text_selection() {
            env.call_method(
                node_info,
                "setTextSelection",
                "(II)V",
                &[(start as jint).into(), (end as jint).into()],
            )
            .unwrap();
        }

        if let Some(url) = self.url() {
            let extras = env
                .call_method(node_info, "getExtras", "()Landroid/os/Bundle;", &[])
                .unwrap()
                .l()
                .unwrap();
            let key = env.new_string("AccessibilityNodeInfo.targetUrl").unwrap();
            let value = env.new_string(url).unwrap();
            env.call_method(
                &extras,
                "putString",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[(&key).into(), (&value).into()],
            )
            .unwrap();
        }

        let class_name = env.new_string(self.class_name()).unwrap();
        env.call_method(
            node_info,
            "setClassName",
            "(Ljava/lang/CharSequence;)V",
            &[(&class_name).into()],
        )
        .unwrap();

        let can_focus = self.is_focusable() && !self.0.is_focused();
        // Like the framework's `SeekBar`, a range control doesn't advertise
        // a click action unless it's really clickable. TalkBack then handles
        // a double-tap by synthesizing a tap at the center of the control,
        // which is how users expect to jump to the middle of the range.
        if self.0.is_clickable(&filter) || (can_focus && !self.is_range_control()) {
            add_action(env, node_info, ACTION_CLICK);
        }
        if can_focus {
            add_action(env, node_info, ACTION_FOCUS);
        }
        if self.0.supports_text_ranges() {
            add_action(env, node_info, ACTION_SET_SELECTION);
            add_action(env, node_info, ACTION_NEXT_AT_MOVEMENT_GRANULARITY);
            add_action(env, node_info, ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY);
            env.call_method(
                node_info,
                "setMovementGranularities",
                "(I)V",
                &[(MOVEMENT_GRANULARITY_CHARACTER
                    | MOVEMENT_GRANULARITY_WORD
                    | MOVEMENT_GRANULARITY_LINE
                    | MOVEMENT_GRANULARITY_PARAGRAPH)
                    .into()],
            )
            .unwrap();
        }
        // Like the framework's own `SeekBar`, a control with a numeric
        // value is adjusted through the scroll actions; TalkBack's
        // "adjust slider" reading control performs nothing else.
        if self.0.supports_action(Action::ScrollLeft, &filter)
            || self.0.supports_action(Action::ScrollUp, &filter)
            || self.0.supports_decrement(&filter)
        {
            add_action(env, node_info, ACTION_SCROLL_BACKWARD);
        }
        if self.0.supports_action(Action::ScrollRight, &filter)
            || self.0.supports_action(Action::ScrollDown, &filter)
            || self.0.supports_increment(&filter)
        {
            add_action(env, node_info, ACTION_SCROLL_FORWARD);
        }

        if let Some((current, min, max)) = self.range() {
            let range_info_class = env
                .find_class("android/view/accessibility/AccessibilityNodeInfo$RangeInfo")
                .unwrap();
            let range_info = env
                .call_static_method(
                    &range_info_class,
                    "obtain",
                    "(IFFF)Landroid/view/accessibility/AccessibilityNodeInfo$RangeInfo;",
                    &[
                        self.range_type(current, min, max).into(),
                        (min as f32).into(),
                        (max as f32).into(),
                        (current as f32).into(),
                    ],
                )
                .unwrap()
                .l()
                .unwrap();
            env.call_method(
                node_info,
                "setRangeInfo",
                "(Landroid/view/accessibility/AccessibilityNodeInfo$RangeInfo;)V",
                &[(&range_info).into()],
            )
            .unwrap();
        }
        if self.supports_set_progress() {
            // Unlike the legacy actions, this one isn't a bitmask
            // and must be added as an `AccessibilityAction` object.
            let action_class = env
                .find_class("android/view/accessibility/AccessibilityNodeInfo$AccessibilityAction")
                .unwrap();
            let action = env
                .get_static_field(
                    &action_class,
                    "ACTION_SET_PROGRESS",
                    "Landroid/view/accessibility/AccessibilityNodeInfo$AccessibilityAction;",
                )
                .unwrap()
                .l()
                .unwrap();
            env.call_method(
                node_info,
                "addAction",
                "(Landroid/view/accessibility/AccessibilityNodeInfo$AccessibilityAction;)V",
                &[(&action).into()],
            )
            .unwrap();
        }

        let live = match self.0.live() {
            Live::Off => LIVE_REGION_NONE,
            Live::Polite => LIVE_REGION_POLITE,
            Live::Assertive => LIVE_REGION_ASSERTIVE,
        };
        env.call_method(node_info, "setLiveRegion", "(I)V", &[live.into()])
            .unwrap();
    }
}
