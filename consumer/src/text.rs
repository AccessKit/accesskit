// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::kurbo::Rect;
use accesskit::{NodeId, Role, TextDirection, TextPosition as WeakPosition};
use std::{cmp::Ordering, iter::FusedIterator};

use crate::{FilterResult, Node, TreeState};

#[derive(Clone, Copy)]
pub(crate) struct InnerPosition<'a> {
    pub(crate) node: Node<'a>,
    pub(crate) character_index: usize,
}

impl<'a> InnerPosition<'a> {
    fn upgrade(tree_state: &'a TreeState, weak: WeakPosition) -> Option<Self> {
        let node = tree_state.node_by_id(weak.node)?;
        if node.role() != Role::InlineTextBox {
            return None;
        }
        let character_index = weak.character_index;
        if character_index > node.data().character_lengths.len() {
            return None;
        }
        Some(Self {
            node,
            character_index,
        })
    }

    fn is_word_start(&self) -> bool {
        let mut total_length = 0usize;
        for length in self.node.data().word_lengths.iter() {
            if total_length == self.character_index {
                return true;
            }
            total_length += *length as usize;
        }
        false
    }

    fn is_box_start(&self) -> bool {
        self.character_index == 0
    }

    fn is_line_start(&self) -> bool {
        self.is_box_start() && self.node.data().previous_on_line.is_none()
    }

    fn is_box_end(&self) -> bool {
        self.character_index == self.node.data().character_lengths.len()
    }

    fn is_line_end(&self) -> bool {
        self.is_box_end() && self.node.data().next_on_line.is_none()
    }

    fn is_paragraph_end(&self) -> bool {
        self.is_line_end() && self.node.value().unwrap().ends_with('\n')
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

    fn biased_to_start(&self, root_node: &Node) -> Self {
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

    fn biased_to_end(&self, root_node: &Node) -> Self {
        if self.is_box_start() {
            if let Some(node) = self.node.preceding_inline_text_boxes(root_node).next() {
                return Self {
                    node,
                    character_index: node.data().character_lengths.len(),
                };
            }
        }
        *self
    }

    fn normalized(&self, root_node: &Node) -> Self {
        if self.is_line_end() && !self.is_paragraph_end() {
            *self
        } else {
            self.biased_to_start(root_node)
        }
    }

    fn comparable(&self, root_node: &Node) -> (Vec<usize>, usize) {
        let normalized = self.biased_to_start(root_node);
        (
            normalized.node.relative_index_path(root_node.id()),
            normalized.character_index,
        )
    }

    fn previous_word_start(&self) -> Self {
        let mut total_length_before = 0usize;
        for length in self.node.data().word_lengths.iter() {
            let new_total_length = total_length_before + (*length as usize);
            if new_total_length >= self.character_index {
                break;
            }
            total_length_before = new_total_length;
        }
        Self {
            node: self.node,
            character_index: total_length_before,
        }
    }

    fn word_end(&self) -> Self {
        let mut total_length = 0usize;
        for length in self.node.data().word_lengths.iter() {
            total_length += *length as usize;
            if total_length > self.character_index {
                break;
            }
        }
        Self {
            node: self.node,
            character_index: total_length,
        }
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
            character_index: node.data().character_lengths.len(),
        }
    }

    pub(crate) fn downgrade(&self) -> WeakPosition {
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
    pub(crate) inner: InnerPosition<'a>,
}

impl<'a> Position<'a> {
    pub fn is_format_start(&self) -> bool {
        // TODO: support variable text formatting (part of rich text)
        self.is_document_start()
    }

    pub fn is_word_start(&self) -> bool {
        self.inner.is_word_start()
    }

    pub fn is_line_start(&self) -> bool {
        self.inner.is_line_start()
    }

    pub fn is_line_end(&self) -> bool {
        self.inner.is_line_end()
    }

    pub fn is_paragraph_start(&self) -> bool {
        self.is_document_start()
            || (self.is_line_start()
                && self.inner.biased_to_end(&self.root_node).is_paragraph_end())
    }

    pub fn is_page_start(&self) -> bool {
        self.is_document_start()
    }

    pub fn is_document_start(&self) -> bool {
        self.inner.is_document_start(&self.root_node)
    }

    pub fn is_document_end(&self) -> bool {
        self.inner.is_document_end(&self.root_node)
    }

    pub fn forward_by_character(&self) -> Self {
        let pos = self.inner.biased_to_start(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: InnerPosition {
                node: pos.node,
                character_index: pos.character_index + 1,
            }
            .normalized(&self.root_node),
        }
    }

    pub fn backward_by_character(&self) -> Self {
        let pos = self.inner.biased_to_end(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: InnerPosition {
                node: pos.node,
                character_index: pos.character_index - 1,
            }
            .normalized(&self.root_node),
        }
    }

    pub fn forward_by_format(&self) -> Self {
        // TODO: support variable text formatting (part of rich text)
        self.forward_by_document()
    }

    pub fn backward_by_format(&self) -> Self {
        // TODO: support variable text formatting (part of rich text)
        self.backward_by_document()
    }

    pub fn forward_by_word(&self) -> Self {
        let pos = self.inner.biased_to_start(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: pos.word_end().normalized(&self.root_node),
        }
    }

    pub fn backward_by_word(&self) -> Self {
        let pos = self.inner.biased_to_end(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: pos.previous_word_start().normalized(&self.root_node),
        }
    }

    pub fn forward_by_line(&self) -> Self {
        let pos = self.inner.biased_to_start(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: pos.line_end().normalized(&self.root_node),
        }
    }

    pub fn backward_by_line(&self) -> Self {
        let pos = self.inner.biased_to_end(&self.root_node);
        Self {
            root_node: self.root_node,
            inner: pos.line_start().normalized(&self.root_node),
        }
    }

    pub fn forward_by_paragraph(&self) -> Self {
        let mut current = *self;
        loop {
            current = current.forward_by_line();
            if current.is_document_end() || current.inner.is_paragraph_end() {
                break;
            }
        }
        current
    }

    pub fn backward_by_paragraph(&self) -> Self {
        let mut current = *self;
        loop {
            current = current.backward_by_line();
            if current.is_paragraph_start() {
                break;
            }
        }
        current
    }

    pub fn forward_by_page(&self) -> Self {
        self.forward_by_document()
    }

    pub fn backward_by_page(&self) -> Self {
        self.backward_by_document()
    }

    pub fn forward_by_document(&self) -> Self {
        Self {
            root_node: self.root_node,
            inner: self.root_node.document_end(),
        }
    }

    pub fn backward_by_document(&self) -> Self {
        Self {
            root_node: self.root_node,
            inner: self.root_node.document_start(),
        }
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

pub enum AttributeValue<T> {
    Single(T),
    Mixed,
}

#[derive(Clone, Copy)]
pub struct Range<'a> {
    pub(crate) node: Node<'a>,
    pub(crate) start: InnerPosition<'a>,
    pub(crate) end: InnerPosition<'a>,
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

    pub fn is_degenerate(&self) -> bool {
        self.start.comparable(&self.node) == self.end.comparable(&self.node)
    }

    fn walk<F, T>(&self, mut f: F) -> Option<T>
    where
        F: FnMut(&Node) -> Option<T>,
    {
        let start = self.start.biased_to_start(&self.node);
        // For a degenerate range, the following avoids having `end`
        // come before `start`.
        let end = if self.is_degenerate() {
            start
        } else {
            self.end.biased_to_end(&self.node)
        };
        if let Some(result) = f(&start.node) {
            return Some(result);
        }
        if start.node.id() == end.node.id() {
            return None;
        }
        for node in start.node.following_inline_text_boxes(&self.node) {
            if let Some(result) = f(&node) {
                return Some(result);
            }
            if node.id() == end.node.id() {
                break;
            }
        }
        None
    }

    pub fn text(&self) -> String {
        let mut result = String::new();
        self.walk::<_, ()>(|node| {
            let character_lengths = &node.data().character_lengths;
            let start_index = if node.id() == self.start.node.id() {
                self.start.character_index
            } else {
                0
            };
            let end_index = if node.id() == self.end.node.id() {
                self.end.character_index
            } else {
                character_lengths.len()
            };
            let value = node.value().unwrap();
            let s = if start_index == end_index {
                ""
            } else if start_index == 0 && end_index == character_lengths.len() {
                value
            } else {
                let slice_start = character_lengths[..start_index]
                    .iter()
                    .copied()
                    .map(usize::from)
                    .sum::<usize>();
                let slice_end = slice_start
                    + character_lengths[start_index..end_index]
                        .iter()
                        .copied()
                        .map(usize::from)
                        .sum::<usize>();
                &value[slice_start..slice_end]
            };
            result.push_str(s);
            None
        });
        result
    }

    /// Returns the range's transformed bounding boxes relative to the tree's
    /// container (e.g. window).
    ///
    /// If the return value is empty, it means that the source tree doesn't
    /// provide enough information to calculate bounding boxes. Otherwise,
    /// there will always be at least one box, even if it's zero-width,
    /// as it is for a degenerate range.
    pub fn bounding_boxes(&self) -> Vec<Rect> {
        let mut result = Vec::new();
        self.walk(|node| {
            let mut rect = match &node.data().bounds {
                Some(rect) => *rect,
                None => {
                    return Some(Vec::new());
                }
            };
            let positions = match &node.data().character_positions {
                Some(positions) => positions,
                None => {
                    return Some(Vec::new());
                }
            };
            let widths = match &node.data().character_widths {
                Some(widths) => widths,
                None => {
                    return Some(Vec::new());
                }
            };
            let direction = match node.data().text_direction {
                Some(direction) => direction,
                None => {
                    return Some(Vec::new());
                }
            };
            let character_lengths = &node.data().character_lengths;
            let start_index = if node.id() == self.start.node.id() {
                self.start.character_index
            } else {
                0
            };
            let end_index = if node.id() == self.end.node.id() {
                self.end.character_index
            } else {
                character_lengths.len()
            };
            if start_index != 0 || end_index != character_lengths.len() {
                let pixel_start = if start_index < character_lengths.len() {
                    positions[start_index]
                } else {
                    positions[start_index - 1] + widths[start_index - 1]
                };
                let pixel_end = if end_index == start_index {
                    pixel_start
                } else {
                    positions[end_index - 1] + widths[end_index - 1]
                };
                let pixel_start = f64::from(pixel_start);
                let pixel_end = f64::from(pixel_end);
                match direction {
                    TextDirection::LeftToRight => {
                        let orig_left = rect.x0;
                        rect.x0 = orig_left + pixel_start;
                        rect.x1 = orig_left + pixel_end;
                    }
                    TextDirection::RightToLeft => {
                        let orig_right = rect.x1;
                        rect.x1 = orig_right - pixel_start;
                        rect.x0 = orig_right - pixel_end;
                    }
                    // Note: The following directions that the rectangle,
                    // before being transformed, is y-down. TBD: Will we
                    // ever encounter a case where this isn't true?
                    TextDirection::TopToBottom => {
                        let orig_top = rect.y0;
                        rect.y0 = orig_top + pixel_start;
                        rect.y1 = orig_top + pixel_end;
                    }
                    TextDirection::BottomToTop => {
                        let orig_bottom = rect.y1;
                        rect.y1 = orig_bottom - pixel_start;
                        rect.y0 = orig_bottom - pixel_end;
                    }
                }
            }
            result.push(node.transform().transform_rect_bbox(rect));
            None
        })
        .unwrap_or(result)
    }

    pub fn attribute<F, T>(&self, f: F) -> AttributeValue<T>
    where
        F: Fn(&Node) -> T,
        T: PartialEq,
    {
        let mut value = None;
        self.walk(|node| {
            let current = f(node);
            if let Some(value) = &value {
                if *value != current {
                    return Some(AttributeValue::Mixed);
                }
            } else {
                value = Some(current);
            }
            None
        })
        .unwrap_or_else(|| AttributeValue::Single(value.unwrap()))
    }

    pub fn set_start(&mut self, pos: Position<'a>) {
        assert_eq!(pos.root_node.id(), self.node.id());
        self.start = pos.inner;
        // We use `>=` here because if the two endpoints are equivalent
        // but with a different bias, we want to normalize the bias.
        if self.start.comparable(&self.node) >= self.end.comparable(&self.node) {
            self.end = self.start;
        }
    }

    pub fn set_end(&mut self, pos: Position<'a>) {
        assert_eq!(pos.root_node.id(), self.node.id());
        self.end = pos.inner;
        // We use `>=` here because if the two endpoints are equivalent
        // but with a different bias, we want to normalize the bias.
        if self.start.comparable(&self.node) >= self.end.comparable(&self.node) {
            self.start = self.end;
        }
    }

    pub fn downgrade(&self) -> WeakRange {
        WeakRange {
            node_id: self.node.id(),
            start: self.start.downgrade(),
            end: self.end.downgrade(),
            start_comparable: self.start.comparable(&self.node),
            end_comparable: self.end.comparable(&self.node),
        }
    }
}

impl<'a> PartialEq for Range<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.node.id() == other.node.id() && self.start == other.start && self.end == other.end
    }
}

impl<'a> Eq for Range<'a> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeakRange {
    node_id: NodeId,
    start: WeakPosition,
    end: WeakPosition,
    start_comparable: (Vec<usize>, usize),
    end_comparable: (Vec<usize>, usize),
}

impl WeakRange {
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn start_comparable(&self) -> &(Vec<usize>, usize) {
        &self.start_comparable
    }

    pub fn end_comparable(&self) -> &(Vec<usize>, usize) {
        &self.end_comparable
    }

    pub fn upgrade_node<'a>(&self, tree_state: &'a TreeState) -> Option<Node<'a>> {
        tree_state.node_by_id(self.node_id)
    }

    pub fn upgrade<'a>(&self, tree_state: &'a TreeState) -> Option<Range<'a>> {
        let node = self.upgrade_node(tree_state)?;
        let start = InnerPosition::upgrade(tree_state, self.start)?;
        let end = InnerPosition::upgrade(tree_state, self.end)?;
        Some(Range { node, start, end })
    }
}

fn text_node_filter(root_id: NodeId, node: &Node) -> FilterResult {
    if node.id() == root_id || node.role() == Role::InlineTextBox {
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
            character_index: node.data().character_lengths.len(),
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
