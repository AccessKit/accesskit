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

fn common_filter_base(node: &Node) -> Option<FilterResult> {
    if node.is_focused() {
        return Some(FilterResult::Include);
    }

    if node.is_hidden() {
        return Some(FilterResult::ExcludeSubtree);
    }

    // Graft nodes are structural and should be excluded
    if node.is_graft() {
        return Some(FilterResult::ExcludeNode);
    }

    let role = node.role();
    if role == Role::GenericContainer || role == Role::TextRun {
        return Some(FilterResult::ExcludeNode);
    }

    None
}

fn common_filter_without_parent_checks(node: &Node) -> FilterResult {
    common_filter_base(node).unwrap_or(FilterResult::Include)
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
    if let Some(result) = common_filter_base(node) {
        return result;
    }

    if let Some(parent) = node.parent() {
        if common_filter(&parent) == FilterResult::ExcludeSubtree {
            return FilterResult::ExcludeSubtree;
        }
    }

    if let Some(parent) = node.filtered_parent(&common_filter_without_parent_checks) {
        if parent.clips_children() {
            // If the parent clips its children, then exclude this subtree
            // if this child's bounding box isn't inside the parent's bounding
            // box, and if the previous or next filtered sibling isn't inside
            // the parent's bounding box either. The latter condition is meant
            // to allow off-screen items to be seen by consumers so they can be
            // scrolled into view.
            if let Some(bbox) = node.bounding_box() {
                if let Some(parent_bbox) = parent.bounding_box() {
                    if bbox.intersect(parent_bbox).is_empty()
                        && !(is_first_sibling_in_parent_bbox(
                            node.following_filtered_siblings(&common_filter_without_parent_checks),
                            parent_bbox,
                        ) || is_first_sibling_in_parent_bbox(
                            node.preceding_filtered_siblings(&common_filter_without_parent_checks),
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

#[cfg(test)]
mod tests {
    use accesskit::{Node, NodeId as SchemaNodeId, Rect, Role, Tree, TreeId, TreeUpdate, Uuid};
    use alloc::vec;

    use super::{
        common_filter, common_filter_with_root_exception,
        FilterResult::{self, *},
    };

    fn tree_id() -> TreeId {
        TreeId(Uuid::nil())
    }

    fn node_id(n: u64) -> crate::node::NodeId {
        crate::node::NodeId::new(SchemaNodeId(n), crate::tree::TreeIndex(0))
    }

    #[track_caller]
    fn assert_filter_result(expected: FilterResult, tree: &crate::Tree, id: SchemaNodeId) {
        assert_eq!(
            expected,
            common_filter(&tree.state().node_by_id(node_id(id.0)).unwrap())
        );
    }

    #[test]
    fn normal() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_filter_result(Include, &tree, SchemaNodeId(1));
    }

    #[test]
    fn hidden() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::Button);
                    node.set_hidden();
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(1));
    }

    #[test]
    fn hidden_but_focused() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::Button);
                    node.set_hidden();
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(1),
        };
        let tree = crate::Tree::new(update, true);
        assert_filter_result(Include, &tree, SchemaNodeId(1));
    }

    #[test]
    fn generic_container() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_filter_result(ExcludeNode, &tree, SchemaNodeId(0));
        assert_eq!(
            Include,
            common_filter_with_root_exception(&tree.state().node_by_id(node_id(0)).unwrap())
        );
        assert_filter_result(Include, &tree, SchemaNodeId(1));
    }

    #[test]
    fn hidden_parent() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_hidden();
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(0));
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(1));
    }

    #[test]
    fn hidden_parent_but_focused() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_hidden();
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(1),
        };
        let tree = crate::Tree::new(update, true);
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(0));
        assert_filter_result(Include, &tree, SchemaNodeId(1));
    }

    #[test]
    fn text_run() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::TextInput);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::TextRun)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_filter_result(ExcludeNode, &tree, SchemaNodeId(1));
    }

    fn clipped_children_test_tree() -> crate::Tree {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::ScrollView);
                    node.set_clips_children();
                    node.set_bounds(Rect::new(0.0, 0.0, 30.0, 30.0));
                    node.set_children(vec![
                        SchemaNodeId(1),
                        SchemaNodeId(2),
                        SchemaNodeId(3),
                        SchemaNodeId(4),
                        SchemaNodeId(5),
                        SchemaNodeId(6),
                        SchemaNodeId(7),
                        SchemaNodeId(8),
                        SchemaNodeId(9),
                        SchemaNodeId(10),
                        SchemaNodeId(11),
                    ]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, -30.0, 30.0, -20.0));
                    node
                }),
                (SchemaNodeId(2), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, -20.0, 30.0, -10.0));
                    node
                }),
                (SchemaNodeId(3), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, -10.0, 30.0, 0.0));
                    node
                }),
                (SchemaNodeId(4), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_hidden();
                    node
                }),
                (SchemaNodeId(5), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, 0.0, 30.0, 10.0));
                    node
                }),
                (SchemaNodeId(6), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, 10.0, 30.0, 20.0));
                    node
                }),
                (SchemaNodeId(7), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, 20.0, 30.0, 30.0));
                    node
                }),
                (SchemaNodeId(8), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_hidden();
                    node
                }),
                (SchemaNodeId(9), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, 30.0, 30.0, 40.0));
                    node
                }),
                (SchemaNodeId(10), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, 40.0, 30.0, 50.0));
                    node
                }),
                (SchemaNodeId(11), {
                    let mut node = Node::new(Role::Unknown);
                    node.set_bounds(Rect::new(0.0, 50.0, 30.0, 60.0));
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        crate::Tree::new(update, false)
    }

    #[test]
    fn clipped_children_excluded_above() {
        let tree = clipped_children_test_tree();
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(1));
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(2));
    }

    #[test]
    fn clipped_children_included_above() {
        let tree = clipped_children_test_tree();
        assert_filter_result(Include, &tree, SchemaNodeId(3));
    }

    #[test]
    fn clipped_children_hidden() {
        let tree = clipped_children_test_tree();
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(4));
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(8));
    }

    #[test]
    fn clipped_children_visible() {
        let tree = clipped_children_test_tree();
        assert_filter_result(Include, &tree, SchemaNodeId(5));
        assert_filter_result(Include, &tree, SchemaNodeId(6));
        assert_filter_result(Include, &tree, SchemaNodeId(7));
    }

    #[test]
    fn clipped_children_included_below() {
        let tree = clipped_children_test_tree();
        assert_filter_result(Include, &tree, SchemaNodeId(9));
    }

    #[test]
    fn clipped_children_excluded_below() {
        let tree = clipped_children_test_tree();
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(10));
        assert_filter_result(ExcludeSubtree, &tree, SchemaNodeId(11));
    }
}
