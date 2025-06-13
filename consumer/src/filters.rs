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

#[cfg(test)]
mod tests {
    use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
    use alloc::vec;

    use super::{common_filter, common_filter_with_root_exception, FilterResult};

    #[test]
    fn normal() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            FilterResult::Include,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }

    #[test]
    fn hidden() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::Button);
                    node.set_hidden();
                    node
                }),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            FilterResult::ExcludeSubtree,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }

    #[test]
    fn hidden_but_focused() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::Button);
                    node.set_hidden();
                    node
                }),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(1),
        };
        let tree = crate::Tree::new(update, true);
        assert_eq!(
            FilterResult::Include,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }

    #[test]
    fn generic_container() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            FilterResult::ExcludeNode,
            common_filter(&tree.state().node_by_id(NodeId(0)).unwrap())
        );
        assert_eq!(
            FilterResult::Include,
            common_filter_with_root_exception(&tree.state().node_by_id(NodeId(0)).unwrap())
        );
        assert_eq!(
            FilterResult::Include,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }

    #[test]
    fn hidden_parent() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_hidden();
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            FilterResult::ExcludeSubtree,
            common_filter(&tree.state().node_by_id(NodeId(0)).unwrap())
        );
        assert_eq!(
            FilterResult::ExcludeSubtree,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }

    #[test]
    fn hidden_parent_but_focused() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_hidden();
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(1),
        };
        let tree = crate::Tree::new(update, true);
        assert_eq!(
            FilterResult::ExcludeSubtree,
            common_filter(&tree.state().node_by_id(NodeId(0)).unwrap())
        );
        assert_eq!(
            FilterResult::Include,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }

    #[test]
    fn text_run() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::TextInput);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::TextRun)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            FilterResult::ExcludeNode,
            common_filter(&tree.state().node_by_id(NodeId(1)).unwrap())
        );
    }
}
