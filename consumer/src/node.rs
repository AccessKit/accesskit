// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::iter::FusedIterator;
use std::sync::{Arc, Weak};

use accesskit::kurbo::{Affine, Rect};
use accesskit::{Action, ActionRequest, CheckedState, NodeId, Role};

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

    pub fn set_focus(&self) {
        self.tree_reader.tree.action_handler.do_action(ActionRequest {
            action: Action::Focus,
            target: self.id(),
            data: None,
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

    pub fn checked_state(&self) -> Option<CheckedState> {
        self.data().checked_state
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
            .map(|tree| tree.read().node_by_id(self.id).map(f))
            .flatten()
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
    use accesskit::kurbo::Rect;
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
    fn no_name_or_labelled_by() {
        let update = TreeUpdate {
            clear: None,
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
            clear: None,
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
