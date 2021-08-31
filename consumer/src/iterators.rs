// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::iter::FusedIterator;

use accesskit_schema::NodeId;

use crate::node::Node;
use crate::tree::{ParentAndIndex, Reader as TreeReader};

/// An iterator that yields following siblings of a node.
///
/// This struct is created by the [following_siblings](Node::following_siblings) method on [Node].
pub struct FollowingSiblings<'a> {
    back_position: usize,
    done: bool,
    front_position: usize,
    parent: Option<Node<'a>>,
}

impl<'a> FollowingSiblings<'a> {
    pub(crate) fn new(node: &'a Node<'a>) -> Self {
        let parent = node.parent();
        let (back_position, front_position, done) = if let Some(parent) = parent.as_ref() {
            if let Some(ParentAndIndex(_, index)) = node.state.parent_and_index {
                let back_position = parent.data().children.len() - 1;
                let front_position = index + 1;
                (
                    back_position,
                    front_position,
                    front_position > back_position,
                )
            } else {
                (0, 0, true)
            }
        } else {
            (0, 0, true)
        };
        Self {
            back_position,
            done,
            front_position,
            parent,
        }
    }
}

impl<'a> Iterator for FollowingSiblings<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front_position == self.back_position;
            let child = self
                .parent
                .as_ref()?
                .data()
                .children
                .get(self.front_position)?;
            self.front_position += 1;
            Some(*child)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self.done {
            true => 0,
            _ => self.back_position + 1 - self.front_position,
        };
        (len, Some(len))
    }
}

impl<'a> DoubleEndedIterator for FollowingSiblings<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back_position == self.front_position;
            let child = self
                .parent
                .as_ref()?
                .data()
                .children
                .get(self.back_position)?;
            self.back_position -= 1;
            Some(*child)
        }
    }
}

impl<'a> ExactSizeIterator for FollowingSiblings<'a> {}

impl<'a> FusedIterator for FollowingSiblings<'a> {}

/// An iterator that yields preceding siblings of a node.
///
/// This struct is created by the [preceding_siblings](Node::preceding_siblings) method on [Node].
pub struct PrecedingSiblings<'a> {
    back_position: usize,
    done: bool,
    front_position: usize,
    parent: Option<Node<'a>>,
}

impl<'a> PrecedingSiblings<'a> {
    pub(crate) fn new(node: &'a Node<'a>) -> Self {
        let parent = node.parent();
        let (back_position, front_position, done) = if parent.is_some() {
            if let Some(ParentAndIndex(_, index)) = node.state.parent_and_index {
                let front_position = index.saturating_sub(1);
                (0, front_position, index == 0)
            } else {
                (0, 0, true)
            }
        } else {
            (0, 0, true)
        };
        Self {
            back_position,
            done,
            front_position,
            parent,
        }
    }
}

impl<'a> Iterator for PrecedingSiblings<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front_position == self.back_position;
            let child = self
                .parent
                .as_ref()?
                .data()
                .children
                .get(self.front_position)?;
            if !self.done {
                self.front_position -= 1;
            }
            Some(*child)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self.done {
            true => 0,
            _ => self.front_position + 1 - self.back_position,
        };
        (len, Some(len))
    }
}

impl<'a> DoubleEndedIterator for PrecedingSiblings<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back_position == self.front_position;
            let child = self
                .parent
                .as_ref()?
                .data()
                .children
                .get(self.back_position)?;
            self.back_position += 1;
            Some(*child)
        }
    }
}

impl<'a> ExactSizeIterator for PrecedingSiblings<'a> {}

impl<'a> FusedIterator for PrecedingSiblings<'a> {}

/// An iterator that yields unignored children of a node.
///
/// This struct is created by the [unignored_children](Node::unignored_children) method on [Node].
pub struct UnignoredChildren<'a> {
    back_id: Option<NodeId>,
    done: bool,
    front_id: Option<NodeId>,
    reader: &'a TreeReader<'a>,
}

impl<'a> UnignoredChildren<'a> {
    pub(crate) fn new(node: &'a Node<'a>) -> Self {
        let front_id = UnignoredChildren::first_unignored_child(node);
        let back_id = UnignoredChildren::last_unignored_child(node);
        Self {
            back_id,
            done: back_id.is_none() || front_id.is_none(),
            front_id,
            reader: node.tree_reader,
        }
    }

    fn first_unignored_child(node: &'a Node<'a>) -> Option<NodeId> {
        for child in node.children() {
            if !child.is_ignored() {
                return Some(child.id());
            }
            if let Some(descendant) = UnignoredChildren::first_unignored_child(&child) {
                return Some(descendant);
            }
        }
        None
    }

    fn last_unignored_child(node: &'a Node<'a>) -> Option<NodeId> {
        for child in node.children().rev() {
            if !child.is_ignored() {
                return Some(child.id());
            }
            if let Some(descendant) = UnignoredChildren::last_unignored_child(&child) {
                return Some(descendant);
            }
        }
        None
    }
}

impl<'a> Iterator for UnignoredChildren<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        self.done = self.front_id == self.back_id;
        let mut front_id = self.front_id;
        let mut consider_children = false;
        while let Some(current_node) = front_id.and_then(|id| self.reader.node_by_id(id)) {
            if let Some(Some(child)) = consider_children.then(|| current_node.children().next()) {
                front_id = Some(child.id());
                if !child.is_ignored() {
                    break;
                }
            } else if let Some(sibling) = current_node.following_siblings().next() {
                front_id = Some(sibling.id());
                if !sibling.is_ignored() {
                    break;
                }
                consider_children = true;
            } else {
                let parent = current_node.parent();
                front_id = parent.as_ref().map(|parent| parent.id());
                if let Some(parent) = parent {
                    if !parent.is_ignored() {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        let value = self.front_id;
        self.front_id = front_id;
        value
    }
}

impl<'a> DoubleEndedIterator for UnignoredChildren<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        self.done = self.back_id == self.front_id;
        let mut back_id = self.back_id;
        let mut consider_children = false;
        while let Some(current_node) = back_id.and_then(|id| self.reader.node_by_id(id)) {
            if let Some(Some(child)) =
                consider_children.then(|| current_node.children().next_back())
            {
                back_id = Some(child.id());
                if !child.is_ignored() {
                    break;
                }
            } else if let Some(sibling) = current_node.preceding_siblings().next() {
                back_id = Some(sibling.id());
                if !sibling.is_ignored() {
                    break;
                }
                consider_children = true;
            } else {
                let parent = current_node.parent();
                back_id = parent.as_ref().map(|parent| parent.id());
                if let Some(parent) = parent {
                    if !parent.is_ignored() {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        let value = self.back_id;
        self.back_id = back_id;
        value
    }
}

impl<'a> FusedIterator for UnignoredChildren<'a> {}

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
            .next_back()
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
            .next_back()
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
            .next_back()
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
            .next_back()
            .is_none());
    }

    #[test]
    fn unignored_children() {
        let tree = test_tree();
        assert_eq!(
            [
                PARAGRAPH_0_ID,
                STATIC_TEXT_1_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_3_0_0_ID,
                BUTTON_3_1_ID
            ],
            tree.read()
                .root()
                .unignored_children()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .unignored_children()
            .next()
            .is_none());
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .unignored_children()
            .next()
            .is_none());
    }

    #[test]
    fn unignored_children_reversed() {
        let tree = test_tree();
        assert_eq!(
            [
                BUTTON_3_1_ID,
                STATIC_TEXT_3_0_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_1_0_ID,
                PARAGRAPH_0_ID
            ],
            tree.read()
                .root()
                .unignored_children()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .unignored_children()
            .next_back()
            .is_none());
        assert!(tree
            .read()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .unignored_children()
            .next_back()
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
