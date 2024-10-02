// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{Action, Live, Role, Toggled};
use accesskit_consumer::Node;
use jni::{errors::Result, objects::JObject, sys::jint, JNIEnv};

use crate::{filters::filter, util::*};

pub(crate) struct NodeWrapper<'a>(pub(crate) &'a Node<'a>);

impl<'a> NodeWrapper<'a> {
    fn is_editable(&self) -> bool {
        self.0.is_text_input() && !self.0.is_read_only()
    }

    fn is_enabled(&self) -> bool {
        !self.0.is_disabled()
    }

    fn is_focusable(&self) -> bool {
        self.0.is_focusable()
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
        if self.0.role() == Role::Label {
            return None;
        }
        self.0.name()
    }

    pub(crate) fn text(&self) -> Option<String> {
        self.0.value().or_else(|| {
            if self.0.role() != Role::Label {
                return None;
            }
            self.0.name()
        })
    }

    pub(crate) fn text_selection(&self) -> Option<(usize, usize)> {
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
            Role::DescriptionList | Role::List | Role::ListBox => "android.widget.ListView",
            Role::Dialog => "android.app.Dialog",
            Role::RootWebArea => "android.webkit.WebView",
            Role::MenuItem | Role::MenuItemCheckBox | Role::MenuItemRadio => {
                "android.view.MenuItem"
            }
            Role::Label => "android.widget.TextView",
            _ => "android.view.View",
        }
    }

    pub(crate) fn populate_node_info(
        &self,
        env: &mut JNIEnv,
        host: &JObject,
        host_screen_x: jint,
        host_screen_y: jint,
        id_map: &mut NodeIdMap,
        jni_node: &JObject,
    ) -> Result<()> {
        for child in self.0.filtered_children(&filter) {
            env.call_method(
                jni_node,
                "addChild",
                "(Landroid/view/View;I)V",
                &[host.into(), id_map.get_or_create_java_id(&child).into()],
            )?;
        }
        if let Some(parent) = self.0.filtered_parent(&filter) {
            if parent.is_root() {
                env.call_method(
                    jni_node,
                    "setParent",
                    "(Landroid/view/View;)V",
                    &[host.into()],
                )?;
            } else {
                env.call_method(
                    jni_node,
                    "setParent",
                    "(Landroid/view/View;I)V",
                    &[host.into(), id_map.get_or_create_java_id(&parent).into()],
                )?;
            }
        }

        if let Some(rect) = self.0.bounding_box() {
            let android_rect_class = env.find_class("android/graphics/Rect")?;
            let android_rect = env.new_object(
                &android_rect_class,
                "(IIII)V",
                &[
                    ((rect.x0 as jint) + host_screen_x).into(),
                    ((rect.y0 as jint) + host_screen_y).into(),
                    ((rect.x1 as jint) + host_screen_x).into(),
                    ((rect.y1 as jint) + host_screen_y).into(),
                ],
            )?;
            env.call_method(
                jni_node,
                "setBoundsInScreen",
                "(Landroid/graphics/Rect;)V",
                &[(&android_rect).into()],
            )?;
        }

        if self.is_checkable() {
            env.call_method(jni_node, "setCheckable", "(Z)V", &[true.into()])?;
            env.call_method(jni_node, "setChecked", "(Z)V", &[self.is_checked().into()])?;
        }
        env.call_method(
            jni_node,
            "setEditable",
            "(Z)V",
            &[self.is_editable().into()],
        )?;
        env.call_method(jni_node, "setEnabled", "(Z)V", &[self.is_enabled().into()])?;
        env.call_method(
            jni_node,
            "setFocusable",
            "(Z)V",
            &[self.is_focusable().into()],
        )?;
        env.call_method(jni_node, "setFocused", "(Z)V", &[self.is_focused().into()])?;
        env.call_method(
            jni_node,
            "setPassword",
            "(Z)V",
            &[self.is_password().into()],
        )?;
        env.call_method(
            jni_node,
            "setSelected",
            "(Z)V",
            &[self.is_selected().into()],
        )?;
        if let Some(desc) = self.content_description() {
            let desc = env.new_string(desc)?;
            env.call_method(
                jni_node,
                "setContentDescription",
                "(Ljava/lang/CharSequence;)V",
                &[(&desc).into()],
            )?;
        }

        if let Some(text) = self.text() {
            let text = env.new_string(text)?;
            env.call_method(
                jni_node,
                "setText",
                "(Ljava/lang/CharSequence;)V",
                &[(&text).into()],
            )?;
        }
        if let Some((start, end)) = self.text_selection() {
            env.call_method(
                jni_node,
                "setTextSelection",
                "(II)V",
                &[(start as jint).into(), (end as jint).into()],
            )?;
        }

        let class_name = env.new_string(self.class_name())?;
        env.call_method(
            jni_node,
            "setClassName",
            "(Ljava/lang/CharSequence;)V",
            &[(&class_name).into()],
        )?;

        fn add_action(env: &mut JNIEnv, jni_node: &JObject, action: jint) -> Result<()> {
            // Note: We're using the deprecated addAction signature.
            // But this one is much easier to call from JNI since it uses
            // a simple integer constant. Revisit if Android ever gets strict
            // about prohibiting deprecated methods for applications targeting
            // newer SDKs.
            env.call_method(jni_node, "addAction", "(I)V", &[action.into()])?;
            Ok(())
        }

        let can_focus = self.0.is_focusable() && !self.0.is_focused();
        if self.0.is_clickable() || can_focus {
            add_action(env, jni_node, ACTION_CLICK)?;
        }
        if can_focus {
            add_action(env, jni_node, ACTION_FOCUS)?;
        }
        if self.0.supports_action(Action::SetTextSelection) {
            add_action(env, jni_node, ACTION_SET_SELECTION)?;
        }

        let live = match self.0.live() {
            Live::Off => LIVE_REGION_NONE,
            Live::Polite => LIVE_REGION_POLITE,
            Live::Assertive => LIVE_REGION_ASSERTIVE,
        };
        env.call_method(jni_node, "setLiveRegion", "(I)V", &[live.into()])?;

        Ok(())
    }
}
