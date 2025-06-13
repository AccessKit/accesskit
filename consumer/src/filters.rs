// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Rect, Role};

use crate::node::Node;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterResult {
    Include,
    ExcludeNode,
    ExcludeSubtree,
}

fn filter_for_sibling_clip_check(node: &Node) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }

    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    FilterResult::Include
}

fn is_first_sibling_in_parent_bbox<'a>(
    mut siblings: impl Iterator<Item = Node<'a>>,
    parent_bbox: Rect,
) -> bool {
    siblings.next().is_some_and(|sibling| {
        sibling
            .bounding_box()
            .is_some_and(|bbox| !bbox.intersect(parent_bbox).is_empty())
    })
}

pub fn common_filter(node: &Node) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }

    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    let role = node.role();
    if role == Role::GenericContainer || role == Role::TextRun {
        return FilterResult::ExcludeNode;
    }

    if let Some(parent) = node.parent() {
        if common_filter(&parent) == FilterResult::ExcludeSubtree {
            return FilterResult::ExcludeSubtree;
        }

        if parent.clips_children() {
            if let Some(bbox) = node.bounding_box() {
                if let Some(parent_bbox) = parent.bounding_box() {
                    if bbox.intersect(parent_bbox).is_empty()
                        && !(is_first_sibling_in_parent_bbox(
                            node.following_filtered_siblings(&filter_for_sibling_clip_check),
                            parent_bbox,
                        ) || is_first_sibling_in_parent_bbox(
                            node.preceding_filtered_siblings(&filter_for_sibling_clip_check),
                            parent_bbox,
                        ))
                    {
                        return FilterResult::ExcludeSubtree;
                    }
                }
            }
        }
    }

    FilterResult::Include
}

pub fn common_filter_with_root_exception(node: &Node) -> FilterResult {
    if node.is_root() {
        return FilterResult::Include;
    }
    common_filter(node)
}
