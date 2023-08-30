// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Role;

use crate::node::{DetachedNode, Node, NodeState};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterResult {
    Include,
    ExcludeNode,
    ExcludeSubtree,
}

fn common_filter_base(node: &NodeState) -> FilterResult {
    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    let role = node.role();
    if role == Role::GenericContainer || role == Role::InlineTextBox {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

pub fn common_filter(node: &Node) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }
    common_filter_base(node.state())
}

pub fn common_filter_detached(node: &DetachedNode) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }
    common_filter_base(node.state())
}

pub fn common_filter_with_root_exception(node: &Node) -> FilterResult {
    if node.is_root() {
        return FilterResult::Include;
    }
    common_filter(node)
}
