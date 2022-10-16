// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2021 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use std::iter::FusedIterator;

use accesskit::kurbo::{Affine, Point, Rect};
use accesskit::{CheckedState, DefaultActionVerb, Live, Node as NodeData, NodeId, Role};

use crate::iterators::{
    FilterResult, FilteredChildren, FollowingFilteredSiblings, FollowingSiblings,
    PrecedingFilteredSiblings, PrecedingSiblings,
};
use crate::tree::{NodeState, ParentAndIndex, State as TreeState};

#[derive(Copy, Clone)]
pub struct Node<'a> {
    pub tree_state: &'a TreeState,
    pub(crate) state: &'a NodeState,
}

impl<'a> Node<'a> {
    pub(crate) fn data(&self) -> &NodeData {
        &self.state.data
    }

    pub fn is_focused(&self) -> bool {
        self.tree_state.focus == Some(self.id())
    }

    pub fn is_focusable(&self) -> bool {
        // TBD: Is it ever safe to imply this on a node that doesn't explicitly
        // specify it?
        self.data().focusable
    }

    pub fn is_root(&self) -> bool {
        // Don't check for absence of a parent node, in case a non-root node
        // somehow gets detached from the tree.
        self.id() == self.tree_state.root_id()
    }

    pub fn parent_id(&self) -> Option<NodeId> {
        self.state
            .parent_and_index
            .as_ref()
            .map(|ParentAndIndex(id, _)| *id)
    }

    pub fn parent(&self) -> Option<Node<'a>> {
        self.parent_id()
            .map(|id| self.tree_state.node_by_id(id).unwrap())
    }

    pub fn filtered_parent(&self, filter: &impl Fn(&Node) -> FilterResult) -> Option<Node<'a>> {
        if let Some(parent) = self.parent() {
            if filter(&parent) != FilterResult::Include {
                parent.filtered_parent(filter)
            } else {
                Some(parent)
            }
        } else {
            None
        }
    }

    pub fn parent_and_index(self) -> Option<(Node<'a>, usize)> {
        self.state
            .parent_and_index
            .as_ref()
            .map(|ParentAndIndex(parent, index)| {
                (self.tree_state.node_by_id(*parent).unwrap(), *index)
            })
    }

    pub fn child_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + 'a {
        let data = &self.state.data;
        data.children.iter().copied()
    }

    pub fn children(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let state = self.tree_state;
        self.child_ids()
            .map(move |id| state.node_by_id(id).unwrap())
    }

    pub fn filtered_children(
        &self,
        filter: impl Fn(&Node) -> FilterResult + 'a,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        FilteredChildren::new(*self, filter)
    }

    pub fn following_sibling_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + 'a {
        FollowingSiblings::new(*self)
    }

    pub fn following_siblings(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let state = self.tree_state;
        self.following_sibling_ids()
            .map(move |id| state.node_by_id(id).unwrap())
    }

    pub fn following_filtered_siblings(
        &self,
        filter: impl Fn(&Node) -> FilterResult + 'a,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        FollowingFilteredSiblings::new(*self, filter)
    }

    pub fn preceding_sibling_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + 'a {
        PrecedingSiblings::new(*self)
    }

    pub fn preceding_siblings(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let state = self.tree_state;
        self.preceding_sibling_ids()
            .map(move |id| state.node_by_id(id).unwrap())
    }

    pub fn preceding_filtered_siblings(
        &self,
        filter: impl Fn(&Node) -> FilterResult + 'a,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        PrecedingFilteredSiblings::new(*self, filter)
    }

    pub fn deepest_first_child(self) -> Option<Node<'a>> {
        let mut deepest_child = self.children().next()?;
        while let Some(first_child) = deepest_child.children().next() {
            deepest_child = first_child;
        }
        Some(deepest_child)
    }

    pub fn deepest_first_filtered_child(
        &self,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<Node<'a>> {
        let mut deepest_child = self.first_filtered_child(filter)?;
        while let Some(first_child) = deepest_child.first_filtered_child(filter) {
            deepest_child = first_child;
        }
        Some(deepest_child)
    }

    pub fn deepest_last_child(self) -> Option<Node<'a>> {
        let mut deepest_child = self.children().next_back()?;
        while let Some(last_child) = deepest_child.children().next_back() {
            deepest_child = last_child;
        }
        Some(deepest_child)
    }

    pub fn deepest_last_filtered_child(
        &self,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<Node<'a>> {
        let mut deepest_child = self.last_filtered_child(filter)?;
        while let Some(last_child) = deepest_child.last_filtered_child(filter) {
            deepest_child = last_child;
        }
        Some(deepest_child)
    }

    pub fn is_descendant_of(&self, ancestor: &Node) -> bool {
        if self.id() == ancestor.id() {
            return true;
        }
        if let Some(parent) = self.parent() {
            return parent.is_descendant_of(ancestor);
        }
        false
    }

    /// Returns the transform defined directly on this node, or the identity
    /// transform, without taking into account transforms on ancestors.
    pub fn direct_transform(&self) -> Affine {
        self.data()
            .transform
            .as_ref()
            .map_or(Affine::IDENTITY, |t| **t)
    }

    /// Returns the combined affine transform of this node and its ancestors,
    /// up to and including the root of this node's tree.
    pub fn transform(&self) -> Affine {
        self.parent()
            .map_or(Affine::IDENTITY, |parent| parent.transform())
            * self.direct_transform()
    }

    /// Returns the node's transformed bounding box relative to the tree's
    /// container (e.g. window).
    pub fn bounding_box(&self) -> Option<Rect> {
        self.data()
            .bounds
            .as_ref()
            .map(|rect| self.transform().transform_rect_bbox(*rect))
    }

    /// Returns the deepest filtered node, either this node or a descendant,
    /// at the given point in this node's coordinate space.
    pub fn node_at_point(
        &self,
        point: Point,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<Node<'a>> {
        let filter_result = filter(self);

        if filter_result == FilterResult::ExcludeSubtree {
            return None;
        }

        for child in self.children().rev() {
            let point = child.direct_transform().inverse() * point;
            if let Some(node) = child.node_at_point(point, filter) {
                return Some(node);
            }
        }

        if filter_result == FilterResult::Include {
            if let Some(rect) = &self.data().bounds {
                if rect.contains(point) {
                    return Some(*self);
                }
            }
        }

        None
    }

    pub fn id(&self) -> NodeId {
        self.state.id
    }

    pub fn role(&self) -> Role {
        self.data().role
    }

    pub fn is_hidden(&self) -> bool {
        self.data().hidden
    }

    pub fn is_disabled(&self) -> bool {
        self.data().disabled
    }

    pub fn is_read_only(&self) -> bool {
        let data = self.data();
        if data.read_only {
            true
        } else if !data.editable {
            false
        } else {
            self.should_have_read_only_state_by_default() || !self.is_read_only_supported()
        }
    }

    pub fn is_read_only_or_disabled(&self) -> bool {
        self.is_read_only() || self.is_disabled()
    }

    pub fn checked_state(&self) -> Option<CheckedState> {
        self.data().checked_state
    }

    pub fn value(&self) -> Option<&str> {
        self.data().value.as_deref()
    }

    pub fn numeric_value(&self) -> Option<f64> {
        self.data().numeric_value
    }

    pub fn min_numeric_value(&self) -> Option<f64> {
        self.data().min_numeric_value
    }

    pub fn max_numeric_value(&self) -> Option<f64> {
        self.data().max_numeric_value
    }

    pub fn numeric_value_step(&self) -> Option<f64> {
        self.data().numeric_value_step
    }

    pub fn numeric_value_jump(&self) -> Option<f64> {
        self.data().numeric_value_jump
    }

    pub fn is_text_field(&self) -> bool {
        self.is_atomic_text_field() || self.data().nonatomic_text_field_root
    }

    pub fn is_atomic_text_field(&self) -> bool {
        // The ARIA spec suggests a textbox is a simple text field, like an <input> or
        // <textarea> depending on aria-multiline. However there is nothing to stop
        // an author from adding the textbox role to a non-contenteditable element,
        // or from adding or removing non-plain-text nodes. If we treat the textbox
        // role as atomic when contenteditable is not set, it can break accessibility
        // by pruning interactive elements from the accessibility tree. Therefore,
        // until we have a reliable means to identify truly atomic ARIA textboxes,
        // treat them as non-atomic.
        match self.role() {
            Role::SearchBox | Role::TextField | Role::TextFieldWithComboBox => {
                !self.data().nonatomic_text_field_root
            }
            _ => false,
        }
    }

    pub fn default_action_verb(&self) -> Option<DefaultActionVerb> {
        self.data().default_action_verb
    }

    // When probing for supported actions as the next several functions do,
    // it's tempting to check the role. But it's better to not assume anything
    // beyond what the provider has explicitly told us. Rationale:
    // if the provider developer forgot to correctly set `default_action_verb`,
    // an AT (or even AccessKit itself) can fall back to simulating
    // a mouse click. But if the provider doesn't handle an action request
    // and we assume that it will based on the role, the attempted action
    // does nothing. This stance is a departure from Chromium.

    pub fn is_clickable(&self) -> bool {
        // If it has a custom default action verb except for
        // `DefaultActionVerb::ClickAncestor`, it's definitely clickable.
        // `DefaultActionVerb::ClickAncestor` is used when a node with a
        // click listener is present in its ancestry chain.
        if let Some(verb) = self.default_action_verb() {
            if verb != DefaultActionVerb::ClickAncestor {
                return true;
            }
        }

        false
    }

    pub fn supports_toggle(&self) -> bool {
        self.checked_state().is_some()
    }

    pub fn supports_expand_collapse(&self) -> bool {
        self.data().expanded.is_some()
    }

    pub fn is_invocable(&self) -> bool {
        // A control is "invocable" if it initiates an action when activated but
        // does not maintain any state. A control that maintains state
        // when activated would be considered a toggle or expand-collapse
        // control - these controls are "clickable" but not "invocable".
        // Similarly, if the action only gives the control keyboard focus,
        // such as when clicking a text field, the control is not considered
        // "invocable", as the "invoke" action would be a redundant synonym
        // for the "set focus" action. The same logic applies to selection.
        self.is_clickable()
            && !matches!(
                self.default_action_verb(),
                Some(DefaultActionVerb::Focus | DefaultActionVerb::Select)
            )
            && !self.supports_toggle()
            && !self.supports_expand_collapse()
    }

    pub fn name(&self) -> Option<String> {
        if let Some(name) = &self.data().name {
            Some(name.to_string())
        } else {
            let labelled_by = &self.data().labelled_by;
            if labelled_by.is_empty() {
                None
            } else {
                Some(
                    labelled_by
                        .iter()
                        .filter_map(|id| self.tree_state.node_by_id(*id).unwrap().name())
                        .collect::<Vec<String>>()
                        .join(" "),
                )
            }
        }
    }

    pub fn is_read_only_supported(&self) -> bool {
        matches!(
            self.role(),
            Role::CheckBox
                | Role::ColorWell
                | Role::ComboBoxGrouping
                | Role::ComboBoxMenuButton
                | Role::Date
                | Role::DateTime
                | Role::Grid
                | Role::InputTime
                | Role::ListBox
                | Role::MenuItemCheckBox
                | Role::MenuItemRadio
                | Role::MenuListPopup
                | Role::PopupButton
                | Role::RadioButton
                | Role::RadioGroup
                | Role::SearchBox
                | Role::Slider
                | Role::SpinButton
                | Role::Switch
                | Role::TextField
                | Role::TextFieldWithComboBox
                | Role::ToggleButton
                | Role::TreeGrid
        )
    }

    pub fn should_have_read_only_state_by_default(&self) -> bool {
        matches!(
            self.role(),
            Role::Article
                | Role::Definition
                | Role::DescriptionList
                | Role::DescriptionListTerm
                | Role::Directory
                | Role::Document
                | Role::GraphicsDocument
                | Role::Image
                | Role::List
                | Role::ListItem
                | Role::PdfRoot
                | Role::ProgressIndicator
                | Role::RootWebArea
                | Role::Term
                | Role::Timer
                | Role::Toolbar
                | Role::Tooltip
        )
    }

    pub fn live(&self) -> Live {
        self.data()
            .live
            .unwrap_or_else(|| self.parent().map_or(Live::Off, |parent| parent.live()))
    }

    pub fn is_selected(&self) -> Option<bool> {
        self.data().selected
    }

    pub(crate) fn first_filtered_child(
        &self,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<Node<'a>> {
        for child in self.children() {
            let result = filter(&child);
            if result == FilterResult::Include {
                return Some(child);
            }
            if result == FilterResult::ExcludeNode {
                if let Some(descendant) = child.first_filtered_child(filter) {
                    return Some(descendant);
                }
            }
        }
        None
    }

    pub(crate) fn last_filtered_child(
        &self,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<Node<'a>> {
        for child in self.children().rev() {
            let result = filter(&child);
            if result == FilterResult::Include {
                return Some(child);
            }
            if result == FilterResult::ExcludeNode {
                if let Some(descendant) = child.last_filtered_child(filter) {
                    return Some(descendant);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use accesskit::kurbo::{Point, Rect};
    use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
    use std::{num::NonZeroU128, sync::Arc};

    use crate::tests::*;

    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
    const NODE_ID_4: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
    const NODE_ID_5: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(5) });

    #[test]
    fn parent_and_index() {
        let tree = test_tree();
        assert!(tree.read().root().parent_and_index().is_none());
        assert_eq!(
            Some((ROOT_ID, 0)),
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .parent_and_index()
                .map(|(parent, index)| (parent.id(), index))
        );
        assert_eq!(
            Some((PARAGRAPH_0_ID, 0)),
            tree.read()
                .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
                .unwrap()
                .parent_and_index()
                .map(|(parent, index)| (parent.id(), index))
        );
        assert_eq!(
            Some((ROOT_ID, 1)),
            tree.read()
                .node_by_id(PARAGRAPH_1_IGNORED_ID)
                .unwrap()
                .parent_and_index()
                .map(|(parent, index)| (parent.id(), index))
        );
    }

    #[test]
    fn deepest_first_child() {
        let tree = test_tree();
        assert_eq!(
            STATIC_TEXT_0_0_IGNORED_ID,
            tree.read().root().deepest_first_child().unwrap().id()
        );
        assert_eq!(
            STATIC_TEXT_0_0_IGNORED_ID,
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .deepest_first_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .deepest_first_child()
            .is_none());
    }

    #[test]
    fn deepest_first_filtered_child() {
        let tree = test_tree();
        assert_eq!(
            PARAGRAPH_0_ID,
            tree.read()
                .root()
                .deepest_first_filtered_child(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .deepest_first_filtered_child(&test_tree_filter)
            .is_none());
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .deepest_first_filtered_child(&test_tree_filter)
            .is_none());
    }

    #[test]
    fn deepest_last_child() {
        let tree = test_tree();
        assert_eq!(
            EMPTY_CONTAINER_3_3_IGNORED_ID,
            tree.read().root().deepest_last_child().unwrap().id()
        );
        assert_eq!(
            EMPTY_CONTAINER_3_3_IGNORED_ID,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(BUTTON_3_2_ID)
            .unwrap()
            .deepest_last_child()
            .is_none());
    }

    #[test]
    fn deepest_last_filtered_child() {
        let tree = test_tree();
        assert_eq!(
            BUTTON_3_2_ID,
            tree.read()
                .root()
                .deepest_last_filtered_child(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert_eq!(
            BUTTON_3_2_ID,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_filtered_child(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(BUTTON_3_2_ID)
            .unwrap()
            .deepest_last_filtered_child(&test_tree_filter)
            .is_none());
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .deepest_last_filtered_child(&test_tree_filter)
            .is_none());
    }

    #[test]
    fn is_descendant_of() {
        let tree = test_tree();
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .is_descendant_of(&tree.read().root()));
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.read().root()));
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.read().node_by_id(PARAGRAPH_0_ID).unwrap()));
        assert!(!tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.read().node_by_id(PARAGRAPH_2_ID).unwrap()));
        assert!(!tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .is_descendant_of(&tree.read().node_by_id(PARAGRAPH_2_ID).unwrap()));
    }

    #[test]
    fn is_root() {
        let tree = test_tree();
        assert!(tree.read().node_by_id(ROOT_ID).unwrap().is_root());
        assert!(!tree.read().node_by_id(PARAGRAPH_0_ID).unwrap().is_root());
    }

    #[test]
    fn bounding_box() {
        let tree = test_tree();
        assert!(tree
            .read()
            .node_by_id(ROOT_ID)
            .unwrap()
            .bounding_box()
            .is_none());
        assert_eq!(
            Some(Rect {
                x0: 10.0,
                y0: 40.0,
                x1: 810.0,
                y1: 80.0,
            }),
            tree.read()
                .node_by_id(PARAGRAPH_1_IGNORED_ID)
                .unwrap()
                .bounding_box()
        );
        assert_eq!(
            Some(Rect {
                x0: 20.0,
                y0: 50.0,
                x1: 100.0,
                y1: 70.0,
            }),
            tree.read()
                .node_by_id(STATIC_TEXT_1_0_ID)
                .unwrap()
                .bounding_box()
        );
    }

    #[test]
    fn node_at_point() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .node_at_point(Point::new(10.0, 40.0), &test_tree_filter)
            .is_none());
        assert_eq!(
            Some(STATIC_TEXT_1_0_ID),
            tree.read()
                .root()
                .node_at_point(Point::new(20.0, 50.0), &test_tree_filter)
                .map(|node| node.id())
        );
        assert_eq!(
            Some(STATIC_TEXT_1_0_ID),
            tree.read()
                .root()
                .node_at_point(Point::new(50.0, 60.0), &test_tree_filter)
                .map(|node| node.id())
        );
        assert!(tree
            .read()
            .root()
            .node_at_point(Point::new(100.0, 70.0), &test_tree_filter)
            .is_none());
    }

    #[test]
    fn no_name_or_labelled_by() {
        let update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        role: Role::Window,
                        children: vec![NODE_ID_2],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::Button,
                        ..Default::default()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = crate::Tree::new(update, Box::new(NullActionHandler {}));
        assert_eq!(None, tree.read().node_by_id(NODE_ID_2).unwrap().name());
    }

    #[test]
    fn name_from_labelled_by() {
        // The following mock UI probably isn't very localization-friendly,
        // but it's good for this test.
        const LABEL_1: &str = "Check email every";
        const LABEL_2: &str = "minutes";

        let update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        role: Role::Window,
                        children: vec![NODE_ID_2, NODE_ID_3, NODE_ID_4, NODE_ID_5],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::CheckBox,
                        labelled_by: vec![NODE_ID_3, NODE_ID_5],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_3,
                    Arc::new(Node {
                        role: Role::StaticText,
                        name: Some(LABEL_1.into()),
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_4,
                    Arc::new(Node {
                        role: Role::CheckBox,
                        labelled_by: vec![NODE_ID_5],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_5,
                    Arc::new(Node {
                        role: Role::StaticText,
                        name: Some(LABEL_2.into()),
                        ..Default::default()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = crate::Tree::new(update, Box::new(NullActionHandler {}));
        assert_eq!(
            Some([LABEL_1, LABEL_2].join(" ")),
            tree.read().node_by_id(NODE_ID_2).unwrap().name()
        );
        assert_eq!(
            Some(LABEL_2.into()),
            tree.read().node_by_id(NODE_ID_4).unwrap().name()
        );
    }
}
