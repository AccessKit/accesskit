// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Role;
use accesskit_consumer::{common_filter, common_filter_with_root_exception, FilterResult, Node};

pub(crate) fn filter(node: &Node) -> FilterResult {
    let result = common_filter(node);
    if result != FilterResult::Include {
        return result;
    }

    filter_combobox_descendants(node)
}

pub(crate) fn filter_with_combobox_popup_exception(node: &Node) -> FilterResult {
    common_filter(node)
}

pub(crate) fn filter_with_root_exception(node: &Node) -> FilterResult {
    let result = common_filter_with_root_exception(node);
    if result != FilterResult::Include {
        return result;
    }

    filter_combobox_descendants(node)
}

fn filter_combobox_descendants(node: &Node) -> FilterResult {
    if node.role() == Role::MenuListPopup
        || (node.role() == Role::MenuListOption && !node.is_selected().unwrap_or(false))
    {
        if let Some(parent) = node.filtered_parent(&filter) {
            if parent.role() == Role::ComboBox && parent.is_expanded() == Some(false) {
                return FilterResult::ExcludeNode;
            }
        }
    }

    FilterResult::Include
}
