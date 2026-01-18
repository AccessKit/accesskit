// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{Action, Live, Role, Toggled};
use accesskit_consumer::Node;
use jni::{objects::JObject, sys::jint, JNIEnv};

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

pub(crate) struct NodeWrapper<'a>(pub(crate) &'a Node<'a>);

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
        match self.0.role() {
            // https://www.w3.org/TR/core-aam-1.1/#mapping_state-property_table
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-checked for 'radio' and 'menuitemradio' roles.
            Role::RadioButton | Role::MenuItemRadio => self.0.toggled() == Some(Toggled::True),
            // https://www.w3.org/TR/wai-aria-1.1/#aria-selected
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-selected.
            _ => self.0.is_selected().unwrap_or(false),
        }
    }

    fn content_description(&self) -> Option<String> {
        self.0.label()
    }

    fn url(&self) -> Option<&str> {
        if self.0.supports_url() || self.0.role() == Role::Image {
            self.0.url()
        } else {
            None
        }
    }

    pub(crate) fn text(&self) -> Option<String> {
        self.0.value().or_else(|| {
            self.0
                .supports_text_ranges()
                .then(|| self.0.document_range().text())
        })
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
        if self.0.is_clickable(&filter) || can_focus {
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
        if self.0.supports_action(Action::ScrollLeft, &filter)
            || self.0.supports_action(Action::ScrollUp, &filter)
        {
            add_action(env, node_info, ACTION_SCROLL_BACKWARD);
        }
        if self.0.supports_action(Action::ScrollRight, &filter)
            || self.0.supports_action(Action::ScrollDown, &filter)
        {
            add_action(env, node_info, ACTION_SCROLL_FORWARD);
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
