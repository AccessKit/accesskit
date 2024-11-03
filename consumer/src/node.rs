// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2021 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{
    Action, Affine, FrozenNode as NodeData, Live, NodeId, Orientation, Point, Rect, Role,
    TextSelection, Toggled,
};
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::iter::FusedIterator;

use crate::filters::FilterResult;
use crate::iterators::{
    FilteredChildren, FollowingFilteredSiblings, FollowingSiblings, LabelledBy,
    PrecedingFilteredSiblings, PrecedingSiblings,
};
use crate::tree::State as TreeState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParentAndIndex(pub(crate) NodeId, pub(crate) usize);

#[derive(Clone)]
pub(crate) struct NodeState {
    pub(crate) parent_and_index: Option<ParentAndIndex>,
    pub(crate) data: Arc<NodeData>,
}

#[derive(Copy, Clone)]
pub struct Node<'a> {
    pub tree_state: &'a TreeState,
    pub(crate) id: NodeId,
    pub(crate) state: &'a NodeState,
}

impl<'a> Node<'a> {
    pub(crate) fn data(&self) -> &NodeData {
        &self.state.data
    }

    pub fn is_focused(&self) -> bool {
        self.tree_state.focus_id() == Some(self.id())
    }

    pub fn is_focused_in_tree(&self) -> bool {
        self.tree_state.focus == self.id()
    }

    pub fn is_focusable(&self) -> bool {
        self.supports_action(Action::Focus) || self.is_focused_in_tree()
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

    pub fn child_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + '_ {
        let data = &self.state.data;
        data.children().iter().copied()
    }

    pub fn children(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let state = self.tree_state;
        let data = &self.state.data;
        data.children()
            .iter()
            .map(move |id| state.node_by_id(*id).unwrap())
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
            .transform()
            .map_or(Affine::IDENTITY, |value| *value)
    }

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

    pub fn raw_bounds(&self) -> Option<Rect> {
        self.data().bounds()
    }

    pub fn has_bounds(&self) -> bool {
        self.raw_bounds().is_some()
    }

    /// Returns the node's transformed bounding box relative to the tree's
    /// container (e.g. window).
    pub fn bounding_box(&self) -> Option<Rect> {
        self.raw_bounds()
            .as_ref()
            .map(|rect| self.transform().transform_rect_bbox(*rect))
    }

    pub(crate) fn bounding_box_in_coordinate_space(&self, other: &Node) -> Option<Rect> {
        self.raw_bounds()
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
            if let Some(rect) = &self.raw_bounds() {
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

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn role(&self) -> Role {
        self.data().role()
    }

    pub fn role_description(&self) -> Option<String> {
        self.data().role_description().map(String::from)
    }

    pub fn has_role_description(&self) -> bool {
        self.data().role_description().is_some()
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
        } else {
            self.should_have_read_only_state_by_default() || !self.is_read_only_supported()
        }
    }

    pub fn is_read_only_or_disabled(&self) -> bool {
        self.is_read_only() || self.is_disabled()
    }

    pub fn toggled(&self) -> Option<Toggled> {
        self.data().toggled()
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

    pub fn is_text_input(&self) -> bool {
        matches!(
            self.role(),
            Role::TextInput
                | Role::MultilineTextInput
                | Role::SearchInput
                | Role::DateInput
                | Role::DateTimeInput
                | Role::WeekInput
                | Role::MonthInput
                | Role::TimeInput
                | Role::EmailInput
                | Role::NumberInput
                | Role::PasswordInput
                | Role::PhoneNumberInput
                | Role::UrlInput
                | Role::EditableComboBox
                | Role::SpinButton
        )
    }

    pub fn is_multiline(&self) -> bool {
        self.role() == Role::MultilineTextInput
    }

    pub fn orientation(&self) -> Option<Orientation> {
        self.data().orientation()
    }

    // When probing for supported actions as the next several functions do,
    // it's tempting to check the role. But it's better to not assume anything
    // beyond what the provider has explicitly told us. Rationale:
    // if the provider developer forgot to call `add_action` for an action,
    // an AT (or even AccessKit itself) can fall back to simulating
    // a mouse click. But if the provider doesn't handle an action request
    // and we assume that it will based on the role, the attempted action
    // does nothing. This stance is a departure from Chromium.

    pub fn is_clickable(&self) -> bool {
        self.supports_action(Action::Click)
    }

    pub fn supports_toggle(&self) -> bool {
        self.toggled().is_some()
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
        // such as when clicking a text input, the control is not considered
        // "invocable", as the "invoke" action would be a redundant synonym
        // for the "set focus" action. The same logic applies to selection.
        self.is_clickable()
            && !self.is_text_input()
            && !matches!(self.role(), Role::Document | Role::Terminal)
            && !self.supports_toggle()
            && !self.supports_expand_collapse()
            && self.is_selected().is_none()
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
        Role::Label | Role::Image => FilterResult::Include,
        Role::GenericContainer => FilterResult::ExcludeNode,
        _ => FilterResult::ExcludeSubtree,
    }
}

impl<'a> Node<'a> {
    pub fn labelled_by(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        let explicit = &self.state.data.labelled_by();
        if explicit.is_empty()
            && matches!(
                self.role(),
                Role::Button
                    | Role::CheckBox
                    | Role::DefaultButton
                    | Role::Link
                    | Role::MenuItem
                    | Role::MenuItemCheckBox
                    | Role::MenuItemRadio
                    | Role::RadioButton
            )
        {
            LabelledBy::FromDescendants(FilteredChildren::new(*self, &descendant_label_filter))
        } else {
            LabelledBy::Explicit {
                ids: explicit.iter(),
                tree_state: self.tree_state,
            }
        }
    }

    pub fn label_comes_from_value(&self) -> bool {
        self.role() == Role::Label
    }

    pub fn label(&self) -> Option<String> {
        if let Some(label) = &self.data().label() {
            Some(label.to_string())
        } else {
            let labels = self
                .labelled_by()
                .filter_map(|node| {
                    if node.label_comes_from_value() {
                        node.value()
                    } else {
                        node.label()
                    }
                })
                .collect::<Vec<String>>();
            (!labels.is_empty()).then(move || labels.join(" "))
        }
    }

    pub fn description(&self) -> Option<String> {
        self.data()
            .description()
            .map(|description| description.to_string())
    }

    pub fn placeholder(&self) -> Option<String> {
        self.data()
            .placeholder()
            .map(|placeholder| placeholder.to_string())
    }

    pub fn value(&self) -> Option<String> {
        if let Some(value) = &self.data().value() {
            Some(value.to_string())
        } else if self.supports_text_ranges() && !self.is_multiline() {
            Some(self.document_range().text())
        } else {
            None
        }
    }

    pub fn has_value(&self) -> bool {
        self.data().value().is_some() || (self.supports_text_ranges() && !self.is_multiline())
    }

    pub fn is_read_only_supported(&self) -> bool {
        self.is_text_input()
            || matches!(
                self.role(),
                Role::CheckBox
                    | Role::ColorWell
                    | Role::ComboBox
                    | Role::Grid
                    | Role::ListBox
                    | Role::MenuItemCheckBox
                    | Role::MenuItemRadio
                    | Role::MenuListPopup
                    | Role::RadioButton
                    | Role::RadioGroup
                    | Role::Slider
                    | Role::Switch
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
            .live()
            .unwrap_or_else(|| self.parent().map_or(Live::Off, |parent| parent.live()))
    }

    pub fn is_selected(&self) -> Option<bool> {
        self.data().is_selected()
    }

    pub fn raw_text_selection(&self) -> Option<&TextSelection> {
        self.data().text_selection()
    }

    pub fn raw_value(&self) -> Option<&str> {
        self.data().value()
    }

    pub fn author_id(&self) -> Option<&str> {
        self.data().author_id()
    }

    pub fn class_name(&self) -> Option<&str> {
        self.data().class_name()
    }

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
}

#[cfg(test)]
mod tests {
    use accesskit::{Node, NodeId, Point, Rect, Role, Tree, TreeUpdate};
    use alloc::vec;

    use crate::tests::*;

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
                .node_by_id(LABEL_0_0_IGNORED_ID)
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
            LABEL_0_0_IGNORED_ID,
            tree.state().root().deepest_first_child().unwrap().id()
        );
        assert_eq!(
            LABEL_0_0_IGNORED_ID,
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .deepest_first_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .state()
            .node_by_id(LABEL_0_0_IGNORED_ID)
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
                .node_by_id(LABEL_1_1_ID)
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
            .node_by_id(LABEL_0_0_IGNORED_ID)
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
            .node_by_id(LABEL_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.state().root()));
        assert!(tree
            .state()
            .node_by_id(LABEL_0_0_IGNORED_ID)
            .unwrap()
            .is_descendant_of(&tree.state().node_by_id(PARAGRAPH_0_ID).unwrap()));
        assert!(!tree
            .state()
            .node_by_id(LABEL_0_0_IGNORED_ID)
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
                .node_by_id(LABEL_1_1_ID)
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
            Some(LABEL_1_1_ID),
            tree.state()
                .root()
                .node_at_point(Point::new(20.0, 50.0), &test_tree_filter)
                .map(|node| node.id())
        );
        assert_eq!(
            Some(LABEL_1_1_ID),
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
    fn no_label_or_labelled_by() {
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
        assert_eq!(None, tree.state().node_by_id(NodeId(1)).unwrap().label());
    }

    #[test]
    fn label_from_labelled_by() {
        // The following mock UI probably isn't very localization-friendly,
        // but it's good for this test.
        const LABEL_1: &str = "Check email every";
        const LABEL_2: &str = "minutes";

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2), NodeId(3), NodeId(4)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::CheckBox);
                    node.set_labelled_by(vec![NodeId(2), NodeId(4)]);
                    node
                }),
                (NodeId(2), {
                    let mut node = Node::new(Role::Label);
                    node.set_value(LABEL_1);
                    node
                }),
                (NodeId(3), {
                    let mut node = Node::new(Role::TextInput);
                    node.push_labelled_by(NodeId(4));
                    node
                }),
                (NodeId(4), {
                    let mut node = Node::new(Role::Label);
                    node.set_value(LABEL_2);
                    node
                }),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            Some([LABEL_1, LABEL_2].join(" ")),
            tree.state().node_by_id(NodeId(1)).unwrap().label()
        );
        assert_eq!(
            Some(LABEL_2.into()),
            tree.state().node_by_id(NodeId(3)).unwrap().label()
        );
    }

    #[test]
    fn label_from_descendant_label() {
        const ROOT_ID: NodeId = NodeId(0);
        const DEFAULT_BUTTON_ID: NodeId = NodeId(1);
        const DEFAULT_BUTTON_LABEL_ID: NodeId = NodeId(2);
        const LINK_ID: NodeId = NodeId(3);
        const LINK_LABEL_CONTAINER_ID: NodeId = NodeId(4);
        const LINK_LABEL_ID: NodeId = NodeId(5);
        const CHECKBOX_ID: NodeId = NodeId(6);
        const CHECKBOX_LABEL_ID: NodeId = NodeId(7);
        const RADIO_BUTTON_ID: NodeId = NodeId(8);
        const RADIO_BUTTON_LABEL_ID: NodeId = NodeId(9);
        const MENU_BUTTON_ID: NodeId = NodeId(10);
        const MENU_BUTTON_LABEL_ID: NodeId = NodeId(11);
        const MENU_ID: NodeId = NodeId(12);
        const MENU_ITEM_ID: NodeId = NodeId(13);
        const MENU_ITEM_LABEL_ID: NodeId = NodeId(14);
        const MENU_ITEM_CHECKBOX_ID: NodeId = NodeId(15);
        const MENU_ITEM_CHECKBOX_LABEL_ID: NodeId = NodeId(16);
        const MENU_ITEM_RADIO_ID: NodeId = NodeId(17);
        const MENU_ITEM_RADIO_LABEL_ID: NodeId = NodeId(18);

        const DEFAULT_BUTTON_LABEL: &str = "Play";
        const LINK_LABEL: &str = "Watch in browser";
        const CHECKBOX_LABEL: &str = "Resume from previous position";
        const RADIO_BUTTON_LABEL: &str = "Normal speed";
        const MENU_BUTTON_LABEL: &str = "More";
        const MENU_ITEM_LABEL: &str = "Share";
        const MENU_ITEM_CHECKBOX_LABEL: &str = "Apply volume processing";
        const MENU_ITEM_RADIO_LABEL: &str = "Maximize loudness for noisy environment";

        let update = TreeUpdate {
            nodes: vec![
                (ROOT_ID, {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![
                        DEFAULT_BUTTON_ID,
                        LINK_ID,
                        CHECKBOX_ID,
                        RADIO_BUTTON_ID,
                        MENU_BUTTON_ID,
                        MENU_ID,
                    ]);
                    node
                }),
                (DEFAULT_BUTTON_ID, {
                    let mut node = Node::new(Role::DefaultButton);
                    node.push_child(DEFAULT_BUTTON_LABEL_ID);
                    node
                }),
                (DEFAULT_BUTTON_LABEL_ID, {
                    let mut node = Node::new(Role::Image);
                    node.set_label(DEFAULT_BUTTON_LABEL);
                    node
                }),
                (LINK_ID, {
                    let mut node = Node::new(Role::Link);
                    node.push_child(LINK_LABEL_CONTAINER_ID);
                    node
                }),
                (LINK_LABEL_CONTAINER_ID, {
                    let mut node = Node::new(Role::GenericContainer);
                    node.push_child(LINK_LABEL_ID);
                    node
                }),
                (LINK_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(LINK_LABEL);
                    node
                }),
                (CHECKBOX_ID, {
                    let mut node = Node::new(Role::CheckBox);
                    node.push_child(CHECKBOX_LABEL_ID);
                    node
                }),
                (CHECKBOX_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(CHECKBOX_LABEL);
                    node
                }),
                (RADIO_BUTTON_ID, {
                    let mut node = Node::new(Role::RadioButton);
                    node.push_child(RADIO_BUTTON_LABEL_ID);
                    node
                }),
                (RADIO_BUTTON_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(RADIO_BUTTON_LABEL);
                    node
                }),
                (MENU_BUTTON_ID, {
                    let mut node = Node::new(Role::Button);
                    node.push_child(MENU_BUTTON_LABEL_ID);
                    node
                }),
                (MENU_BUTTON_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(MENU_BUTTON_LABEL);
                    node
                }),
                (MENU_ID, {
                    let mut node = Node::new(Role::Menu);
                    node.set_children([MENU_ITEM_ID, MENU_ITEM_CHECKBOX_ID, MENU_ITEM_RADIO_ID]);
                    node
                }),
                (MENU_ITEM_ID, {
                    let mut node = Node::new(Role::MenuItem);
                    node.push_child(MENU_ITEM_LABEL_ID);
                    node
                }),
                (MENU_ITEM_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(MENU_ITEM_LABEL);
                    node
                }),
                (MENU_ITEM_CHECKBOX_ID, {
                    let mut node = Node::new(Role::MenuItemCheckBox);
                    node.push_child(MENU_ITEM_CHECKBOX_LABEL_ID);
                    node
                }),
                (MENU_ITEM_CHECKBOX_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(MENU_ITEM_CHECKBOX_LABEL);
                    node
                }),
                (MENU_ITEM_RADIO_ID, {
                    let mut node = Node::new(Role::MenuItemRadio);
                    node.push_child(MENU_ITEM_RADIO_LABEL_ID);
                    node
                }),
                (MENU_ITEM_RADIO_LABEL_ID, {
                    let mut node = Node::new(Role::Label);
                    node.set_value(MENU_ITEM_RADIO_LABEL);
                    node
                }),
            ],
            tree: Some(Tree::new(ROOT_ID)),
            focus: ROOT_ID,
        };
        let tree = crate::Tree::new(update, false);
        assert_eq!(
            Some(DEFAULT_BUTTON_LABEL.into()),
            tree.state().node_by_id(DEFAULT_BUTTON_ID).unwrap().label()
        );
        assert_eq!(
            Some(LINK_LABEL.into()),
            tree.state().node_by_id(LINK_ID).unwrap().label()
        );
        assert_eq!(
            Some(CHECKBOX_LABEL.into()),
            tree.state().node_by_id(CHECKBOX_ID).unwrap().label()
        );
        assert_eq!(
            Some(RADIO_BUTTON_LABEL.into()),
            tree.state().node_by_id(RADIO_BUTTON_ID).unwrap().label()
        );
        assert_eq!(
            Some(MENU_BUTTON_LABEL.into()),
            tree.state().node_by_id(MENU_BUTTON_ID).unwrap().label()
        );
        assert_eq!(
            Some(MENU_ITEM_LABEL.into()),
            tree.state().node_by_id(MENU_ITEM_ID).unwrap().label()
        );
        assert_eq!(
            Some(MENU_ITEM_CHECKBOX_LABEL.into()),
            tree.state()
                .node_by_id(MENU_ITEM_CHECKBOX_ID)
                .unwrap()
                .label()
        );
        assert_eq!(
            Some(MENU_ITEM_RADIO_LABEL.into()),
            tree.state().node_by_id(MENU_ITEM_RADIO_ID).unwrap().label()
        );
    }
}
