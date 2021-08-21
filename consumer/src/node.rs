// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::iter::FusedIterator;
use std::sync::{Arc, Weak};

use accesskit_schema::{NodeId, Rect, Role};

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
    ) -> impl DoubleEndedIterator<Item = Node<'_>>
           + ExactSizeIterator<Item = Node<'_>>
           + FusedIterator<Item = Node<'_>> {
        self.data()
            .children
            .iter()
            .map(move |id| self.tree_reader.node_by_id(*id).unwrap())
    }

    pub fn following_siblings(&self) -> FollowingSiblings {
        FollowingSiblings {
            delta: 1,
            index_in_parent: self
                .state
                .parent_and_index
                .as_ref()
                .map_or(0, |ParentAndIndex(_, index)| *index),
            parent: self.parent(),
            reader: self.tree_reader,
        }
    }

    pub fn preceding_siblings(&self) -> PrecedingSiblings {
        PrecedingSiblings {
            delta: 1,
            index_in_parent: self
                .state
                .parent_and_index
                .as_ref()
                .map_or(0, |ParentAndIndex(_, index)| *index),
            parent: self.parent(),
            reader: self.tree_reader,
        }
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

/// An iterator that yields following siblings of a node.
///
/// This struct is created by the [following_siblings](Node::following_siblings) method on [Node].
pub struct FollowingSiblings<'a> {
    delta: usize,
    index_in_parent: usize,
    parent: Option<Node<'a>>,
    reader: &'a TreeReader<'a>,
}

impl<'a> Iterator for FollowingSiblings<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.parent.as_ref()?;
        let sibling = parent
            .data()
            .children
            .get(self.index_in_parent + self.delta)?;
        self.delta += 1;
        self.reader.node_by_id(*sibling)
    }
}

impl<'a> DoubleEndedIterator for FollowingSiblings<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let parent = self.parent.as_ref()?;
        let parent_child_count = parent.data().children.len();
        if parent_child_count - self.delta != self.index_in_parent {
            let sibling = parent
                .data()
                .children
                .get(parent_child_count - self.delta)?;
            self.delta += 1;
            self.reader.node_by_id(*sibling)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for FollowingSiblings<'a> {
    fn len(&self) -> usize {
        if let Some(parent) = &self.parent {
            parent.data().children.len() - self.index_in_parent - 1
        } else {
            0
        }
    }
}

impl<'a> FusedIterator for FollowingSiblings<'a> {}

/// An iterator that yields preceding siblings of a node.
///
/// This struct is created by the [preceding_siblings](Node::preceding_siblings) method on [Node].
pub struct PrecedingSiblings<'a> {
    delta: usize,
    index_in_parent: usize,
    parent: Option<Node<'a>>,
    reader: &'a TreeReader<'a>,
}

impl<'a> Iterator for PrecedingSiblings<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.parent.as_ref()?;
        if self.delta - 1 != self.index_in_parent {
            let sibling = parent
                .data()
                .children
                .get(self.index_in_parent - self.delta)?;
            self.delta += 1;
            self.reader.node_by_id(*sibling)
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for PrecedingSiblings<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let parent = self.parent.as_ref()?;
        if self.delta - 1 != self.index_in_parent {
            let sibling = parent.data().children.get(self.delta - 1)?;
            self.delta += 1;
            self.reader.node_by_id(*sibling)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for PrecedingSiblings<'a> {
    fn len(&self) -> usize {
        self.index_in_parent
    }
}

impl<'a> FusedIterator for PrecedingSiblings<'a> {}

#[cfg(test)]
mod tests {
    use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
    use std::num::NonZeroU64;
    use std::sync::Arc;

    const ROOT_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
    const PARAGRAPH_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
    const STATIC_TEXT_0_0_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
    const PARAGRAPH_1_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(4) });
    const STATIC_TEXT_1_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(5) });
    const PARAGRAPH_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(6) });
    const STATIC_TEXT_2_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(7) });
    const PARAGRAPH_3_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(8) });
    const LINK_3_0_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(9) });
    const STATIC_TEXT_3_0_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(10) });
    const BUTTON_3_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(11) });

    #[test]
    fn following_siblings() {
        let tree = test_tree();
        assert!(tree.read().root().following_siblings().next().is_none());
        assert_eq!(0, tree.read().root().following_siblings().len());
        assert_eq!(
            [
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID
            ],
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_siblings()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert_eq!(
            3,
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_siblings()
                .len()
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_siblings()
            .next()
            .is_none());
        assert_eq!(
            0,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .following_siblings()
                .len()
        );
    }

    #[test]
    fn following_siblings_reversed() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .following_siblings()
            .rev()
            .next()
            .is_none());
        assert_eq!(
            [
                PARAGRAPH_3_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_1_IGNORED_ID
            ],
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_siblings()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_siblings()
            .rev()
            .next()
            .is_none());
    }

    #[test]
    fn preceding_siblings() {
        let tree = test_tree();
        assert!(tree.read().root().preceding_siblings().next().is_none());
        assert_eq!(0, tree.read().root().preceding_siblings().len());
        assert_eq!(
            [PARAGRAPH_2_ID, PARAGRAPH_1_IGNORED_ID, PARAGRAPH_0_ID],
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_siblings()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert_eq!(
            3,
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_siblings()
                .len()
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_siblings()
            .next()
            .is_none());
        assert_eq!(
            0,
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .preceding_siblings()
                .len()
        );
    }

    #[test]
    fn preceding_siblings_reversed() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .preceding_siblings()
            .rev()
            .next()
            .is_none());
        assert_eq!(
            [PARAGRAPH_0_ID, PARAGRAPH_1_IGNORED_ID, PARAGRAPH_2_ID],
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_siblings()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_siblings()
            .rev()
            .next()
            .is_none());
    }

    fn test_tree() -> Arc<crate::tree::Tree> {
        let root = Node {
            children: vec![
                PARAGRAPH_0_ID,
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID,
            ],
            ..Node::new(ROOT_ID, Role::RootWebArea)
        };
        let paragraph_0 = Node {
            children: vec![STATIC_TEXT_0_0_IGNORED_ID],
            ..Node::new(PARAGRAPH_0_ID, Role::Paragraph)
        };
        let static_text_0_0_ignored = Node {
            ignored: true,
            name: Some("static_text_0_0_ignored".to_string()),
            ..Node::new(STATIC_TEXT_0_0_IGNORED_ID, Role::StaticText)
        };
        let paragraph_1_ignored = Node {
            children: vec![STATIC_TEXT_1_0_ID],
            ignored: true,
            ..Node::new(PARAGRAPH_1_IGNORED_ID, Role::Paragraph)
        };
        let static_text_1_0 = Node {
            name: Some("static_text_1_0".to_string()),
            ..Node::new(STATIC_TEXT_1_0_ID, Role::StaticText)
        };
        let paragraph_2 = Node {
            children: vec![STATIC_TEXT_2_0_ID],
            ..Node::new(PARAGRAPH_2_ID, Role::Paragraph)
        };
        let static_text_2_0 = Node {
            name: Some("static_text_2_0".to_string()),
            ..Node::new(STATIC_TEXT_2_0_ID, Role::StaticText)
        };
        let paragraph_3_ignored = Node {
            children: vec![LINK_3_0_IGNORED_ID, BUTTON_3_1_ID],
            ignored: true,
            ..Node::new(PARAGRAPH_3_IGNORED_ID, Role::Paragraph)
        };
        let link_3_0_ignored = Node {
            children: vec![STATIC_TEXT_3_0_0_ID],
            ignored: true,
            linked: true,
            ..Node::new(LINK_3_0_IGNORED_ID, Role::Link)
        };
        let static_text_3_0_0 = Node {
            name: Some("static_text_3_0_0".to_string()),
            ..Node::new(STATIC_TEXT_3_0_0_ID, Role::StaticText)
        };
        let button_3_1 = Node {
            name: Some("button_3_1".to_string()),
            ..Node::new(BUTTON_3_1_ID, Role::Button)
        };
        let initial_update = TreeUpdate {
            clear: None,
            nodes: vec![
                root,
                paragraph_0,
                static_text_0_0_ignored,
                paragraph_1_ignored,
                static_text_1_0,
                paragraph_2,
                static_text_2_0,
                paragraph_3_ignored,
                link_3_0_ignored,
                static_text_3_0_0,
                button_3_1,
            ],
            tree: Some(Tree::new(
                TreeId("test_tree".to_string()),
                StringEncoding::Utf8,
            )),
            root: Some(ROOT_ID),
        };
        crate::tree::Tree::new(initial_update)
    }
}
