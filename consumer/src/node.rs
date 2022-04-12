// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2021 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use std::iter::FusedIterator;
use std::sync::{Arc, Weak};

use accesskit::kurbo::{Affine, Point, Rect};
use accesskit::{Action, ActionData, ActionRequest, CheckedState, DefaultActionVerb, NodeId, Role};

use crate::iterators::{
    FollowingSiblings, FollowingUnignoredSiblings, PrecedingSiblings, PrecedingUnignoredSiblings,
    UnignoredChildren,
};
use crate::tree::{NodeState, ParentAndIndex, Reader as TreeReader, Tree};
use crate::NodeData;

#[derive(Copy, Clone)]
pub struct Node<'a> {
    pub tree_reader: &'a TreeReader<'a>,
    pub(crate) state: &'a NodeState,
}

impl<'a> Node<'a> {
    pub fn data(&self) -> &NodeData {
        &self.state.data
    }

    pub fn is_focused(&self) -> bool {
        self.tree_reader.state.focus == Some(self.id())
    }

    pub fn is_focusable(&self) -> bool {
        // TBD: Is it ever safe to imply this on a node that doesn't explicitly
        // specify it?
        self.data().focusable
    }

    pub fn is_ignored(&self) -> bool {
        self.data().ignored || (self.role() == Role::Presentation)
    }

    pub fn is_invisible_or_ignored(&self) -> bool {
        (self.is_invisible() || self.is_ignored()) && !self.is_focused()
    }

    pub fn is_root(&self) -> bool {
        // Don't check for absence of a parent node, in case a non-root node
        // somehow gets detached from the tree.
        self.id() == self.tree_reader.state.data.root
    }

    pub fn parent(self) -> Option<Node<'a>> {
        if let Some(ParentAndIndex(parent, _)) = &self.state.parent_and_index {
            Some(self.tree_reader.node_by_id(*parent).unwrap())
        } else {
            None
        }
    }

    pub fn unignored_parent(self) -> Option<Node<'a>> {
        if let Some(parent) = self.parent() {
            if parent.is_ignored() {
                parent.unignored_parent()
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
                (self.tree_reader.node_by_id(*parent).unwrap(), *index)
            })
    }

    pub fn children(
        self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let data = &self.state.data;
        let reader = self.tree_reader;
        data.children
            .iter()
            .map(move |id| reader.node_by_id(*id).unwrap())
    }

    pub fn unignored_children(
        self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        UnignoredChildren::new(self)
    }

    pub fn following_siblings(
        self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let reader = self.tree_reader;
        FollowingSiblings::new(self).map(move |id| reader.node_by_id(id).unwrap())
    }

    pub fn following_unignored_siblings(
        self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        FollowingUnignoredSiblings::new(self)
    }

    pub fn preceding_siblings(
        self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        let reader = self.tree_reader;
        PrecedingSiblings::new(self).map(move |id| reader.node_by_id(id).unwrap())
    }

    pub fn preceding_unignored_siblings(
        self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        PrecedingUnignoredSiblings::new(self)
    }

    pub fn deepest_first_child(self) -> Option<Node<'a>> {
        let mut deepest_child = self.children().next()?;
        while let Some(first_child) = deepest_child.children().next() {
            deepest_child = first_child;
        }
        Some(deepest_child)
    }

    pub fn deepest_first_unignored_child(self) -> Option<Node<'a>> {
        let mut deepest_child = self.first_unignored_child()?;
        while let Some(first_child) = deepest_child.first_unignored_child() {
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

    pub fn deepest_last_unignored_child(self) -> Option<Node<'a>> {
        let mut deepest_child = self.last_unignored_child()?;
        while let Some(last_child) = deepest_child.last_unignored_child() {
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

    pub fn global_id(&self) -> String {
        format!("{}:{}", self.tree_reader.id().0, self.id().0)
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

    /// Returns the deepest visible node, either this node or a descendant,
    /// at the given point in this node's coordinate space.
    pub fn node_at_point(self, point: Point) -> Option<Node<'a>> {
        if self.is_invisible() {
            return None;
        }

        for child in self.children().rev() {
            let point = child.direct_transform().inverse() * point;
            if let Some(node) = child.node_at_point(point) {
                return Some(node);
            }
        }

        if !self.is_ignored() {
            if let Some(rect) = &self.data().bounds {
                if rect.contains(point) {
                    return Some(self);
                }
            }
        }

        None
    }

    pub fn set_focus(&self) {
        self.tree_reader
            .tree
            .action_handler
            .do_action(ActionRequest {
                action: Action::Focus,
                target: self.id(),
                data: None,
            })
    }

    pub fn do_default_action(&self) {
        self.tree_reader
            .tree
            .action_handler
            .do_action(ActionRequest {
                action: Action::Default,
                target: self.id(),
                data: None,
            })
    }

    pub fn set_value(&self, value: impl Into<Box<str>>) {
        self.tree_reader
            .tree
            .action_handler
            .do_action(ActionRequest {
                action: Action::SetValue,
                target: self.id(),
                data: Some(ActionData::Value(value.into())),
            })
    }

    pub fn set_numeric_value(&self, value: f64) {
        self.tree_reader
            .tree
            .action_handler
            .do_action(ActionRequest {
                action: Action::SetValue,
                target: self.id(),
                data: Some(ActionData::NumericValue(value)),
            })
    }

    // Convenience getters

    pub fn id(&self) -> NodeId {
        self.data().id
    }

    pub fn role(&self) -> Role {
        self.data().role
    }

    pub fn is_invisible(&self) -> bool {
        self.data().invisible
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
                        .filter_map(|id| self.tree_reader.node_by_id(*id).unwrap().name())
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

    pub(crate) fn first_unignored_child(self) -> Option<Node<'a>> {
        for child in self.children() {
            if !child.is_ignored() {
                return Some(child);
            }
            if let Some(descendant) = child.first_unignored_child() {
                return Some(descendant);
            }
        }
        None
    }

    pub(crate) fn last_unignored_child(self) -> Option<Node<'a>> {
        for child in self.children().rev() {
            if !child.is_ignored() {
                return Some(child);
            }
            if let Some(descendant) = child.last_unignored_child() {
                return Some(descendant);
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct WeakNode {
    pub tree: Weak<Tree>,
    pub id: NodeId,
}

impl WeakNode {
    pub fn map<F, T>(&self, f: F) -> Option<T>
    where
        for<'a> F: FnOnce(Node<'a>) -> T,
    {
        self.tree
            .upgrade()
            .and_then(|tree| tree.read().node_by_id(self.id).map(f))
    }
}

impl Node<'_> {
    pub fn downgrade(&self) -> WeakNode {
        WeakNode {
            tree: Arc::downgrade(self.tree_reader.tree),
            id: self.id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use accesskit::kurbo::{Point, Rect};
    use accesskit::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
    use std::num::NonZeroU64;

    use crate::tests::*;

    const TREE_ID: &str = "test_tree";
    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
    const NODE_ID_4: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(4) });
    const NODE_ID_5: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(5) });

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
    fn deepest_first_unignored_child() {
        let tree = test_tree();
        assert_eq!(
            PARAGRAPH_0_ID,
            tree.read()
                .root()
                .deepest_first_unignored_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .deepest_first_unignored_child()
            .is_none());
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .deepest_first_unignored_child()
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
    fn deepest_last_unignored_child() {
        let tree = test_tree();
        assert_eq!(
            BUTTON_3_2_ID,
            tree.read()
                .root()
                .deepest_last_unignored_child()
                .unwrap()
                .id()
        );
        assert_eq!(
            BUTTON_3_2_ID,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_unignored_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(BUTTON_3_2_ID)
            .unwrap()
            .deepest_last_unignored_child()
            .is_none());
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .deepest_last_unignored_child()
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
            .node_at_point(Point::new(10.0, 40.0))
            .is_none());
        assert_eq!(
            Some(STATIC_TEXT_1_0_ID),
            tree.read()
                .root()
                .node_at_point(Point::new(20.0, 50.0))
                .map(|node| node.id())
        );
        assert_eq!(
            Some(STATIC_TEXT_1_0_ID),
            tree.read()
                .root()
                .node_at_point(Point::new(50.0, 60.0))
                .map(|node| node.id())
        );
        assert!(tree
            .read()
            .root()
            .node_at_point(Point::new(100.0, 70.0))
            .is_none());
    }

    #[test]
    fn no_name_or_labelled_by() {
        let update = TreeUpdate {
            nodes: vec![
                Node {
                    children: vec![NODE_ID_2],
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node::new(NODE_ID_2, Role::Button),
            ],
            tree: Some(Tree::new(
                TreeId(TREE_ID.into()),
                NODE_ID_1,
                StringEncoding::Utf8,
            )),
            focus: None,
        };
        let tree = super::Tree::new(update, Box::new(NullActionHandler {}));
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
                Node {
                    children: vec![NODE_ID_2, NODE_ID_3, NODE_ID_4, NODE_ID_5],
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node {
                    labelled_by: vec![NODE_ID_3, NODE_ID_5],
                    ..Node::new(NODE_ID_2, Role::CheckBox)
                },
                Node {
                    name: Some(LABEL_1.into()),
                    ..Node::new(NODE_ID_3, Role::StaticText)
                },
                Node {
                    labelled_by: vec![NODE_ID_5],
                    ..Node::new(NODE_ID_4, Role::CheckBox)
                },
                Node {
                    name: Some(LABEL_2.into()),
                    ..Node::new(NODE_ID_5, Role::StaticText)
                },
            ],
            tree: Some(Tree::new(
                TreeId(TREE_ID.into()),
                NODE_ID_1,
                StringEncoding::Utf8,
            )),
            focus: None,
        };
        let tree = super::Tree::new(update, Box::new(NullActionHandler {}));
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
