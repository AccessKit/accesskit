// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::{FilterResult, NodeRef};
use std::iter::successors;

pub(crate) use accesskit_consumer::common_filter as filter;

use crate::node::NodeWrapper;

/// Filter for determining if a node should be an accessibility element.
/// On iOS, a node with focusable children must NOT be an accessibility element
/// (must return false for isAccessibilityElement), otherwise VoiceOver will
/// ignore its children entirely.
///
/// A node that has its own interaction semantics — actions, a value, or a
/// toggled state — is always a leaf accessibility element. Its descendants
/// (e.g. a `Role::Label` child) are collapsed into the parent's
/// `accessibilityLabel` via `accesskit_consumer::Node::labelled_by`, so
/// exposing them separately would let VoiceOver focus the label and skip
/// the actionable parent.
///
/// Otherwise, non-focusable children (e.g. Labels, Images) just provide
/// labeling info to the parent, so the parent should remain the
/// accessibility element.
pub(crate) fn filter_for_is_accessibility_element(node: &NodeRef) -> FilterResult {
    let result = filter(node);
    if result != FilterResult::Include {
        return result;
    }

    let wrapper = NodeWrapper(node);
    if wrapper.has_non_scroll_action()
        || node.toggled().is_some()
        || node.has_value()
        || node.numeric_value().is_some()
    {
        return FilterResult::Include;
    }

    // If this node has any filtered children that are focusable or are
    // themselves containers (have their own filtered children), it should be
    // a container, not an accessibility element.
    if node.filtered_children(&filter).any(|child| {
        NodeWrapper(&child).can_be_focused() || child.filtered_children(&filter).next().is_some()
    }) {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

/// Finds the accessibility element at a point, if any. Hit testing must only
/// return accessibility elements: VoiceOver reads whatever it is given, then
/// moves its cursor back to a real element.
pub(crate) fn element_at_point<'a>(root: NodeRef<'a>, point: Point) -> Option<NodeRef<'a>> {
    let hit = root.node_at_point(point, &filter).unwrap_or(root);
    successors(Some(hit), |node| node.filtered_parent(&filter))
        .filter(|node| filter_for_is_accessibility_element(node) == FilterResult::Include)
        .last()
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::{Action, Node, NodeId, Rect, Role, TreeId, TreeInfo, TreeUpdate};

    const ROOT_ID: NodeId = NodeId(0);
    const CHILD_1_ID: NodeId = NodeId(1);

    fn build_tree(nodes: Vec<(NodeId, Node)>) -> accesskit_consumer::Tree {
        let update = TreeUpdate {
            nodes,
            tree: Some(TreeInfo::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: ROOT_ID,
        };
        accesskit_consumer::Tree::new(update, false)
    }

    fn filter_node(nodes: Vec<(NodeId, Node)>, target: NodeId) -> FilterResult {
        let tree = build_tree(nodes);
        let node = tree
            .state()
            .node_by_tree_local_id(target, TreeId::ROOT)
            .unwrap();
        filter_for_is_accessibility_element(&node)
    }

    fn make_button(label: &str) -> Node {
        let mut node = Node::new(Role::Button);
        node.set_label(label);
        node.add_action(Action::Click);
        node
    }

    #[test]
    fn leaf_button_is_element() {
        let mut root = Node::new(Role::Window);
        root.set_children(vec![CHILD_1_ID]);
        let child = make_button("OK");
        assert_eq!(
            filter_node(vec![(ROOT_ID, root), (CHILD_1_ID, child)], CHILD_1_ID),
            FilterResult::Include,
        );
    }

    #[test]
    fn hidden_node_excluded() {
        let mut root = Node::new(Role::Window);
        root.set_children(vec![CHILD_1_ID]);
        let mut hidden = make_button("Hidden");
        hidden.set_hidden();
        assert_ne!(
            filter_node(vec![(ROOT_ID, root), (CHILD_1_ID, hidden)], CHILD_1_ID),
            FilterResult::Include,
        );
    }

    #[test]
    fn checkbox_with_label_child_is_leaf() {
        const CHECKBOX_ID: NodeId = NodeId(1);
        const LABEL_ID: NodeId = NodeId(2);
        let mut root = Node::new(Role::Window);
        root.set_children(vec![CHECKBOX_ID]);
        let mut checkbox = Node::new(Role::CheckBox);
        checkbox.add_action(Action::Click);
        checkbox.set_children(vec![LABEL_ID]);
        let mut label = Node::new(Role::Label);
        label.set_value("Accept terms");
        assert_eq!(
            filter_node(
                vec![(ROOT_ID, root), (CHECKBOX_ID, checkbox), (LABEL_ID, label),],
                CHECKBOX_ID,
            ),
            FilterResult::Include,
        );
    }

    const CONTAINER_ID: NodeId = NodeId(1);
    const BUTTON_ID: NodeId = NodeId(2);
    const LABEL_ID: NodeId = NodeId(3);

    // A container whose button covers only its right half, and whose button
    // gets its label from a child.
    fn hit_test(point: Point) -> Option<NodeId> {
        let mut root = Node::new(Role::Window);
        root.set_bounds(Rect::new(0.0, 0.0, 200.0, 200.0));
        root.set_children(vec![CONTAINER_ID]);
        let mut container = Node::new(Role::Group);
        container.set_bounds(Rect::new(0.0, 0.0, 200.0, 100.0));
        container.set_children(vec![BUTTON_ID]);
        let mut button = Node::new(Role::Button);
        button.add_action(Action::Click);
        button.set_bounds(Rect::new(100.0, 0.0, 200.0, 100.0));
        button.set_children(vec![LABEL_ID]);
        let mut label = Node::new(Role::Label);
        label.set_value("OK");
        label.set_bounds(Rect::new(120.0, 20.0, 180.0, 80.0));
        let tree = build_tree(vec![
            (ROOT_ID, root),
            (CONTAINER_ID, container),
            (BUTTON_ID, button),
            (LABEL_ID, label),
        ]);
        element_at_point(tree.state().root(), point).map(|node| node.locate().0)
    }

    #[test]
    fn hit_test_skips_containers() {
        assert_eq!(hit_test(Point::new(50.0, 50.0)), None);
    }

    #[test]
    fn hit_test_finds_leaf() {
        assert_eq!(hit_test(Point::new(110.0, 50.0)), Some(BUTTON_ID));
    }

    #[test]
    fn hit_test_skips_children_of_leaf() {
        assert_eq!(hit_test(Point::new(150.0, 50.0)), Some(BUTTON_ID));
    }
}
