// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Role;
use accesskit_consumer::{FilterResult, Node, TreeState};
use web_sys::Element;

use crate::filters::{filter, filter_with_root_exception};

pub(crate) struct NodeWrapper<'a>(pub(crate) Node<'a>);

impl<'a> NodeWrapper<'a> {
    fn role(&self) -> Option<String> {
        let role = self.0.role();
        match role {
            Role::Button => Some("button".into()),
            _ => todo!(),
        }
    }

    fn name(&self) -> Option<String> {
        self.0.name()
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
        self.0.value()
    }
}

macro_rules! attributes {
    ($(($name:literal, $m:ident)),+) => {
        impl NodeWrapper<'_> {
            pub(crate) fn set_all_attributes(&self, element: &Element) {
                $(let value = self.$m();
                if let Some(value) = value.as_ref() {
                    element.set_attribute(&$name, value).unwrap();
                }
                )*
                if let Some(text_content) = self.text_content().as_ref() {
                    element.set_text_content(Some(text_content));
                }
            }
        }
    };
}

attributes! {
    ("role", role),
    ("aria-label", aria_label)
}
