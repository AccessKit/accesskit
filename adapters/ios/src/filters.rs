// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Action, Point};
use accesskit_consumer::{FilterResult, NodeRef};
use std::iter::successors;

pub(crate) use accesskit_consumer::common_filter as filter;

use crate::node::NodeWrapper;

/// VoiceOver ignores the children of an accessibility element, so a node
/// with children it must reach is a container instead.
///
/// A node with a value, a toggled state, or actions other than focus is
/// always a leaf: its descendants only contribute to its label.
pub(crate) fn filter_for_is_accessibility_element(node: &NodeRef) -> FilterResult {
    let result = filter(node);
    if result != FilterResult::Include {
        return result;
    }

    let wrapper = NodeWrapper(node);
    if wrapper.has_non_scroll_non_focus_action()
        || node.toggled().is_some()
        || node.has_value()
        || node.numeric_value().is_some()
    {
        return FilterResult::Include;
    }

    if node.supports_action(Action::Focus, &filter) {
        return if has_actionable_or_container_child(node) {
            FilterResult::ExcludeNode
        } else {
            FilterResult::Include
        };
    }

    if has_focusable_child_or_any_grandchild(node) {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

fn has_actionable_or_container_child(node: &NodeRef) -> bool {
    node.filtered_children(&filter).any(|child| {
        NodeWrapper(&child).has_non_scroll_action()
            || filter_for_is_accessibility_element(&child) == FilterResult::ExcludeNode
    })
}

fn has_focusable_child_or_any_grandchild(node: &NodeRef) -> bool {
    node.filtered_children(&filter).any(|child| {
        NodeWrapper(&child).can_be_focused() || child.filtered_children(&filter).next().is_some()
    })
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
        build_tree_with_focus(nodes, None)
    }

    fn build_tree_with_focus(
        nodes: Vec<(NodeId, Node)>,
        focus: Option<NodeId>,
    ) -> accesskit_consumer::Tree {
        let update = TreeUpdate {
            nodes,
            tree: Some(TreeInfo::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: focus.unwrap_or(ROOT_ID),
        };
        accesskit_consumer::Tree::new(update, focus.is_some())
    }

    fn filter_node(nodes: Vec<(NodeId, Node)>, target: NodeId) -> FilterResult {
        filter_focused_node(nodes, None, target)
    }

    fn filter_focused_node(
        nodes: Vec<(NodeId, Node)>,
        focus: Option<NodeId>,
        target: NodeId,
    ) -> FilterResult {
        let tree = build_tree_with_focus(nodes, focus);
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

    const LIST_BOX_ID: NodeId = NodeId(1);
    const OPTION_1_ID: NodeId = NodeId(2);
    const OPTION_2_ID: NodeId = NodeId(3);

    fn focusable_list_box(active_descendant: Option<NodeId>) -> Vec<(NodeId, Node)> {
        let mut root = Node::new(Role::Window);
        root.set_bounds(Rect::new(0.0, 0.0, 200.0, 200.0));
        root.set_children(vec![LIST_BOX_ID]);
        let mut list_box = Node::new(Role::ListBox);
        list_box.set_label("List");
        list_box.add_action(Action::Focus);
        list_box.set_bounds(Rect::new(0.0, 0.0, 200.0, 100.0));
        list_box.set_children(vec![OPTION_1_ID, OPTION_2_ID]);
        if let Some(id) = active_descendant {
            list_box.set_active_descendant(id);
        }
        let mut option_1 = Node::new(Role::ListBoxOption);
        option_1.set_label("Option 1");
        option_1.add_action(Action::Click);
        option_1.set_bounds(Rect::new(0.0, 0.0, 100.0, 50.0));
        let mut option_2 = Node::new(Role::ListBoxOption);
        option_2.set_label("Option 2");
        option_2.add_action(Action::Click);
        option_2.set_bounds(Rect::new(0.0, 50.0, 100.0, 100.0));
        vec![
            (ROOT_ID, root),
            (LIST_BOX_ID, list_box),
            (OPTION_1_ID, option_1),
            (OPTION_2_ID, option_2),
        ]
    }

    #[test]
    fn focusable_list_box_is_container() {
        assert_eq!(
            filter_node(focusable_list_box(None), LIST_BOX_ID),
            FilterResult::ExcludeNode,
        );
    }

    #[test]
    fn focused_list_box_with_active_descendant_is_container() {
        assert_eq!(
            filter_focused_node(
                focusable_list_box(Some(OPTION_1_ID)),
                Some(LIST_BOX_ID),
                LIST_BOX_ID,
            ),
            FilterResult::ExcludeNode,
        );
    }

    #[test]
    fn focusable_node_with_label_child_is_leaf() {
        const GROUP_ID: NodeId = NodeId(1);
        const LABEL_ID: NodeId = NodeId(2);
        let mut root = Node::new(Role::Window);
        root.set_children(vec![GROUP_ID]);
        let mut group = Node::new(Role::Group);
        group.add_action(Action::Focus);
        group.set_children(vec![LABEL_ID]);
        let mut label = Node::new(Role::Label);
        label.set_value("Label");
        assert_eq!(
            filter_node(
                vec![(ROOT_ID, root), (GROUP_ID, group), (LABEL_ID, label)],
                GROUP_ID,
            ),
            FilterResult::Include,
        );
    }

    fn hit_test_focusable_list_box(point: Point) -> Option<NodeId> {
        let tree = build_tree(focusable_list_box(None));
        element_at_point(tree.state().root(), point).map(|node| node.locate().0)
    }

    #[test]
    fn hit_test_finds_child_of_focusable_container() {
        assert_eq!(
            hit_test_focusable_list_box(Point::new(50.0, 75.0)),
            Some(OPTION_2_ID),
        );
    }

    #[test]
    fn hit_test_skips_focusable_container() {
        assert_eq!(hit_test_focusable_list_box(Point::new(150.0, 50.0)), None);
    }
}
