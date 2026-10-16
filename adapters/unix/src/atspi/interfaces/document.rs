// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::PlatformNode;
use atspi::TextSelection;
use std::collections::HashMap;
use zbus::{fdo, interface};

fn unsupported() -> fdo::Error {
    fdo::Error::NotSupported("document operation is not supported".into())
}

pub(crate) struct DocumentInterface(PlatformNode);

impl DocumentInterface {
    pub fn new(node: PlatformNode) -> Self {
        Self(node)
    }

    fn map_error(&self) -> impl '_ + FnOnce(accesskit_atspi_common::Error) -> fdo::Error {
        |error| crate::util::map_error_from_node(&self.0, error)
    }
}

#[interface(name = "org.a11y.atspi.Document")]
impl DocumentInterface {
    #[zbus(property)]
    fn current_page_number(&self) -> fdo::Result<i32> {
        Err(unsupported())
    }

    #[zbus(property)]
    fn page_count(&self) -> fdo::Result<i32> {
        Err(unsupported())
    }

    fn get_attribute_value(&self, attribute_name: &str) -> fdo::Result<String> {
        self.0
            .document_attribute_value(attribute_name)
            .map_err(self.map_error())
    }

    fn get_attributes(&self) -> fdo::Result<HashMap<&'static str, String>> {
        self.0.document_attributes().map_err(self.map_error())
    }

    fn get_locale(&self) -> fdo::Result<String> {
        Err(unsupported())
    }

    fn get_text_selections(&self) -> fdo::Result<Vec<TextSelection>> {
        Err(unsupported())
    }

    fn set_text_selections(&self, _selections: Vec<TextSelection>) -> fdo::Result<bool> {
        Err(unsupported())
    }
}
