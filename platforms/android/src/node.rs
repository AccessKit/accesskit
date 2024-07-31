// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Role, Toggled};
use accesskit_consumer::Node;
use jni::{
    errors::Result,
    objects::{JObject, JValue},
    JNIEnv,
};

use crate::{classes::AccessibilityNodeInfo, filters::filter, util::*};

pub(crate) struct NodeWrapper<'a>(pub(crate) &'a Node<'a>);

impl<'a> NodeWrapper<'a> {
    fn name(&self) -> Option<String> {
        self.0.name()
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

    pub(crate) fn populate_node_info(
        &self,
        env: &mut JNIEnv,
        host: &JObject,
        node_info_class: &AccessibilityNodeInfo,
        id_map: &mut NodeIdMap,
        jni_node: &JObject,
    ) -> Result<()> {
        for child in self.0.filtered_children(&filter) {
            node_info_class.addChild(
                env,
                jni_node,
                object_value(host),
                id_value(id_map, child.id()),
            )?;
        }
        if let Some(parent) = self.0.filtered_parent(&filter) {
            if !parent.is_root() {
                node_info_class.setParent(
                    env,
                    jni_node,
                    object_value(host),
                    id_value(id_map, parent.id()),
                )?;
            }
        }

        if self.is_checkable() {
            node_info_class.setCheckable(env, jni_node, bool_value(true))?;
            node_info_class.setChecked(env, jni_node, bool_value(self.is_checked()))?;
        }
        node_info_class.setEnabled(env, jni_node, bool_value(self.is_enabled()))?;
        node_info_class.setFocusable(env, jni_node, bool_value(self.is_focusable()))?;
        node_info_class.setFocused(env, jni_node, bool_value(self.is_focused()))?;
        node_info_class.setPassword(
            env,
            jni_node,
            bool_value(self.0.role() == Role::PasswordInput),
        )?;
        node_info_class.setSelected(env, jni_node, bool_value(self.is_selected()))?;
        if let Some(name) = self.name() {
            let name = env.new_string(name)?;
            node_info_class.setText(env, jni_node, JValue::Object(&name).as_jni())?;
        }

        Ok(())
    }
}
