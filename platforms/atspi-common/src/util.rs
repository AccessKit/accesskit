// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Point, Rect};
use accesskit_consumer::{Node, TextPosition, TextRange};
use atspi_common::CoordType;

#[derive(Clone, Copy, Default)]
pub struct WindowBounds {
    pub outer: Rect,
    pub inner: Rect,
}

impl WindowBounds {
    pub fn new(outer: Rect, inner: Rect) -> Self {
        Self { outer, inner }
    }

    pub(crate) fn top_left(&self, coord_type: CoordType, is_root: bool) -> Point {
        match coord_type {
            CoordType::Screen if is_root => self.outer.origin(),
            CoordType::Screen => self.inner.origin(),
            CoordType::Window if is_root => Point::ZERO,
            CoordType::Window => {
                let outer_position = self.outer.origin();
                let inner_position = self.inner.origin();
                Point::new(
                    inner_position.x - outer_position.x,
                    inner_position.y - outer_position.y,
                )
            }
            _ => unimplemented!(),
        }
    }
}

pub(crate) fn text_position_from_offset<'a>(
    node: &'a Node,
    offset: i32,
) -> Option<TextPosition<'a>> {
    let index = offset.try_into().ok()?;
    node.text_position_from_global_character_index(index)
}

pub(crate) fn text_range_from_offsets<'a>(
    node: &'a Node,
    start_offset: i32,
    end_offset: i32,
) -> Option<TextRange<'a>> {
    let start = text_position_from_offset(node, start_offset)?;
    let end = if end_offset == -1 {
        node.document_range().end()
    } else {
        text_position_from_offset(node, end_offset)?
    };

    let mut range = start.to_degenerate_range();
    range.set_end(end);
    Some(range)
}

pub(crate) fn text_range_bounds_from_offsets(
    node: &Node,
    start_offset: i32,
    end_offset: i32,
) -> Option<Rect> {
    text_range_from_offsets(node, start_offset, end_offset)?
        .bounding_boxes()
        .into_iter()
        .reduce(|rect1, rect2| rect1.union(rect2))
}
