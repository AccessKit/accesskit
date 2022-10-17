// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{NodeId, Role, TextPosition as WeakPosition};
use std::cmp::Ordering;

use crate::{FilterResult, Node, TreeState};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unit {
    Character,
    Format,
    Word,
    Line,
    Paragraph,
    Page,
    Document,
}

#[derive(Clone, Copy)]
struct InnerPosition<'a> {
    node: Node<'a>,
    character_index: u16,
}

impl<'a> InnerPosition<'a> {
    fn upgrade(tree_state: &'a TreeState, weak: WeakPosition) -> Option<Self> {
        let node = tree_state.node_by_id(weak.node)?;
        if node.role() != Role::InlineTextBox {
            return None;
        }
        let character_index = weak.character_index;
        if (character_index as usize) > node.data().character_end_indices.len() {
            return None;
        }
        Some(Self {
            node,
            character_index,
        })
    }

    fn comparable(&self, root_node_id: NodeId) -> (Vec<usize>, u16) {
        (
            self.node.relative_index_path(root_node_id),
            self.character_index,
        )
    }

    fn downgrade(&self) -> WeakPosition {
        WeakPosition {
            node: self.node.id(),
            character_index: self.character_index,
        }
    }
}

impl<'a> PartialEq for InnerPosition<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.node.id() == other.node.id() && self.character_index == other.character_index
    }
}

impl<'a> Eq for InnerPosition<'a> {}

#[derive(Clone, Copy)]
pub struct Position<'a> {
    root_node: Node<'a>,
    inner: InnerPosition<'a>,
}

impl<'a> PartialEq for Position<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.root_node.id() == other.root_node.id() && self.inner == other.inner
    }
}

impl<'a> Eq for Position<'a> {}

impl<'a> PartialOrd for Position<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let root_node_id = self.root_node.id();
        if root_node_id != other.root_node.id() {
            return None;
        }
        let self_comparable = self.inner.comparable(root_node_id);
        let other_comparable = other.inner.comparable(root_node_id);
        Some(self_comparable.cmp(&other_comparable))
    }
}

#[derive(Clone, Copy)]
pub struct Range<'a> {
    node: Node<'a>,
    start: InnerPosition<'a>,
    end: InnerPosition<'a>,
}

impl<'a> Range<'a> {
    fn new(node: Node<'a>, mut start: InnerPosition<'a>, mut end: InnerPosition<'a>) -> Self {
        if start.comparable(node.id()) > end.comparable(node.id()) {
            std::mem::swap(&mut start, &mut end);
        }
        Self { node, start, end }
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn start(&self) -> Position {
        Position {
            root_node: self.node,
            inner: self.start,
        }
    }

    pub fn end(&self) -> Position {
        Position {
            root_node: self.node,
            inner: self.end,
        }
    }

    pub fn downgrade(&self) -> WeakRange {
        WeakRange {
            node_id: self.node.id(),
            start: self.start.downgrade(),
            end: self.end.downgrade(),
        }
    }
}

impl<'a> PartialEq for Range<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.node.id() == other.node.id() && self.start == other.start && self.end == other.end
    }
}

impl<'a> Eq for Range<'a> {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeakRange {
    node_id: NodeId,
    start: WeakPosition,
    end: WeakPosition,
}

impl WeakRange {
    pub fn upgrade<'a>(&self, tree_state: &'a TreeState) -> Option<Range<'a>> {
        let node = tree_state.node_by_id(self.node_id)?;
        let start = InnerPosition::upgrade(tree_state, self.start)?;
        let end = InnerPosition::upgrade(tree_state, self.end)?;
        Some(Range { node, start, end })
    }
}

impl<'a> Node<'a> {
    fn text_node_filter(&self, node: &Node) -> FilterResult {
        if node.id() == self.id() || (node.role() == Role::InlineTextBox && !node.is_hidden()) {
            FilterResult::Include
        } else {
            FilterResult::ExcludeNode
        }
    }

    pub fn supports_text_ranges(&self) -> bool {
        let role = self.role();
        if role != Role::StaticText && role != Role::TextField && role != Role::Document {
            return false;
        }
        self.filtered_children(|node| self.text_node_filter(node))
            .next()
            .is_some()
    }

    fn document_endpoints(&self) -> (InnerPosition, InnerPosition) {
        let mut boxes = self.filtered_children(|node| self.text_node_filter(node));
        let first_box = boxes.next().unwrap();
        let start = InnerPosition {
            node: first_box,
            character_index: 0,
        };
        let last_box = boxes.next_back().unwrap();
        let end = InnerPosition {
            node: last_box,
            character_index: last_box.data().character_end_indices.len() as u16,
        };
        (start, end)
    }

    pub fn document_range(&self) -> Range {
        let (start, end) = self.document_endpoints();
        Range::new(*self, start, end)
    }

    pub fn has_text_selection(&self) -> bool {
        self.data().text_selection.is_some()
    }

    pub fn text_selection(&self) -> Option<Range> {
        self.data().text_selection.map(|selection| {
            let anchor = InnerPosition::upgrade(self.tree_state, selection.anchor).unwrap();
            let focus = InnerPosition::upgrade(self.tree_state, selection.focus).unwrap();
            Range::new(*self, anchor, focus)
        })
    }
}
