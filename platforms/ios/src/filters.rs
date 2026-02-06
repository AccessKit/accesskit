// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{FilterResult, Node};

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
pub(crate) fn filter_for_is_accessibility_element(node: &Node) -> FilterResult {
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

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::{Action, Node as NodeBuilder, NodeId, Role, Tree, TreeId, TreeUpdate};

    const ROOT_ID: NodeId = NodeId(0);
    const CHILD_1_ID: NodeId = NodeId(1);

    fn build_tree(nodes: Vec<(NodeId, NodeBuilder)>) -> accesskit_consumer::Tree {
        let update = TreeUpdate {
            nodes,
            tree: Some(Tree::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: ROOT_ID,
        };
        accesskit_consumer::Tree::new(update, false)
    }

    fn filter_node(nodes: Vec<(NodeId, NodeBuilder)>, target: NodeId) -> FilterResult {
        let tree = build_tree(nodes);
        let node = tree
            .state()
            .node_by_tree_local_id(target, TreeId::ROOT)
            .unwrap();
        filter_for_is_accessibility_element(&node)
    }

    fn make_button(label: &str) -> NodeBuilder {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_label(label);
        node.add_action(Action::Click);
        node
    }

    #[test]
    fn leaf_button_is_element() {
        let mut root = NodeBuilder::new(Role::Window);
        root.set_children(vec![CHILD_1_ID]);
        let child = make_button("OK");
        assert_eq!(
            filter_node(vec![(ROOT_ID, root), (CHILD_1_ID, child)], CHILD_1_ID),
            FilterResult::Include,
        );
    }

    #[test]
    fn hidden_node_excluded() {
        let mut root = NodeBuilder::new(Role::Window);
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
        let mut root = NodeBuilder::new(Role::Window);
        root.set_children(vec![CHECKBOX_ID]);
        let mut checkbox = NodeBuilder::new(Role::CheckBox);
        checkbox.add_action(Action::Click);
        checkbox.set_children(vec![LABEL_ID]);
        let mut label = NodeBuilder::new(Role::Label);
        label.set_value("Accept terms");
        assert_eq!(
            filter_node(
                vec![(ROOT_ID, root), (CHECKBOX_ID, checkbox), (LABEL_ID, label),],
                CHECKBOX_ID,
            ),
            FilterResult::Include,
        );
    }
}
