// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Role, Toggled};
use accesskit_consumer::Node;
use jni::{errors::Result, objects::JObject, JNIEnv};

use crate::{filters::filter, util::*};

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

    pub(crate) fn populate_node_info(
        &self,
        env: &mut JNIEnv,
        host: &JObject,
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

        if self.is_checkable() {
            env.call_method(jni_node, "setCheckable", "(Z)V", &[true.into()])?;
            env.call_method(jni_node, "setChecked", "(Z)V", &[self.is_checked().into()])?;
        }
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
        if let Some(name) = self.name() {
            let name = env.new_string(name)?;
            env.call_method(
                jni_node,
                "setText",
                "(Ljava/lang/String;)V",
                &[(&name).into()],
            )?;
        }

        Ok(())
    }
}
