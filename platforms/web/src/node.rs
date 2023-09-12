// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Role;
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, TreeState};
use web_sys::Element;

use crate::filters::{filter, filter_detached, filter_with_root_exception};

pub(crate) enum NodeWrapper<'a> {
    Node(&'a Node<'a>),
    DetachedNode(&'a DetachedNode),
}

impl<'a> NodeWrapper<'a> {
    fn node_state(&self) -> &'a NodeState {
        match self {
            Self::Node(node) => node.state(),
            Self::DetachedNode(node) => node.state(),
        }
    }

    fn role(&self) -> Option<String> {
        let role = self.node_state().role();
        match role {
            Role::Button => Some("button".into()),
            _ => todo!(),
        }
    }

    fn name(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    fn aria_label(&self) -> Option<String> {
        if self.node_state().role() == Role::StaticText {
            return None;
        }
        self.name()
    }

    fn text_content(&self) -> Option<String> {
        if self.node_state().role() != Role::StaticText {
            return None;
        }
        self.name()
    }

    fn value(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.value(),
            Self::DetachedNode(node) => node.value(),
        }
    }
}

macro_rules! attributes {
    ($(($name:literal, $m:ident)),+) => {
        impl NodeWrapper<'_> {
            pub(crate) fn set_all_attributes(&self, element: &Element) {
                $(let value = self.$m();
                if let Some(value) = value.as_ref() {
                    element.set_attribute(&$name, value).unwrap();
                })*
                if let Some(text_content) = self.text_content().as_ref() {
                    element.set_text_content(Some(text_content));
                }
            }
            pub(crate) fn update_attributes(&self, element: &Element, old: &NodeWrapper) {
                $({
                    let old_value = old.$m();
                    let new_value = self.$m();
                    if old_value != new_value {
                        if let Some(value) = new_value.as_ref() {
                            element.set_attribute(&$name, value).unwrap();
                        } else {
                            element.remove_attribute(&$name).unwrap();
                        }
                    }
                })*
                let old_text_content = old.text_content();
                let new_text_content = self.text_content();
                if old_text_content != new_text_content {
                    if let Some(text_content) = new_text_content.as_ref() {
                        element.set_text_content(Some(text_content));
                    } else {
                        element.set_text_content(None);
                    }
                }
            }
        }
    };
}

attributes! {
    ("role", role),
    ("aria-label", aria_label)
}
