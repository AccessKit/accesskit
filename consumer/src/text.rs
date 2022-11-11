// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::kurbo::{Point, Rect};
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

    pub fn to_degenerate_range(&self) -> Range {
        Range::new(self.root_node, self.inner, self.inner)
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
            if current.is_document_end()
                || current
                    .inner
                    .biased_to_end(&self.root_node)
                    .is_paragraph_end()
            {
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
        // If the range is degenerate, we don't want to normalize it.
        // This is important e.g. when getting the bounding rectangle
        // of the caret range when the caret is at the end of a wrapped line.
        let (start, end) = if self.is_degenerate() {
            (self.start, self.start)
        } else {
            let start = self.start.biased_to_start(&self.node);
            let end = self.end.biased_to_end(&self.node);
            (start, end)
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
                    // Note: The following directions assume that the rectangle,
                    // in the node's coordinate space, is y-down. TBD: Will we
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

fn character_index_at_point(node: &Node, point: Point) -> usize {
    // We know the node has a bounding rectangle because it was returned
    // by a hit test.
    let rect = node.data().bounds.as_ref().unwrap();
    let character_lengths = &node.data().character_lengths;
    let positions = match &node.data().character_positions {
        Some(positions) => positions,
        None => {
            return 0;
        }
    };
    let widths = match &node.data().character_widths {
        Some(widths) => widths,
        None => {
            return 0;
        }
    };
    let direction = match node.data().text_direction {
        Some(direction) => direction,
        None => {
            return 0;
        }
    };
    for (i, (position, width)) in positions.iter().zip(widths.iter()).enumerate().rev() {
        let relative_pos = match direction {
            TextDirection::LeftToRight => point.x - rect.x0,
            TextDirection::RightToLeft => rect.x1 - point.x,
            // Note: The following directions assume that the rectangle,
            // in the node's coordinate space, is y-down. TBD: Will we
            // ever encounter a case where this isn't true?
            TextDirection::TopToBottom => point.y - rect.y0,
            TextDirection::BottomToTop => rect.y1 - point.y,
        };
        if relative_pos >= f64::from(*position) && relative_pos < f64::from(*position + *width) {
            return i;
        }
    }
    character_lengths.len()
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

    /// Returns the nearest text position to the given point
    /// in this node's coordinate space.
    pub fn text_position_at_point(&self, point: Point) -> Position {
        let id = self.id();
        if let Some((node, point)) = self.hit_test(point, &move |node| text_node_filter(id, node)) {
            if node.role() == Role::InlineTextBox {
                let pos = InnerPosition {
                    node,
                    character_index: character_index_at_point(&node, point),
                };
                return Position {
                    root_node: *self,
                    inner: pos,
                };
            }
        }

        // The following tests can assume that the point is not within
        // any inline text box.

        if let Some(node) = self.inline_text_boxes().next() {
            if let Some(rect) = node.bounding_box_in_coordinate_space(self) {
                let origin = rect.origin();
                if point.x < origin.x || point.y < origin.y {
                    return Position {
                        root_node: *self,
                        inner: self.document_start(),
                    };
                }
            }
        }

        for node in self.inline_text_boxes() {
            if let Some(rect) = node.bounding_box_in_coordinate_space(self) {
                if let Some(direction) = node.data().text_direction {
                    let is_past_end = match direction {
                        TextDirection::LeftToRight => {
                            point.y >= rect.y0 && point.y < rect.y1 && point.x >= rect.x1
                        }
                        TextDirection::RightToLeft => {
                            point.y >= rect.y0 && point.y < rect.y1 && point.x < rect.x0
                        }
                        // Note: The following directions assume that the rectangle,
                        // in the root node's coordinate space, is y-down. TBD: Will we
                        // ever encounter a case where this isn't true?
                        TextDirection::TopToBottom => {
                            point.x >= rect.x0 && point.x < rect.x1 && point.y >= rect.y1
                        }
                        TextDirection::BottomToTop => {
                            point.x >= rect.x0 && point.x < rect.x1 && point.y < rect.y0
                        }
                    };
                    if is_past_end {
                        return Position {
                            root_node: *self,
                            inner: InnerPosition {
                                node,
                                character_index: node.data().character_lengths.len(),
                            },
                        };
                    }
                }
            }
        }

        Position {
            root_node: *self,
            inner: self.document_end(),
        }
    }
}

#[cfg(test)]
mod tests {
    use accesskit::kurbo::{Point, Rect};
    use accesskit::{NodeId, TextSelection};
    use std::{num::NonZeroU128, sync::Arc};

    use crate::tests::NullActionHandler;

    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
    const NODE_ID_4: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
    const NODE_ID_5: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(5) });
    const NODE_ID_6: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(6) });
    const NODE_ID_7: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(7) });
    const NODE_ID_8: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(8) });

    // This is based on an actual tree produced by egui.
    fn main_multiline_tree(selection: Option<TextSelection>) -> crate::Tree {
        use accesskit::kurbo::Affine;
        use accesskit::{Node, Role, TextDirection, Tree, TreeUpdate};

        let update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        role: Role::Window,
                        transform: Some(Box::new(Affine::scale(1.5))),
                        children: vec![NODE_ID_2],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::TextField,
                        bounds: Some(Rect {
                            x0: 8.0,
                            y0: 31.666664123535156,
                            x1: 296.0,
                            y1: 123.66666412353516,
                        }),
                        children: vec![
                            NODE_ID_3, NODE_ID_4, NODE_ID_5, NODE_ID_6, NODE_ID_7, NODE_ID_8,
                        ],
                        focusable: true,
                        text_selection: selection,
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_3,
                    Arc::new(Node {
                        role: Role::InlineTextBox,
                        bounds: Some(Rect {
                            x0: 12.0,
                            y0: 33.666664123535156,
                            x1: 290.9189147949219,
                            y1: 48.33333206176758,
                        }),
                        value: Some("This paragraph is long enough to wrap ".into()),
                        text_direction: Some(TextDirection::LeftToRight),
                        character_lengths: vec![
                            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        ]
                        .into(),
                        character_positions: Some(
                            vec![
                                0.0, 7.3333335, 14.666667, 22.0, 29.333334, 36.666668, 44.0,
                                51.333332, 58.666668, 66.0, 73.333336, 80.666664, 88.0, 95.333336,
                                102.666664, 110.0, 117.333336, 124.666664, 132.0, 139.33333,
                                146.66667, 154.0, 161.33333, 168.66667, 176.0, 183.33333,
                                190.66667, 198.0, 205.33333, 212.66667, 220.0, 227.33333,
                                234.66667, 242.0, 249.33333, 256.66666, 264.0, 271.33334,
                            ]
                            .into(),
                        ),
                        character_widths: Some(
                            vec![
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557,
                            ]
                            .into(),
                        ),
                        word_lengths: vec![5, 10, 3, 5, 7, 3, 5].into(),
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_4,
                    Arc::new(Node {
                        role: Role::InlineTextBox,
                        bounds: Some(Rect {
                            x0: 12.0,
                            y0: 48.33333206176758,
                            x1: 129.5855712890625,
                            y1: 63.0,
                        }),
                        value: Some("to another line.\n".into()),
                        text_direction: Some(TextDirection::LeftToRight),
                        character_lengths: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
                            .into(),
                        character_positions: Some(
                            vec![
                                0.0, 7.3333435, 14.666687, 22.0, 29.333344, 36.666687, 44.0,
                                51.333344, 58.666687, 66.0, 73.33334, 80.66669, 88.0, 95.33334,
                                102.66669, 110.0, 117.58557,
                            ]
                            .into(),
                        ),
                        character_widths: Some(
                            vec![
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 0.0,
                            ]
                            .into(),
                        ),
                        word_lengths: vec![3, 8, 6].into(),
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_5,
                    Arc::new(Node {
                        role: Role::InlineTextBox,
                        bounds: Some(Rect {
                            x0: 12.0,
                            y0: 63.0,
                            x1: 144.25222778320313,
                            y1: 77.66666412353516,
                        }),
                        value: Some("Another paragraph.\n".into()),
                        text_direction: Some(TextDirection::LeftToRight),
                        character_lengths: vec![
                            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        ]
                        .into(),
                        character_positions: Some(
                            vec![
                                0.0, 7.3333335, 14.666667, 22.0, 29.333334, 36.666668, 44.0,
                                51.333332, 58.666668, 66.0, 73.333336, 80.666664, 88.0, 95.333336,
                                102.666664, 110.0, 117.333336, 124.666664, 132.25223,
                            ]
                            .into(),
                        ),
                        character_widths: Some(
                            vec![
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 0.0,
                            ]
                            .into(),
                        ),
                        word_lengths: vec![8, 11].into(),
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_6,
                    Arc::new(Node {
                        role: Role::InlineTextBox,
                        bounds: Some(Rect {
                            x0: 12.0,
                            y0: 77.66666412353516,
                            x1: 12.0,
                            y1: 92.33332824707031,
                        }),
                        value: Some("\n".into()),
                        text_direction: Some(TextDirection::LeftToRight),
                        character_lengths: vec![1].into(),
                        character_positions: Some(vec![0.0].into()),
                        character_widths: Some(vec![0.0].into()),
                        word_lengths: vec![1].into(),
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_7,
                    Arc::new(Node {
                        role: Role::InlineTextBox,
                        bounds: Some(Rect {
                            x0: 12.0,
                            y0: 92.33332824707031,
                            x1: 158.9188995361328,
                            y1: 107.0,
                        }),
                        value: Some("Last non-blank line.\n".into()),
                        text_direction: Some(TextDirection::LeftToRight),
                        character_lengths: vec![
                            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        ]
                        .into(),
                        character_positions: Some(
                            vec![
                                0.0, 7.3333335, 14.666667, 22.0, 29.333334, 36.666668, 44.0,
                                51.333332, 58.666668, 66.0, 73.333336, 80.666664, 88.0, 95.333336,
                                102.666664, 110.0, 117.333336, 124.666664, 132.0, 139.33333,
                                146.9189,
                            ]
                            .into(),
                        ),
                        character_widths: Some(
                            vec![
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557,
                                7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 7.58557, 0.0,
                            ]
                            .into(),
                        ),
                        word_lengths: vec![5, 4, 6, 6].into(),
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_8,
                    Arc::new(Node {
                        role: Role::InlineTextBox,
                        bounds: Some(Rect {
                            x0: 12.0,
                            y0: 107.0,
                            x1: 12.0,
                            y1: 121.66666412353516,
                        }),
                        value: Some("".into()),
                        text_direction: Some(TextDirection::LeftToRight),
                        character_lengths: vec![].into(),
                        character_positions: Some(vec![].into()),
                        character_widths: Some(vec![].into()),
                        word_lengths: vec![0].into(),
                        ..Default::default()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: Some(NODE_ID_2),
        };

        crate::Tree::new(update, Box::new(NullActionHandler {}))
    }

    fn multiline_end_selection() -> TextSelection {
        use accesskit::TextPosition;

        TextSelection {
            anchor: TextPosition {
                node: NODE_ID_8,
                character_index: 0,
            },
            focus: TextPosition {
                node: NODE_ID_8,
                character_index: 0,
            },
        }
    }

    fn multiline_wrapped_line_end_selection() -> TextSelection {
        use accesskit::TextPosition;

        TextSelection {
            anchor: TextPosition {
                node: NODE_ID_3,
                character_index: 38,
            },
            focus: TextPosition {
                node: NODE_ID_3,
                character_index: 38,
            },
        }
    }

    fn multiline_second_line_middle_selection() -> TextSelection {
        use accesskit::TextPosition;

        TextSelection {
            anchor: TextPosition {
                node: NODE_ID_4,
                character_index: 5,
            },
            focus: TextPosition {
                node: NODE_ID_4,
                character_index: 5,
            },
        }
    }

    #[test]
    fn supports_text_ranges() {
        let tree = main_multiline_tree(None);
        let state = tree.read();
        assert!(!state.node_by_id(NODE_ID_1).unwrap().supports_text_ranges());
        assert!(state.node_by_id(NODE_ID_2).unwrap().supports_text_ranges());
    }

    #[test]
    fn multiline_document_range() {
        let tree = main_multiline_tree(None);
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();
        let range = node.document_range();
        let start = range.start();
        assert!(start.is_word_start());
        assert!(start.is_line_start());
        assert!(!start.is_line_end());
        assert!(start.is_paragraph_start());
        assert!(start.is_document_start());
        assert!(!start.is_document_end());
        let end = range.end();
        assert!(start < end);
        assert!(end.is_word_start());
        assert!(end.is_line_start());
        assert!(end.is_line_end());
        assert!(end.is_paragraph_start());
        assert!(!end.is_document_start());
        assert!(end.is_document_end());
        assert_eq!(range.text(), "This paragraph is long enough to wrap to another line.\nAnother paragraph.\n\nLast non-blank line.\n");
        assert_eq!(
            range.bounding_boxes(),
            vec![
                Rect {
                    x0: 18.0,
                    y0: 50.499996185302734,
                    x1: 436.3783721923828,
                    y1: 72.49999809265137
                },
                Rect {
                    x0: 18.0,
                    y0: 72.49999809265137,
                    x1: 194.37835693359375,
                    y1: 94.5
                },
                Rect {
                    x0: 18.0,
                    y0: 94.5,
                    x1: 216.3783416748047,
                    y1: 116.49999618530273
                },
                Rect {
                    x0: 18.0,
                    y0: 116.49999618530273,
                    x1: 18.0,
                    y1: 138.49999237060547
                },
                Rect {
                    x0: 18.0,
                    y0: 138.49999237060547,
                    x1: 238.37834930419922,
                    y1: 160.5
                }
            ]
        );
    }

    #[test]
    fn multiline_end_degenerate_range() {
        let tree = main_multiline_tree(Some(multiline_end_selection()));
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();
        let range = node.text_selection().unwrap();
        assert!(range.is_degenerate());
        let pos = range.start();
        assert!(pos.is_word_start());
        assert!(pos.is_line_start());
        assert!(pos.is_line_end());
        assert!(pos.is_paragraph_start());
        assert!(!pos.is_document_start());
        assert!(pos.is_document_end());
        assert_eq!(range.text(), "");
        assert_eq!(
            range.bounding_boxes(),
            vec![Rect {
                x0: 18.0,
                y0: 160.5,
                x1: 18.0,
                y1: 182.49999618530273,
            }]
        );
    }

    #[test]
    fn multiline_wrapped_line_end_range() {
        let tree = main_multiline_tree(Some(multiline_wrapped_line_end_selection()));
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();
        let range = node.text_selection().unwrap();
        assert!(range.is_degenerate());
        let pos = range.start();
        assert!(!pos.is_word_start());
        assert!(!pos.is_line_start());
        assert!(pos.is_line_end());
        assert!(!pos.is_paragraph_start());
        assert!(!pos.is_document_start());
        assert!(!pos.is_document_end());
        assert_eq!(range.text(), "");
        assert_eq!(
            range.bounding_boxes(),
            vec![Rect {
                x0: 436.3783721923828,
                y0: 50.499996185302734,
                x1: 436.3783721923828,
                y1: 72.49999809265137
            }]
        );
        let next_char_pos = pos.forward_by_character();
        let mut line_start_range = range;
        line_start_range.set_end(next_char_pos);
        assert!(!line_start_range.is_degenerate());
        assert_eq!(line_start_range.text(), "t");
        assert_eq!(
            line_start_range.bounding_boxes(),
            vec![Rect {
                x0: 18.0,
                y0: 72.49999809265137,
                x1: 29.378354787826538,
                y1: 94.5
            }]
        );
        let prev_char_pos = pos.backward_by_character();
        let mut prev_char_range = range;
        prev_char_range.set_start(prev_char_pos);
        assert!(!prev_char_range.is_degenerate());
        assert_eq!(prev_char_range.text(), " ");
        assert_eq!(
            prev_char_range.bounding_boxes(),
            vec![Rect {
                x0: 425.00001525878906,
                y0: 50.499996185302734,
                x1: 436.3783721923828,
                y1: 72.49999809265137
            }]
        );
    }

    #[test]
    fn multiline_find_line_ends_from_middle() {
        let tree = main_multiline_tree(Some(multiline_second_line_middle_selection()));
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();
        let mut range = node.text_selection().unwrap();
        assert!(range.is_degenerate());
        let pos = range.start();
        assert!(!pos.is_line_start());
        assert!(!pos.is_line_end());
        assert!(!pos.is_document_start());
        assert!(!pos.is_document_end());
        let line_start = pos.backward_by_line();
        range.set_start(line_start);
        let line_end = line_start.forward_by_line();
        range.set_end(line_end);
        assert!(!range.is_degenerate());
        assert_eq!(range.text(), "to another line.\n");
        assert_eq!(
            range.bounding_boxes(),
            vec![Rect {
                x0: 18.0,
                y0: 72.49999809265137,
                x1: 194.37835693359375,
                y1: 94.5
            },]
        );
    }

    #[test]
    fn multiline_find_paragraph_ends_from_middle() {
        let tree = main_multiline_tree(Some(multiline_second_line_middle_selection()));
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();
        let mut range = node.text_selection().unwrap();
        assert!(range.is_degenerate());
        let pos = range.start();
        assert!(!pos.is_paragraph_start());
        assert!(!pos.is_document_start());
        assert!(!pos.is_document_end());
        let paragraph_start = pos.backward_by_paragraph();
        range.set_start(paragraph_start);
        let paragraph_end = paragraph_start.forward_by_paragraph();
        range.set_end(paragraph_end);
        assert!(!range.is_degenerate());
        assert_eq!(
            range.text(),
            "This paragraph is long enough to wrap to another line.\n"
        );
        assert_eq!(
            range.bounding_boxes(),
            vec![
                Rect {
                    x0: 18.0,
                    y0: 50.499996185302734,
                    x1: 436.3783721923828,
                    y1: 72.49999809265137
                },
                Rect {
                    x0: 18.0,
                    y0: 72.49999809265137,
                    x1: 194.37835693359375,
                    y1: 94.5
                },
            ]
        );
    }

    #[test]
    fn multiline_find_word_ends_from_middle() {
        let tree = main_multiline_tree(Some(multiline_second_line_middle_selection()));
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();
        let mut range = node.text_selection().unwrap();
        assert!(range.is_degenerate());
        let pos = range.start();
        assert!(!pos.is_word_start());
        assert!(!pos.is_document_start());
        assert!(!pos.is_document_end());
        let word_start = pos.backward_by_word();
        range.set_start(word_start);
        let word_end = word_start.forward_by_word();
        range.set_end(word_end);
        assert!(!range.is_degenerate());
        assert_eq!(range.text(), "another ");
        assert_eq!(
            range.bounding_boxes(),
            vec![Rect {
                x0: 51.0,
                y0: 72.49999809265137,
                x1: 139.3783721923828,
                y1: 94.5
            }]
        );
    }

    #[test]
    fn text_position_at_point() {
        let tree = main_multiline_tree(None);
        let state = tree.read();
        let node = state.node_by_id(NODE_ID_2).unwrap();

        {
            let pos = node.text_position_at_point(Point::new(8.0, 31.666664123535156));
            assert!(pos.is_document_start());
        }

        {
            let pos = node.text_position_at_point(Point::new(12.0, 33.666664123535156));
            assert!(pos.is_document_start());
        }

        {
            let pos = node.text_position_at_point(Point::new(16.0, 40.0));
            assert!(pos.is_document_start());
        }

        {
            let pos = node.text_position_at_point(Point::new(144.0, 40.0));
            assert!(!pos.is_document_start());
            assert!(!pos.is_document_end());
            assert!(!pos.is_line_end());
            let mut range = pos.to_degenerate_range();
            range.set_end(pos.forward_by_character());
            assert_eq!(range.text(), "l");
        }

        {
            let pos = node.text_position_at_point(Point::new(150.0, 40.0));
            assert!(!pos.is_document_start());
            assert!(!pos.is_document_end());
            assert!(!pos.is_line_end());
            let mut range = pos.to_degenerate_range();
            range.set_end(pos.forward_by_character());
            assert_eq!(range.text(), "l");
        }

        {
            let pos = node.text_position_at_point(Point::new(291.0, 40.0));
            assert!(!pos.is_document_start());
            assert!(!pos.is_document_end());
            assert!(pos.is_line_end());
            let mut range = pos.to_degenerate_range();
            range.set_start(pos.backward_by_word());
            assert_eq!(range.text(), "wrap ");
        }

        {
            let pos = node.text_position_at_point(Point::new(12.0, 50.0));
            assert!(!pos.is_document_start());
            assert!(pos.is_line_start());
            assert!(!pos.is_paragraph_start());
            let mut range = pos.to_degenerate_range();
            range.set_end(pos.forward_by_word());
            assert_eq!(range.text(), "to ");
        }

        {
            let pos = node.text_position_at_point(Point::new(130.0, 50.0));
            assert!(!pos.is_document_start());
            assert!(!pos.is_document_end());
            assert!(pos.is_line_end());
            let mut range = pos.to_degenerate_range();
            range.set_start(pos.backward_by_word());
            assert_eq!(range.text(), "line.\n");
        }

        {
            let pos = node.text_position_at_point(Point::new(12.0, 80.0));
            assert!(!pos.is_document_start());
            assert!(!pos.is_document_end());
            assert!(pos.is_line_end());
            let mut range = pos.to_degenerate_range();
            range.set_start(pos.backward_by_line());
            assert_eq!(range.text(), "\n");
        }

        {
            let pos = node.text_position_at_point(Point::new(12.0, 120.0));
            assert!(pos.is_document_end());
        }

        {
            let pos = node.text_position_at_point(Point::new(250.0, 122.0));
            assert!(pos.is_document_end());
        }
    }
}
