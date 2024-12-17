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
    use accesskit::{NodeId, Rect, Role, Tree, TreeUpdate};

    use super::{
        common_filter, common_filter_with_root_exception,
        FilterResult::{self, *},
    };

    #[track_caller]
    fn assert_filter_result(expected: FilterResult, tree: &crate::Tree, id: NodeId) {
        assert_eq!(
            expected,
            common_filter(&tree.state().node_by_id(id).unwrap())
        );
    }

    #[test]
    fn normal() {
        let tree = crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_filter_result(Include, &tree, NodeId(1));
    }

    #[test]
    fn hidden() {
        let tree = crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |node| {
                node.set_hidden();
            });
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_filter_result(ExcludeSubtree, &tree, NodeId(1));
    }

    #[test]
    fn hidden_but_focused() {
        let tree = crate::Tree::new(true, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |node| {
                node.set_hidden();
            });
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(1));
        });
        assert_filter_result(Include, &tree, NodeId(1));
    }

    #[test]
    fn generic_container() {
        let tree = crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::GenericContainer, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_filter_result(ExcludeNode, &tree, NodeId(0));
        assert_eq!(
            Include,
            common_filter_with_root_exception(&tree.state().node_by_id(NodeId(0)).unwrap())
        );
        assert_filter_result(Include, &tree, NodeId(1));
    }

    #[test]
    fn hidden_parent() {
        let tree = crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::GenericContainer, |node| {
                node.set_hidden();
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_filter_result(ExcludeSubtree, &tree, NodeId(0));
        assert_filter_result(ExcludeSubtree, &tree, NodeId(1));
    }

    #[test]
    fn hidden_parent_but_focused() {
        let tree = crate::Tree::new(true, |update| {
            update.set_node(NodeId(0), Role::GenericContainer, |node| {
                node.set_hidden();
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(1));
        });
        assert_filter_result(ExcludeSubtree, &tree, NodeId(0));
        assert_filter_result(Include, &tree, NodeId(1));
    }

    #[test]
    fn text_run() {
        let tree = crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::TextInput, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::TextRun, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_filter_result(ExcludeNode, &tree, NodeId(1));
    }

    fn clipped_children_test_tree() -> crate::Tree {
        crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::ScrollView, |node| {
                node.set_clips_children();
                node.set_bounds(Rect::new(0.0, 0.0, 30.0, 30.0));
                node.set_children(&[
                    NodeId(1),
                    NodeId(2),
                    NodeId(3),
                    NodeId(4),
                    NodeId(5),
                    NodeId(6),
                    NodeId(7),
                    NodeId(8),
                    NodeId(9),
                    NodeId(10),
                    NodeId(11),
                ]);
            });
            update.set_node(NodeId(1), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, -30.0, 30.0, -20.0));
            });
            update.set_node(NodeId(2), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, -20.0, 30.0, -10.0));
            });
            update.set_node(NodeId(3), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, -10.0, 30.0, 0.0));
            });
            update.set_node(NodeId(4), Role::Unknown, |node| {
                node.set_hidden();
            });
            update.set_node(NodeId(5), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, 0.0, 30.0, 10.0));
            });
            update.set_node(NodeId(6), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, 10.0, 30.0, 20.0));
            });
            update.set_node(NodeId(7), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, 20.0, 30.0, 30.0));
            });
            update.set_node(NodeId(8), Role::Unknown, |node| {
                node.set_hidden();
            });
            update.set_node(NodeId(9), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, 30.0, 30.0, 40.0));
            });
            update.set_node(NodeId(10), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, 40.0, 30.0, 50.0));
            });
            update.set_node(NodeId(11), Role::Unknown, |node| {
                node.set_bounds(Rect::new(0.0, 50.0, 30.0, 60.0));
            });
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        })
    }

    #[test]
    fn clipped_children_excluded_above() {
        let tree = clipped_children_test_tree();
        assert_filter_result(ExcludeSubtree, &tree, NodeId(1));
        assert_filter_result(ExcludeSubtree, &tree, NodeId(2));
    }

    #[test]
    fn clipped_children_included_above() {
        let tree = clipped_children_test_tree();
        assert_filter_result(Include, &tree, NodeId(3));
    }

    #[test]
    fn clipped_children_hidden() {
        let tree = clipped_children_test_tree();
        assert_filter_result(ExcludeSubtree, &tree, NodeId(4));
        assert_filter_result(ExcludeSubtree, &tree, NodeId(8));
    }

    #[test]
    fn clipped_children_visible() {
        let tree = clipped_children_test_tree();
        assert_filter_result(Include, &tree, NodeId(5));
        assert_filter_result(Include, &tree, NodeId(6));
        assert_filter_result(Include, &tree, NodeId(7));
    }

    #[test]
    fn clipped_children_included_below() {
        let tree = clipped_children_test_tree();
        assert_filter_result(Include, &tree, NodeId(9));
    }

    #[test]
    fn clipped_children_excluded_below() {
        let tree = clipped_children_test_tree();
        assert_filter_result(ExcludeSubtree, &tree, NodeId(10));
        assert_filter_result(ExcludeSubtree, &tree, NodeId(11));
    }
}
