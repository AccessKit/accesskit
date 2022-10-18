// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{NodeId, Role, TextPosition as WeakPosition};
use std::{cmp::Ordering, iter::FusedIterator};

use crate::{FilterResult, Node, TreeState};

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

    fn is_box_start(&self) -> bool {
        self.character_index == 0
    }

    fn is_box_end(&self) -> bool {
        (self.character_index as usize) == self.node.data().character_end_indices.len()
    }

    fn is_document_start(&self, root_node: &Node) -> bool {
        self.is_box_start()
            && self
                .node
                .preceding_inline_text_boxes(root_node)
                .next()
                .is_none()
    }

    fn is_document_end(&self, root_node: &Node) -> bool {
        self.is_box_end()
            && self
                .node
                .following_inline_text_boxes(root_node)
                .next()
                .is_none()
    }

    fn normalize_to_box_start(&self, root_node: &Node) -> Self {
        if self.is_box_end() {
            if let Some(node) = self.node.following_inline_text_boxes(root_node).next() {
                return Self {
                    node,
                    character_index: 0,
                };
            }
        }
        *self
    }

    fn normalize_to_box_end(&self, root_node: &Node) -> Self {
        if self.is_box_start() {
            if let Some(node) = self.node.preceding_inline_text_boxes(root_node).next() {
                return Self {
                    node,
                    character_index: node.data().character_end_indices.len() as _,
                };
            }
        }
        *self
    }

    fn comparable(&self, root_node: &Node) -> (Vec<usize>, u16) {
        let normalized = self.normalize_to_box_start(root_node);
        (
            normalized.node.relative_index_path(root_node.id()),
            normalized.character_index,
        )
    }

    fn line_start(&self) -> Self {
        let mut node = self.node;
        while let Some(id) = node.data().previous_on_line {
            node = node.tree_state.node_by_id(id).unwrap();
        }
        Self {
            node,
            character_index: 0,
        }
    }

    fn line_end(&self) -> Self {
        let mut node = self.node;
        while let Some(id) = node.data().next_on_line {
            node = node.tree_state.node_by_id(id).unwrap();
        }
        Self {
            node,
            character_index: node.data().character_end_indices.len() as _,
        }
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

impl<'a> Position<'a> {
    pub fn is_document_start(&self) -> bool {
        self.inner.is_document_start(&self.root_node)
    }

    pub fn is_document_end(&self) -> bool {
        self.inner.is_document_end(&self.root_node)
    }

    pub fn forward_by_character(&self) -> Self {
        let normalized = self.inner.normalize_to_box_start(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: InnerPosition {
                node: normalized.node,
                character_index: normalized.character_index + 1,
            },
        }
    }

    pub fn backward_by_character(&self) -> Self {
        let normalized = self.inner.normalize_to_box_end(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: InnerPosition {
                node: normalized.node,
                character_index: normalized.character_index - 1,
            },
        }
    }

    pub fn forward_by_format(&self) -> Self {
        todo!()
    }

    pub fn backward_by_format(&self) -> Self {
        todo!()
    }

    pub fn forward_by_word(&self) -> Self {
        todo!()
    }

    pub fn backward_by_word(&self) -> Self {
        todo!()
    }

    pub fn forward_by_line(&self) -> Self {
        todo!()
    }

    pub fn backward_by_line(&self) -> Self {
        todo!()
    }

    pub fn forward_by_paragraph(&self) -> Self {
        todo!()
    }

    pub fn backward_by_paragraph(&self) -> Self {
        todo!()
    }

    pub fn forward_by_page(&self) -> Self {
        todo!()
    }

    pub fn backward_by_page(&self) -> Self {
        todo!()
    }

    pub fn forward_by_document(&self) -> Self {
        todo!()
    }

    pub fn backward_by_document(&self) -> Self {
        todo!()
    }
}

impl<'a> PartialEq for Position<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.root_node.id() == other.root_node.id() && self.inner == other.inner
    }
}

impl<'a> Eq for Position<'a> {}

impl<'a> PartialOrd for Position<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.root_node.id() != other.root_node.id() {
            return None;
        }
        let self_comparable = self.inner.comparable(&self.root_node);
        let other_comparable = other.inner.comparable(&self.root_node);
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
        if start.comparable(&node) > end.comparable(&node) {
            std::mem::swap(&mut start, &mut end);
        }
        Self { node, start, end }
    }

    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn start(&self) -> Position<'a> {
        Position {
            root_node: self.node,
            inner: self.start,
        }
    }

    pub fn end(&self) -> Position<'a> {
        Position {
            root_node: self.node,
            inner: self.end,
        }
    }

    pub fn expand_to_character(&mut self) {
        todo!()
    }

    pub fn expand_to_format(&mut self) {
        // We don't currently support format runs, so fall back to document.
        self.expand_to_document();
    }

    pub fn expand_to_word(&mut self) {
        todo!()
    }

    pub fn expand_to_line(&mut self) {
        self.start = self.start.line_start();
        self.end = self.start.line_end();
    }

    pub fn expand_to_paragraph(&mut self) {
        todo!()
    }

    pub fn expand_to_page(&mut self) {
        // We don't currently support pages, so fall back to document.
        self.expand_to_document();
    }

    pub fn expand_to_document(&mut self) {
        self.start = self.node.document_start();
        self.end = self.node.document_end();
    }

    pub fn set_start(&mut self, pos: Position<'a>) {
        assert_eq!(pos.root_node.id(), self.node.id());
        self.start = pos.inner;
        if self.start.comparable(&self.node) > self.end.comparable(&self.node) {
            self.end = self.start;
        }
    }

    pub fn set_end(&mut self, pos: Position<'a>) {
        assert_eq!(pos.root_node.id(), self.node.id());
        self.end = pos.inner;
        if self.start.comparable(&self.node) > self.end.comparable(&self.node) {
            self.start = self.end;
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

fn text_node_filter(root_id: NodeId, node: &Node) -> FilterResult {
    if node.id() == root_id || (node.role() == Role::InlineTextBox && !node.is_hidden()) {
        FilterResult::Include
    } else {
        FilterResult::ExcludeNode
    }
}

impl<'a> Node<'a> {
    fn inline_text_boxes(
        &self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        let id = self.id();
        self.filtered_children(move |node| text_node_filter(id, node))
    }

    fn following_inline_text_boxes(
        &self,
        root_node: &Node,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        let id = root_node.id();
        self.following_filtered_siblings(move |node| text_node_filter(id, node))
    }

    fn preceding_inline_text_boxes(
        &self,
        root_node: &Node,
    ) -> impl DoubleEndedIterator<Item = Node<'a>> + FusedIterator<Item = Node<'a>> + 'a {
        let id = root_node.id();
        self.preceding_filtered_siblings(move |node| text_node_filter(id, node))
    }

    pub fn supports_text_ranges(&self) -> bool {
        let role = self.role();
        if role != Role::StaticText && role != Role::TextField && role != Role::Document {
            return false;
        }
        self.inline_text_boxes().next().is_some()
    }

    fn document_start(&self) -> InnerPosition<'a> {
        let node = self.inline_text_boxes().next().unwrap();
        InnerPosition {
            node,
            character_index: 0,
        }
    }

    fn document_end(&self) -> InnerPosition<'a> {
        let node = self.inline_text_boxes().next_back().unwrap();
        InnerPosition {
            node,
            character_index: node.data().character_end_indices.len() as u16,
        }
    }

    pub fn document_range(&self) -> Range {
        let start = self.document_start();
        let end = self.document_end();
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
