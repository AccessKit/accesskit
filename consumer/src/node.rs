// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::iter::FusedIterator;
use std::sync::{Arc, Weak};

use accesskit_schema::{NodeId, Rect, Role};

use crate::iterators::{
    FollowingSiblings, FollowingUnignoredSiblings, PrecedingSiblings, PrecedingUnignoredSiblings,
    UnignoredChildren,
};
use crate::tree::{NodeState, ParentAndIndex, Reader as TreeReader, Tree};
use crate::NodeData;

pub struct Node<'a> {
    pub tree_reader: &'a TreeReader<'a>,
    pub(crate) state: &'a NodeState,
}

impl Node<'_> {
    pub fn data(&self) -> &NodeData {
        &self.state.data
    }

    pub fn is_focused(&self) -> bool {
        self.tree_reader.state.data.focus == Some(self.id())
    }

    pub fn is_ignored(&self) -> bool {
        self.data().ignored || (self.role() == Role::Presentation)
    }

    pub fn is_invisible_or_ignored(&self) -> bool {
        (self.is_invisible() || self.is_ignored()) && !self.is_focused()
    }

    pub fn parent(&self) -> Option<Node<'_>> {
        if let Some(ParentAndIndex(parent, _)) = &self.state.parent_and_index {
            Some(self.tree_reader.node_by_id(*parent).unwrap())
        } else {
            None
        }
    }

    pub fn unignored_parent(&self) -> Option<Node<'_>> {
        if let Some(parent) = self.parent() {
            if parent.is_ignored() {
                // Work around lifetime issues.
                parent
                    .unignored_parent()
                    .map(|node| self.tree_reader.node_by_id(node.id()).unwrap())
            } else {
                Some(parent)
            }
        } else {
            None
        }
    }

    pub fn parent_and_index(&self) -> Option<(Node<'_>, usize)> {
        self.state
            .parent_and_index
            .as_ref()
            .map(|ParentAndIndex(parent, index)| {
                (self.tree_reader.node_by_id(*parent).unwrap(), *index)
            })
    }

    pub fn children(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'_>>
           + ExactSizeIterator<Item = Node<'_>>
           + FusedIterator<Item = Node<'_>> {
        self.data()
            .children
            .iter()
            .map(move |id| self.tree_reader.node_by_id(*id).unwrap())
    }

    pub fn unignored_children(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'_>> + FusedIterator<Item = Node<'_>> {
        UnignoredChildren::new(self).map(move |id| self.tree_reader.node_by_id(id).unwrap())
    }

    pub fn following_siblings(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'_>>
           + ExactSizeIterator<Item = Node<'_>>
           + FusedIterator<Item = Node<'_>> {
        FollowingSiblings::new(self).map(move |id| self.tree_reader.node_by_id(id).unwrap())
    }

    pub fn following_unignored_siblings(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'_>> + FusedIterator<Item = Node<'_>> {
        FollowingUnignoredSiblings::new(self)
            .map(move |id| self.tree_reader.node_by_id(id).unwrap())
    }

    pub fn preceding_siblings(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'_>>
           + ExactSizeIterator<Item = Node<'_>>
           + FusedIterator<Item = Node<'_>> {
        PrecedingSiblings::new(self).map(move |id| self.tree_reader.node_by_id(id).unwrap())
    }

    pub fn preceding_unignored_siblings(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'_>> + FusedIterator<Item = Node<'_>> {
        PrecedingUnignoredSiblings::new(self)
            .map(move |id| self.tree_reader.node_by_id(id).unwrap())
    }

    pub fn deepest_first_child(&self) -> Option<Node<'_>> {
        let mut deepest_child = *self.data().children.get(0)?;
        while let Some(first_child) = self
            .tree_reader
            .node_by_id(deepest_child)
            .unwrap()
            .data()
            .children
            .get(0)
        {
            deepest_child = *first_child;
        }
        self.tree_reader.node_by_id(deepest_child)
    }

    pub fn deepest_first_unignored_child(&self) -> Option<Node<'_>> {
        let mut deepest_child = self.first_unignored_child()?;
        while let Some(first_child) = self
            .tree_reader
            .node_by_id(deepest_child)
            .unwrap()
            .first_unignored_child()
        {
            deepest_child = first_child;
        }
        self.tree_reader.node_by_id(deepest_child)
    }

    pub fn deepest_last_child(&self) -> Option<Node<'_>> {
        let mut deepest_child = *self.data().children.iter().next_back()?;
        while let Some(last_child) = self
            .tree_reader
            .node_by_id(deepest_child)
            .unwrap()
            .data()
            .children
            .iter()
            .next_back()
        {
            deepest_child = *last_child;
        }
        self.tree_reader.node_by_id(deepest_child)
    }

    pub fn deepest_last_unignored_child(&self) -> Option<Node<'_>> {
        let mut deepest_child = self.last_unignored_child()?;
        while let Some(last_child) = self
            .tree_reader
            .node_by_id(deepest_child)
            .unwrap()
            .last_unignored_child()
        {
            deepest_child = last_child;
        }
        self.tree_reader.node_by_id(deepest_child)
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

    /// Returns the node's bounds relative to the root of the tree.
    pub fn bounds(&self) -> Option<Rect> {
        if let Some(bounds) = &self.data().bounds {
            // TODO: handle offset container
            assert!(bounds.offset_container.is_none());
            // TODO: handle transform
            assert!(bounds.transform.is_none());
            Some(bounds.rect.clone())
        } else {
            None
        }
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

    pub fn name(&self) -> Option<&str> {
        if let Some(name) = &self.data().name {
            Some(name)
        } else {
            None
        }
    }

    pub(crate) fn first_unignored_child(&self) -> Option<NodeId> {
        for child in self.children() {
            if !child.is_ignored() {
                return Some(child.id());
            }
            if let Some(descendant) = child.first_unignored_child() {
                return Some(descendant);
            }
        }
        None
    }

    pub(crate) fn last_unignored_child(&self) -> Option<NodeId> {
        for child in self.children().rev() {
            if !child.is_ignored() {
                return Some(child.id());
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
        for<'a> F: FnOnce(&Node<'a>) -> T,
    {
        self.tree
            .upgrade()
            .map(|tree| tree.read().node_by_id(self.id).map(|node| f(&node)))
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
    use crate::tests::*;

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
            BUTTON_3_1_ID,
            tree.read().root().deepest_last_child().unwrap().id()
        );
        assert_eq!(
            BUTTON_3_1_ID,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(BUTTON_3_1_ID)
            .unwrap()
            .deepest_last_child()
            .is_none());
    }

    #[test]
    fn deepest_last_unignored_child() {
        let tree = test_tree();
        assert_eq!(
            BUTTON_3_1_ID,
            tree.read()
                .root()
                .deepest_last_unignored_child()
                .unwrap()
                .id()
        );
        assert_eq!(
            BUTTON_3_1_ID,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .deepest_last_unignored_child()
                .unwrap()
                .id()
        );
        assert!(tree
            .read()
            .node_by_id(BUTTON_3_1_ID)
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
}
