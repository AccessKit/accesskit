// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2021 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use std::{iter::FusedIterator, ops::Deref};

use accesskit::{
    Action, Affine, CheckedState, DefaultActionVerb, Live, Node as NodeData, NodeId, Point, Rect,
    Role, TextSelection,
};

use crate::iterators::{
    FilterResult, FilteredChildren, FollowingFilteredSiblings, FollowingSiblings, LabelledBy,
    PrecedingFilteredSiblings, PrecedingSiblings,
};
use crate::tree::State as TreeState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParentAndIndex(pub(crate) NodeId, pub(crate) usize);

#[derive(Clone)]
pub struct NodeState {
    pub(crate) id: NodeId,
    pub(crate) parent_and_index: Option<ParentAndIndex>,
    pub(crate) data: NodeData,
}

#[derive(Copy, Clone)]
pub struct Node<'a> {
    pub tree_state: &'a TreeState,
    pub(crate) state: &'a NodeState,
}

impl NodeState {
    pub(crate) fn data(&self) -> &NodeData {
        &self.data
    }
}

impl<'a> Node<'a> {
    pub fn detached(&self) -> DetachedNode {
        DetachedNode {
            state: self.state.clone(),
            is_focused: self.is_focused(),
            is_root: self.is_root(),
            name: self.name(),
            live: self.live(),
            supports_text_ranges: self.supports_text_ranges(),
        }
    }

    pub fn is_focused(&self) -> bool {
        self.tree_state.focus == Some(self.id())
    }
}

impl NodeState {
    pub fn is_focusable(&self) -> bool {
        self.supports_action(Action::Focus)
    }
}

impl<'a> Node<'a> {
    pub fn is_root(&self) -> bool {
        // Don't check for absence of a parent node, in case a non-root node
        // somehow gets detached from the tree.
        self.id() == self.tree_state.root_id()
    }
}

impl NodeState {
    pub fn parent_id(&self) -> Option<NodeId> {
        self.parent_and_index
            .as_ref()
            .map(|ParentAndIndex(id, _)| *id)
    }
}

impl<'a> Node<'a> {
    pub fn parent(&self) -> Option<Node<'a>> {
        self.parent_id()
            .map(|id| self.tree_state.node_by_id(id).unwrap())
    }

    pub fn filtered_parent(&self, filter: &impl Fn(&Node) -> FilterResult) -> Option<Node<'a>> {
        self.parent().and_then(move |parent| {
            if filter(&parent) == FilterResult::Include {
                Some(parent)
            } else {
                parent.filtered_parent(filter)
            }
        })
    }

    pub fn parent_and_index(self) -> Option<(Node<'a>, usize)> {
        self.state
            .parent_and_index
            .as_ref()
            .map(|ParentAndIndex(parent, index)| {
                (self.tree_state.node_by_id(*parent).unwrap(), *index)
            })
    }
}

impl NodeState {
    pub fn child_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + '_ {
        let data = &self.data;
        data.children().iter().copied()
    }
}

impl<'a> Node<'a> {
    pub fn children(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let state = self.tree_state;
        self.state
            .child_ids()
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
}

impl NodeState {
    /// Returns the transform defined directly on this node, or the identity
    /// transform, without taking into account transforms on ancestors.
    pub fn direct_transform(&self) -> Affine {
        self.data()
            .transform()
            .map_or(Affine::IDENTITY, |value| *value)
    }
}

impl<'a> Node<'a> {
    /// Returns the combined affine transform of this node and its ancestors,
    /// up to and including the root of this node's tree.
    pub fn transform(&self) -> Affine {
        self.parent()
            .map_or(Affine::IDENTITY, |parent| parent.transform())
            * self.direct_transform()
    }

    pub(crate) fn relative_transform(&self, stop_at: &Node) -> Affine {
        let parent_transform = if let Some(parent) = self.parent() {
            if parent.id() == stop_at.id() {
                Affine::IDENTITY
            } else {
                parent.relative_transform(stop_at)
            }
        } else {
            Affine::IDENTITY
        };
        parent_transform * self.direct_transform()
    }
}

impl NodeState {
    pub fn raw_bounds(&self) -> Option<Rect> {
        self.data().bounds()
    }
}

impl<'a> Node<'a> {
    pub fn has_bounds(&self) -> bool {
        self.state.raw_bounds().is_some()
    }

    /// Returns the node's transformed bounding box relative to the tree's
    /// container (e.g. window).
    pub fn bounding_box(&self) -> Option<Rect> {
        self.state
            .raw_bounds()
            .as_ref()
            .map(|rect| self.transform().transform_rect_bbox(*rect))
    }

    pub(crate) fn bounding_box_in_coordinate_space(&self, other: &Node) -> Option<Rect> {
        self.state
            .raw_bounds()
            .as_ref()
            .map(|rect| self.relative_transform(other).transform_rect_bbox(*rect))
    }

    pub(crate) fn hit_test(
        &self,
        point: Point,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<(Node<'a>, Point)> {
        let filter_result = filter(self);

        if filter_result == FilterResult::ExcludeSubtree {
            return None;
        }

        for child in self.children().rev() {
            let point = child.direct_transform().inverse() * point;
            if let Some(result) = child.hit_test(point, filter) {
                return Some(result);
            }
        }

        if filter_result == FilterResult::Include {
            if let Some(rect) = &self.state.raw_bounds() {
                if rect.contains(point) {
                    return Some((*self, point));
                }
            }
        }

        None
    }

    /// Returns the deepest filtered node, either this node or a descendant,
    /// at the given point in this node's coordinate space.
    pub fn node_at_point(
        &self,
        point: Point,
        filter: &impl Fn(&Node) -> FilterResult,
    ) -> Option<Node<'a>> {
        self.hit_test(point, filter).map(|(node, _)| node)
    }
}

impl NodeState {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn role(&self) -> Role {
        self.data().role()
    }

    pub fn is_hidden(&self) -> bool {
        self.data().is_hidden()
    }

    pub fn is_disabled(&self) -> bool {
        self.data().is_disabled()
    }

    pub fn is_read_only(&self) -> bool {
        let data = self.data();
        if data.is_read_only() {
            true
        } else if !data.is_editable() {
            false
        } else {
            self.should_have_read_only_state_by_default() || !self.is_read_only_supported()
        }
    }

    pub fn is_read_only_or_disabled(&self) -> bool {
        self.is_read_only() || self.is_disabled()
    }

    pub fn checked_state(&self) -> Option<CheckedState> {
        self.data().checked_state()
    }

    pub fn value(&self) -> Option<&str> {
        self.data().value()
    }

    pub fn numeric_value(&self) -> Option<f64> {
        self.data().numeric_value()
    }

    pub fn min_numeric_value(&self) -> Option<f64> {
        self.data().min_numeric_value()
    }

    pub fn max_numeric_value(&self) -> Option<f64> {
        self.data().max_numeric_value()
    }

    pub fn numeric_value_step(&self) -> Option<f64> {
        self.data().numeric_value_step()
    }

    pub fn numeric_value_jump(&self) -> Option<f64> {
        self.data().numeric_value_jump()
    }

    pub fn is_text_field(&self) -> bool {
        self.is_atomic_text_field() || self.data().is_nonatomic_text_field_root()
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
                !self.data().is_nonatomic_text_field_root()
            }
            _ => false,
        }
    }

    pub fn is_multiline(&self) -> bool {
        self.data().is_multiline()
    }

    pub fn is_protected(&self) -> bool {
        self.data().is_protected()
    }

    pub fn default_action_verb(&self) -> Option<DefaultActionVerb> {
        self.data().default_action_verb()
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
        self.data().is_expanded().is_some()
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

    // The future of the `Action` enum is undecided, so keep the following
    // function private for now.
    fn supports_action(&self, action: Action) -> bool {
        self.data().supports_action(action)
    }

    pub fn supports_increment(&self) -> bool {
        self.supports_action(Action::Increment)
    }

    pub fn supports_decrement(&self) -> bool {
        self.supports_action(Action::Decrement)
    }
}

fn descendant_label_filter(node: &Node) -> FilterResult {
    match node.role() {
        Role::StaticText | Role::Image => FilterResult::Include,
        Role::GenericContainer => FilterResult::ExcludeNode,
        _ => FilterResult::ExcludeSubtree,
    }
}

impl<'a> Node<'a> {
    pub fn labelled_by(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        let explicit = &self.state.data.labelled_by();
        if explicit.is_empty() && matches!(self.role(), Role::Button | Role::Link) {
            LabelledBy::FromDescendants(FilteredChildren::new(*self, &descendant_label_filter))
        } else {
            LabelledBy::Explicit {
                ids: explicit.iter(),
                tree_state: self.tree_state,
            }
        }
    }

    pub fn name(&self) -> Option<String> {
        if let Some(name) = &self.data().name() {
            Some(name.to_string())
        } else {
            let names = self
                .labelled_by()
                .filter_map(|node| node.name())
                .collect::<Vec<String>>();
            (!names.is_empty()).then(move || names.join(" "))
        }
    }
}

impl NodeState {
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
}

impl<'a> Node<'a> {
    pub fn live(&self) -> Live {
        self.data()
            .live()
            .unwrap_or_else(|| self.parent().map_or(Live::Off, |parent| parent.live()))
    }
}

impl NodeState {
    pub fn is_selected(&self) -> Option<bool> {
        self.data().is_selected()
    }

    pub fn raw_text_selection(&self) -> Option<&TextSelection> {
        self.data().text_selection()
    }
}

impl<'a> Node<'a> {
    pub fn index_path(&self) -> Vec<usize> {
        self.relative_index_path(self.tree_state.root_id())
    }

    pub fn relative_index_path(&self, ancestor_id: NodeId) -> Vec<usize> {
        let mut result = Vec::new();
        let mut current = *self;
        while current.id() != ancestor_id {
            let (parent, index) = current.parent_and_index().unwrap();
            result.push(index);
            current = parent;
        }
        result.reverse();
        result
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

    pub fn state(&self) -> &'a NodeState {
        self.state
    }
}

impl<'a> Deref for Node<'a> {
    type Target = NodeState;

    fn deref(&self) -> &NodeState {
        self.state
    }
}

#[derive(Clone)]
pub struct DetachedNode {
    pub(crate) state: NodeState,
    pub(crate) is_focused: bool,
    pub(crate) is_root: bool,
    pub(crate) name: Option<String>,
    pub(crate) live: Live,
    pub(crate) supports_text_ranges: bool,
}

impl DetachedNode {
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn is_root(&self) -> bool {
        self.is_root
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn live(&self) -> Live {
        self.live
    }

    pub fn supports_text_ranges(&self) -> bool {
        self.supports_text_ranges
    }

    pub fn state(&self) -> &NodeState {
        &self.state
    }
}

impl Deref for DetachedNode {
    type Target = NodeState;

    fn deref(&self) -> &NodeState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use accesskit::{NodeBuilder, NodeClassSet, NodeId, Point, Rect, Role, Tree, TreeUpdate};
    use std::num::NonZeroU128;

    use crate::tests::*;

    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
    const NODE_ID_4: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
    const NODE_ID_5: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(5) });
    const NODE_ID_6: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(6) });

    #[test]
    fn parent_and_index() {
        let tree = test_tree();
        assert!(tree.state().root().parent_and_index().is_none());
        assert_eq!(
            Some((ROOT_ID, 0)),
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .parent_and_index()
                .map(|(parent, index)| (parent.id(), index))
        );
        assert_eq!(
            Some((PARAGRAPH_0_ID, 0)),
            tree.state()
                .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
                .unwrap()
                .parent_and_index()
                .map(|(parent, index)| (parent.id(), index))
        );
        assert_eq!(
            Some((ROOT_ID, 1)),
            tree.state()
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
            tree.state().root().deepest_first_child().unwrap().id()
        );
        assert_eq!(
            STATIC_TEXT_0_0_IGNORED_ID,
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .deepest_first_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .state()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .deepest_first_child()
            .is_none());
    }

    #[test]
    fn filtered_parent() {
        let tree = test_tree();
        assert_eq!(
            ROOT_ID,
            tree.state()
                .node_by_id(STATIC_TEXT_1_0_ID)
                .unwrap()
                .filtered_parent(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert!(tree
            .state()
            .root()
            .filtered_parent(&test_tree_filter)
            .is_none());
    }

    #[test]
    fn deepest_first_filtered_child() {
        let tree = test_tree();
        assert_eq!(
            PARAGRAPH_0_ID,
            tree.state()
                .root()
                .deepest_first_filtered_child(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .deepest_first_filtered_child(&test_tree_filter)
            .is_none());
        assert!(tree
            .state()
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
            tree.state().root().deepest_last_child().unwrap().id()
        );
        assert_eq!(
            EMPTY_CONTAINER_3_3_IGNORED_ID,
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .state()
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
            tree.state()
                .root()
                .deepest_last_filtered_child(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert_eq!(
            BUTTON_3_2_ID,
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_filtered_child(&test_tree_filter)
                .unwrap()
                .id()
        );
        assert!(tree
            .state()
            .node_by_id(BUTTON_3_2_ID)
            .unwrap()
            .deepest_last_filtered_child(&test_tree_filter)
            .is_none());
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .deepest_last_filtered_child(&test_tree_filter)
            .is_none());
    }

    #[test]
    fn is_descendant_of() {
        let tree = test_tree();
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .is_descendant_of(&tree.state().root()));
        assert!(tree
            .state()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.state().root()));
        assert!(tree
            .state()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.state().node_by_id(PARAGRAPH_0_ID).unwrap()));
        assert!(!tree
            .state()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.state().node_by_id(PARAGRAPH_2_ID).unwrap()));
        assert!(!tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .is_descendant_of(&tree.state().node_by_id(PARAGRAPH_2_ID).unwrap()));
    }

    #[test]
    fn is_root() {
        let tree = test_tree();
        assert!(tree.state().node_by_id(ROOT_ID).unwrap().is_root());
        assert!(!tree.state().node_by_id(PARAGRAPH_0_ID).unwrap().is_root());
    }

    #[test]
    fn bounding_box() {
        let tree = test_tree();
        assert!(tree
            .state()
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
            tree.state()
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
            tree.state()
                .node_by_id(STATIC_TEXT_1_0_ID)
                .unwrap()
                .bounding_box()
        );
    }

    #[test]
    fn node_at_point() {
        let tree = test_tree();
        assert!(tree
            .state()
            .root()
            .node_at_point(Point::new(10.0, 40.0), &test_tree_filter)
            .is_none());
        assert_eq!(
            Some(STATIC_TEXT_1_0_ID),
            tree.state()
                .root()
                .node_at_point(Point::new(20.0, 50.0), &test_tree_filter)
                .map(|node| node.id())
        );
        assert_eq!(
            Some(STATIC_TEXT_1_0_ID),
            tree.state()
                .root()
                .node_at_point(Point::new(50.0, 60.0), &test_tree_filter)
                .map(|node| node.id())
        );
        assert!(tree
            .state()
            .root()
            .node_at_point(Point::new(100.0, 70.0), &test_tree_filter)
            .is_none());
    }

    #[test]
    fn no_name_or_labelled_by() {
        let mut classes = NodeClassSet::new();
        let update = TreeUpdate {
            nodes: vec![
                (NODE_ID_1, {
                    let mut builder = NodeBuilder::new(Role::Window);
                    builder.set_children(vec![NODE_ID_2]);
                    builder.build(&mut classes)
                }),
                (
                    NODE_ID_2,
                    NodeBuilder::new(Role::Button).build(&mut classes),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = crate::Tree::new(update);
        assert_eq!(None, tree.state().node_by_id(NODE_ID_2).unwrap().name());
    }

    #[test]
    fn name_from_labelled_by() {
        // The following mock UI probably isn't very localization-friendly,
        // but it's good for this test.
        const LABEL_1: &str = "Check email every";
        const LABEL_2: &str = "minutes";

        let mut classes = NodeClassSet::new();
        let update = TreeUpdate {
            nodes: vec![
                (NODE_ID_1, {
                    let mut builder = NodeBuilder::new(Role::Window);
                    builder.set_children(vec![NODE_ID_2, NODE_ID_3, NODE_ID_4, NODE_ID_5]);
                    builder.build(&mut classes)
                }),
                (NODE_ID_2, {
                    let mut builder = NodeBuilder::new(Role::CheckBox);
                    builder.set_labelled_by(vec![NODE_ID_3, NODE_ID_5]);
                    builder.build(&mut classes)
                }),
                (NODE_ID_3, {
                    let mut builder = NodeBuilder::new(Role::StaticText);
                    builder.set_name(LABEL_1);
                    builder.build(&mut classes)
                }),
                (NODE_ID_4, {
                    let mut builder = NodeBuilder::new(Role::TextField);
                    builder.push_labelled_by(NODE_ID_5);
                    builder.build(&mut classes)
                }),
                (NODE_ID_5, {
                    let mut builder = NodeBuilder::new(Role::StaticText);
                    builder.set_name(LABEL_2);
                    builder.build(&mut classes)
                }),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = crate::Tree::new(update);
        assert_eq!(
            Some([LABEL_1, LABEL_2].join(" ")),
            tree.state().node_by_id(NODE_ID_2).unwrap().name()
        );
        assert_eq!(
            Some(LABEL_2.into()),
            tree.state().node_by_id(NODE_ID_4).unwrap().name()
        );
    }

    #[test]
    fn name_from_descendant_label() {
        const BUTTON_LABEL: &str = "Play";
        const LINK_LABEL: &str = "Watch in browser";

        let mut classes = NodeClassSet::new();
        let update = TreeUpdate {
            nodes: vec![
                (NODE_ID_1, {
                    let mut builder = NodeBuilder::new(Role::Window);
                    builder.set_children(vec![NODE_ID_2, NODE_ID_4]);
                    builder.build(&mut classes)
                }),
                (NODE_ID_2, {
                    let mut builder = NodeBuilder::new(Role::Button);
                    builder.push_child(NODE_ID_3);
                    builder.build(&mut classes)
                }),
                (NODE_ID_3, {
                    let mut builder = NodeBuilder::new(Role::Image);
                    builder.set_name(BUTTON_LABEL);
                    builder.build(&mut classes)
                }),
                (NODE_ID_4, {
                    let mut builder = NodeBuilder::new(Role::Link);
                    builder.push_child(NODE_ID_5);
                    builder.build(&mut classes)
                }),
                (NODE_ID_5, {
                    let mut builder = NodeBuilder::new(Role::GenericContainer);
                    builder.push_child(NODE_ID_6);
                    builder.build(&mut classes)
                }),
                (NODE_ID_6, {
                    let mut builder = NodeBuilder::new(Role::StaticText);
                    builder.set_name(LINK_LABEL);
                    builder.build(&mut classes)
                }),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = crate::Tree::new(update);
        assert_eq!(
            Some(BUTTON_LABEL.into()),
            tree.state().node_by_id(NODE_ID_2).unwrap().name()
        );
        assert_eq!(
            Some(LINK_LABEL.into()),
            tree.state().node_by_id(NODE_ID_4).unwrap().name()
        );
    }
}
