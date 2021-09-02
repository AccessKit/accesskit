// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

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

    pub fn index_in_parent(&self) -> Option<usize> {
        self.state
            .parent_and_index
            .as_ref()
            .map(|ParentAndIndex(_, index)| *index)
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
    use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
    use std::num::NonZeroU64;

    const TREE_ID: &str = "test_tree";
    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
    const NODE_ID_4: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(4) });

    #[test]
    fn index_in_parent() {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: vec![NODE_ID_2],
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node {
                    children: vec![NODE_ID_3, NODE_ID_4],
                    ..Node::new(NODE_ID_2, Role::RootWebArea)
                },
                Node {
                    name: Some(String::from("Button 1")),
                    ..Node::new(NODE_ID_3, Role::Button)
                },
                Node {
                    name: Some(String::from("Button 2")),
                    ..Node::new(NODE_ID_4, Role::Button)
                },
            ],
            tree: Some(Tree::new(TreeId(TREE_ID.to_string()), StringEncoding::Utf8)),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(update);
        assert!(tree.read().root().index_in_parent().is_none());
        assert_eq!(
            Some(0),
            tree.read().node_by_id(NODE_ID_2).unwrap().index_in_parent()
        );
        assert_eq!(
            Some(0),
            tree.read().node_by_id(NODE_ID_3).unwrap().index_in_parent()
        );
        assert_eq!(
            Some(1),
            tree.read().node_by_id(NODE_ID_4).unwrap().index_in_parent()
        );
    }
}
