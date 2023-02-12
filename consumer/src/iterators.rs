// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use std::iter::FusedIterator;

use accesskit::NodeId;

use crate::{node::Node, tree::State as TreeState};

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
    pub(crate) fn new(node: Node<'a>) -> Self {
        let parent_and_index = node.parent_and_index();
        let (back_position, front_position, done) =
            if let Some((ref parent, index)) = parent_and_index {
                let back_position = parent.data().children().len() - 1;
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
                .children()
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
                .children()
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
    pub(crate) fn new(node: Node<'a>) -> Self {
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
                .children()
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
                .children()
                .get(self.back_position)?;
            self.back_position += 1;
            Some(*child)
        }
    }
}

impl<'a> ExactSizeIterator for PrecedingSiblings<'a> {}

impl<'a> FusedIterator for PrecedingSiblings<'a> {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterResult {
    Include,
    ExcludeNode,
    ExcludeSubtree,
}

fn next_filtered_sibling<'a>(
    node: Option<Node<'a>>,
    filter: &impl Fn(&Node) -> FilterResult,
) -> Option<Node<'a>> {
    let mut next = node;
    let mut consider_children = false;
    while let Some(current) = next {
        if let Some(Some(child)) = consider_children.then(|| current.children().next()) {
            let result = filter(&child);
            next = Some(child);
            if result == FilterResult::Include {
                return next;
            }
        } else if let Some(sibling) = current.following_siblings().next() {
            let result = filter(&sibling);
            next = Some(sibling);
            if result == FilterResult::Include {
                return next;
            }
            if result == FilterResult::ExcludeNode {
                consider_children = true;
            }
        } else {
            let parent = current.parent();
            next = parent;
            if let Some(parent) = parent {
                if filter(&parent) != FilterResult::ExcludeNode {
                    return None;
                }
                consider_children = false;
            } else {
                return None;
            }
        }
    }
    None
}

fn previous_filtered_sibling<'a>(
    node: Option<Node<'a>>,
    filter: &impl Fn(&Node) -> FilterResult,
) -> Option<Node<'a>> {
    let mut previous = node;
    let mut consider_children = false;
    while let Some(current) = previous {
        if let Some(Some(child)) = consider_children.then(|| current.children().next_back()) {
            let result = filter(&child);
            previous = Some(child);
            if result == FilterResult::Include {
                return previous;
            }
        } else if let Some(sibling) = current.preceding_siblings().next() {
            let result = filter(&sibling);
            previous = Some(sibling);
            if result == FilterResult::Include {
                return previous;
            }
            if result == FilterResult::ExcludeNode {
                consider_children = true;
            }
        } else {
            let parent = current.parent();
            previous = parent;
            if let Some(parent) = parent {
                if filter(&parent) != FilterResult::ExcludeNode {
                    return None;
                }
                consider_children = false;
            } else {
                return None;
            }
        }
    }
    None
}

/// An iterator that yields following siblings of a node according to the
/// specified filter.
///
/// This struct is created by the [following_filtered_siblings](Node::following_filtered_siblings) method on [Node].
pub struct FollowingFilteredSiblings<'a, Filter: Fn(&Node) -> FilterResult> {
    filter: Filter,
    back: Option<Node<'a>>,
    done: bool,
    front: Option<Node<'a>>,
}

impl<'a, Filter: Fn(&Node) -> FilterResult> FollowingFilteredSiblings<'a, Filter> {
    pub(crate) fn new(node: Node<'a>, filter: Filter) -> Self {
        let front = next_filtered_sibling(Some(node), &filter);
        let back = node
            .parent()
            .and_then(|parent| parent.last_filtered_child(&filter));
        Self {
            filter,
            back,
            done: back.is_none() || front.is_none(),
            front,
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> Iterator for FollowingFilteredSiblings<'a, Filter> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front.as_ref().unwrap().id() == self.back.as_ref().unwrap().id();
            let current = self.front;
            self.front = next_filtered_sibling(self.front, &self.filter);
            current
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> DoubleEndedIterator
    for FollowingFilteredSiblings<'a, Filter>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back.as_ref().unwrap().id() == self.front.as_ref().unwrap().id();
            let current = self.back;
            self.back = previous_filtered_sibling(self.back, &self.filter);
            current
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> FusedIterator
    for FollowingFilteredSiblings<'a, Filter>
{
}

/// An iterator that yields preceding siblings of a node according to the
/// specified filter.
///
/// This struct is created by the [preceding_filtered_siblings](Node::preceding_filtered_siblings) method on [Node].
pub struct PrecedingFilteredSiblings<'a, Filter: Fn(&Node) -> FilterResult> {
    filter: Filter,
    back: Option<Node<'a>>,
    done: bool,
    front: Option<Node<'a>>,
}

impl<'a, Filter: Fn(&Node) -> FilterResult> PrecedingFilteredSiblings<'a, Filter> {
    pub(crate) fn new(node: Node<'a>, filter: Filter) -> Self {
        let front = previous_filtered_sibling(Some(node), &filter);
        let back = node
            .parent()
            .and_then(|parent| parent.first_filtered_child(&filter));
        Self {
            filter,
            back,
            done: back.is_none() || front.is_none(),
            front,
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> Iterator for PrecedingFilteredSiblings<'a, Filter> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front.as_ref().unwrap().id() == self.back.as_ref().unwrap().id();
            let current = self.front;
            self.front = previous_filtered_sibling(self.front, &self.filter);
            current
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> DoubleEndedIterator
    for PrecedingFilteredSiblings<'a, Filter>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back.as_ref().unwrap().id() == self.front.as_ref().unwrap().id();
            let current = self.back;
            self.back = next_filtered_sibling(self.back, &self.filter);
            current
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> FusedIterator
    for PrecedingFilteredSiblings<'a, Filter>
{
}

/// An iterator that yields children of a node according to the specified
/// filter.
///
/// This struct is created by the [filtered_children](Node::filtered_children) method on [Node].
pub struct FilteredChildren<'a, Filter: Fn(&Node) -> FilterResult> {
    filter: Filter,
    back: Option<Node<'a>>,
    done: bool,
    front: Option<Node<'a>>,
}

impl<'a, Filter: Fn(&Node) -> FilterResult> FilteredChildren<'a, Filter> {
    pub(crate) fn new(node: Node<'a>, filter: Filter) -> Self {
        let front = node.first_filtered_child(&filter);
        let back = node.last_filtered_child(&filter);
        Self {
            filter,
            back,
            done: back.is_none() || front.is_none(),
            front,
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> Iterator for FilteredChildren<'a, Filter> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.front.as_ref().unwrap().id() == self.back.as_ref().unwrap().id();
            let current = self.front;
            self.front = next_filtered_sibling(self.front, &self.filter);
            current
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> DoubleEndedIterator for FilteredChildren<'a, Filter> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.back.as_ref().unwrap().id() == self.front.as_ref().unwrap().id();
            let current = self.back;
            self.back = previous_filtered_sibling(self.back, &self.filter);
            current
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> FusedIterator for FilteredChildren<'a, Filter> {}

pub(crate) enum LabelledBy<'a, Filter: Fn(&Node) -> FilterResult> {
    FromDescendants(FilteredChildren<'a, Filter>),
    Explicit {
        ids: std::slice::Iter<'a, NodeId>,
        tree_state: &'a TreeState,
    },
}

impl<'a, Filter: Fn(&Node) -> FilterResult> Iterator for LabelledBy<'a, Filter> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FromDescendants(iter) => iter.next(),
            Self::Explicit { ids, tree_state } => {
                ids.next().map(|id| tree_state.node_by_id(*id).unwrap())
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::FromDescendants(iter) => iter.size_hint(),
            Self::Explicit { ids, .. } => ids.size_hint(),
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> DoubleEndedIterator for LabelledBy<'a, Filter> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::FromDescendants(iter) => iter.next_back(),
            Self::Explicit { ids, tree_state } => ids
                .next_back()
                .map(|id| tree_state.node_by_id(*id).unwrap()),
        }
    }
}

impl<'a, Filter: Fn(&Node) -> FilterResult> FusedIterator for LabelledBy<'a, Filter> {}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use accesskit::NodeId;

    #[test]
    fn following_siblings() {
        let tree = test_tree();
        assert!(tree.state().root().following_siblings().next().is_none());
        assert_eq!(0, tree.state().root().following_siblings().len());
        assert_eq!(
            [
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID
            ],
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_siblings()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert_eq!(
            3,
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_siblings()
                .len()
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_siblings()
            .next()
            .is_none());
        assert_eq!(
            0,
            tree.state()
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
            .state()
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
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_siblings()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_siblings()
            .next_back()
            .is_none());
    }

    #[test]
    fn preceding_siblings() {
        let tree = test_tree();
        assert!(tree.state().root().preceding_siblings().next().is_none());
        assert_eq!(0, tree.state().root().preceding_siblings().len());
        assert_eq!(
            [PARAGRAPH_2_ID, PARAGRAPH_1_IGNORED_ID, PARAGRAPH_0_ID],
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_siblings()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert_eq!(
            3,
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_siblings()
                .len()
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_siblings()
            .next()
            .is_none());
        assert_eq!(
            0,
            tree.state()
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
            .state()
            .root()
            .preceding_siblings()
            .next_back()
            .is_none());
        assert_eq!(
            [PARAGRAPH_0_ID, PARAGRAPH_1_IGNORED_ID, PARAGRAPH_2_ID],
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_siblings()
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_siblings()
            .next_back()
            .is_none());
    }

    #[test]
    fn following_filtered_siblings() {
        let tree = test_tree();
        assert!(tree
            .state()
            .root()
            .following_filtered_siblings(test_tree_filter)
            .next()
            .is_none());
        assert_eq!(
            [
                STATIC_TEXT_1_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_3_1_0_ID,
                BUTTON_3_2_ID
            ],
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_filtered_siblings(test_tree_filter)
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_filtered_siblings(test_tree_filter)
            .next()
            .is_none());
    }

    #[test]
    fn following_filtered_siblings_reversed() {
        let tree = test_tree();
        assert!(tree
            .state()
            .root()
            .following_filtered_siblings(test_tree_filter)
            .next_back()
            .is_none());
        assert_eq!(
            [
                BUTTON_3_2_ID,
                STATIC_TEXT_3_1_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_1_0_ID
            ],
            tree.state()
                .node_by_id(PARAGRAPH_0_ID)
                .unwrap()
                .following_filtered_siblings(test_tree_filter)
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_3_IGNORED_ID)
            .unwrap()
            .following_filtered_siblings(test_tree_filter)
            .next_back()
            .is_none());
    }

    #[test]
    fn preceding_filtered_siblings() {
        let tree = test_tree();
        assert!(tree
            .state()
            .root()
            .preceding_filtered_siblings(test_tree_filter)
            .next()
            .is_none());
        assert_eq!(
            [PARAGRAPH_2_ID, STATIC_TEXT_1_0_ID, PARAGRAPH_0_ID],
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_filtered_siblings(test_tree_filter)
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_filtered_siblings(test_tree_filter)
            .next()
            .is_none());
    }

    #[test]
    fn preceding_filtered_siblings_reversed() {
        let tree = test_tree();
        assert!(tree
            .state()
            .root()
            .preceding_filtered_siblings(test_tree_filter)
            .next_back()
            .is_none());
        assert_eq!(
            [PARAGRAPH_0_ID, STATIC_TEXT_1_0_ID, PARAGRAPH_2_ID],
            tree.state()
                .node_by_id(PARAGRAPH_3_IGNORED_ID)
                .unwrap()
                .preceding_filtered_siblings(test_tree_filter)
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .preceding_filtered_siblings(test_tree_filter)
            .next_back()
            .is_none());
    }

    #[test]
    fn filtered_children() {
        let tree = test_tree();
        assert_eq!(
            [
                PARAGRAPH_0_ID,
                STATIC_TEXT_1_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_3_1_0_ID,
                BUTTON_3_2_ID
            ],
            tree.state()
                .root()
                .filtered_children(test_tree_filter)
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .filtered_children(test_tree_filter)
            .next()
            .is_none());
        assert!(tree
            .state()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .filtered_children(test_tree_filter)
            .next()
            .is_none());
    }

    #[test]
    fn filtered_children_reversed() {
        let tree = test_tree();
        assert_eq!(
            [
                BUTTON_3_2_ID,
                STATIC_TEXT_3_1_0_ID,
                PARAGRAPH_2_ID,
                STATIC_TEXT_1_0_ID,
                PARAGRAPH_0_ID
            ],
            tree.state()
                .root()
                .filtered_children(test_tree_filter)
                .rev()
                .map(|node| node.id())
                .collect::<Vec<NodeId>>()[..]
        );
        assert!(tree
            .state()
            .node_by_id(PARAGRAPH_0_ID)
            .unwrap()
            .filtered_children(test_tree_filter)
            .next_back()
            .is_none());
        assert!(tree
            .state()
            .node_by_id(STATIC_TEXT_0_0_IGNORED_ID)
            .unwrap()
            .filtered_children(test_tree_filter)
            .next_back()
            .is_none());
    }
}
