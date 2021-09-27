// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use std::iter::FusedIterator;

use accesskit_schema::NodeId;

use crate::node::Node;
use crate::tree::Reader as TreeReader;

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
        let parent_and_index = node.parent_and_index();
        let (back_position, front_position, done) =
            if let Some((ref parent, index)) = parent_and_index {
                let back_position = parent.data().children.len() - 1;
                let front_position = index + 1;
                (
                    back_position,
                    front_position,
                    front_position > back_position,
                )
            } else {
                (0, 0, true)
            };
        Self {
            back_position,
            done,
            front_position,
            parent: parent_and_index.map(|(parent, _)| parent),
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
        let parent_and_index = node.parent_and_index();
        let (back_position, front_position, done) = if let Some((_, index)) = parent_and_index {
            let front_position = index.saturating_sub(1);
            (0, front_position, index == 0)
        } else {
            (0, 0, true)
        };
        Self {
            back_position,
            done,
            front_position,
            parent: parent_and_index.map(|(parent, _)| parent),
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

fn next_unignored_sibling(node_id: Option<NodeId>, reader: &TreeReader) -> Option<NodeId> {
    // Search for the next sibling of this node, skipping over any ignored nodes
    // encountered.
    //
    // In our search:
    //   If we find an ignored sibling, we consider its children as our siblings.
    //   If we run out of siblings, we consider an ignored parent's siblings as our
    //     own siblings.
    //
    // Note: this behaviour of 'skipping over' an ignored node makes this subtly
    // different to finding the next (direct) sibling which is unignored.
    let mut next_id = node_id;
    let mut consider_children = false;
    while let Some(current_node) = next_id.and_then(|id| reader.node_by_id(id)) {
        if let Some(Some(child)) = consider_children.then(|| current_node.children().next()) {
            next_id = Some(child.id());
            if !child.is_ignored() {
                return next_id;
            }
        } else if let Some(sibling) = current_node.following_siblings().next() {
            next_id = Some(sibling.id());
            if !sibling.is_ignored() {
                return next_id;
            }
            consider_children = true;
        } else {
            let parent = current_node.parent();
            next_id = parent.as_ref().map(|parent| parent.id());
            if let Some(parent) = parent {
                if !parent.is_ignored() {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
    None
}

fn previous_unignored_sibling(node_id: Option<NodeId>, reader: &TreeReader) -> Option<NodeId> {
    // Search for the previous sibling of this node, skipping over any ignored nodes
    // encountered.
    //
    // In our search for a sibling:
    //   If we find an ignored sibling, we may consider its children as siblings.
    //   If we run out of siblings, we may consider an ignored parent's siblings as
    //     our own.
    let mut previous_id = node_id;
    let mut consider_children = false;
    while let Some(current_node) = previous_id.and_then(|id| reader.node_by_id(id)) {
        if let Some(Some(child)) = consider_children.then(|| current_node.children().next_back()) {
            previous_id = Some(child.id());
            if !child.is_ignored() {
                return previous_id;
            }
        } else if let Some(sibling) = current_node.preceding_siblings().next() {
            previous_id = Some(sibling.id());
            if !sibling.is_ignored() {
                return previous_id;
            }
            consider_children = true;
        } else {
            let parent = current_node.parent();
            previous_id = parent.as_ref().map(|parent| parent.id());
            if let Some(parent) = parent {
                if !parent.is_ignored() {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
    None
}

/// An iterator that yields following unignored siblings of a node.
///
/// This struct is created by the [following_unignored_siblings](Node::following_unignored_siblings) method on [Node].
pub struct FollowingUnignoredSiblings<'a> {
    back_id: Option<NodeId>,
    done: bool,
    front_id: Option<NodeId>,
    reader: &'a TreeReader<'a>,
}

impl<'a> FollowingUnignoredSiblings<'a> {
    pub(crate) fn new(node: &'a Node<'a>) -> Self {
        let front_id = next_unignored_sibling(Some(node.id()), node.tree_reader);
        let back_id = node.parent().as_ref().and_then(Node::last_unignored_child);
        Self {
            back_id,
            done: back_id.is_none() || front_id.is_none(),
            front_id,
            reader: node.tree_reader,
        }
    }
}

impl<'a> Iterator for FollowingUnignoredSiblings<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front_id == self.back_id;
            let current_id = self.front_id;
            self.front_id = next_unignored_sibling(self.front_id, self.reader);
            current_id
        }
    }
}

impl<'a> DoubleEndedIterator for FollowingUnignoredSiblings<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back_id == self.front_id;
            let current_id = self.back_id;
            self.back_id = previous_unignored_sibling(self.back_id, self.reader);
            current_id
        }
    }
}

impl<'a> FusedIterator for FollowingUnignoredSiblings<'a> {}

/// An iterator that yields preceding unignored siblings of a node.
///
/// This struct is created by the [preceding_unignored_siblings](Node::preceding_unignored_siblings) method on [Node].
pub struct PrecedingUnignoredSiblings<'a> {
    back_id: Option<NodeId>,
    done: bool,
    front_id: Option<NodeId>,
    reader: &'a TreeReader<'a>,
}

impl<'a> PrecedingUnignoredSiblings<'a> {
    pub(crate) fn new(node: &'a Node<'a>) -> Self {
        let front_id = previous_unignored_sibling(Some(node.id()), node.tree_reader);
        let back_id = node.parent().as_ref().and_then(Node::first_unignored_child);
        Self {
            back_id,
            done: back_id.is_none() || front_id.is_none(),
            front_id,
            reader: node.tree_reader,
        }
    }
}

impl<'a> Iterator for PrecedingUnignoredSiblings<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front_id == self.back_id;
            let current_id = self.front_id;
            self.front_id = previous_unignored_sibling(self.front_id, self.reader);
            current_id
        }
    }
}

impl<'a> DoubleEndedIterator for PrecedingUnignoredSiblings<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back_id == self.front_id;
            let current_id = self.back_id;
            self.back_id = next_unignored_sibling(self.back_id, self.reader);
            current_id
        }
    }
}

impl<'a> FusedIterator for PrecedingUnignoredSiblings<'a> {}

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
        let front_id = node.first_unignored_child();
        let back_id = node.last_unignored_child();
        Self {
            back_id,
            done: back_id.is_none() || front_id.is_none(),
            front_id,
            reader: node.tree_reader,
        }
    }
}

impl<'a> Iterator for UnignoredChildren<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front_id == self.back_id;
            let current_id = self.front_id;
            self.front_id = next_unignored_sibling(self.front_id, self.reader);
            current_id
        }
    }
}

impl<'a> DoubleEndedIterator for UnignoredChildren<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back_id == self.front_id;
            let current_id = self.back_id;
            self.back_id = previous_unignored_sibling(self.back_id, self.reader);
            current_id
        }
    }
}

impl<'a> FusedIterator for UnignoredChildren<'a> {}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use accesskit_schema::NodeId;

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
    fn following_unignored_siblings() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .following_unignored_siblings()
            .next()
            .is_none());
        assert_eq!(
            [
                STATIC_TEXT_1_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_3_0_0_ID,
                BUTTON_3_1_ID
            ],
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_unignored_siblings()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_unignored_siblings()
            .next()
            .is_none());
    }

    #[test]
    fn following_unignored_siblings_reversed() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .following_unignored_siblings()
            .next_back()
            .is_none());
        assert_eq!(
            [
                BUTTON_3_1_ID,
                STATIC_TEXT_3_0_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_1_0_ID
            ],
            tree.read()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_unignored_siblings()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_unignored_siblings()
            .next_back()
            .is_none());
    }

    #[test]
    fn preceding_unignored_siblings() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .preceding_unignored_siblings()
            .next()
            .is_none());
        assert_eq!(
            [PARAGRAPH_2_ID, STATIC_TEXT_1_0_ID, PARAGRAPH_0_ID],
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_unignored_siblings()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_unignored_siblings()
            .next()
            .is_none());
    }

    #[test]
    fn preceding_unignored_siblings_reversed() {
        let tree = test_tree();
        assert!(tree
            .read()
            .root()
            .preceding_unignored_siblings()
            .next_back()
            .is_none());
        assert_eq!(
            [PARAGRAPH_0_ID, STATIC_TEXT_1_0_ID, PARAGRAPH_2_ID],
            tree.read()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_unignored_siblings()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .read()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_unignored_siblings()
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
}
