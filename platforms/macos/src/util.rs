// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Point, Rect};
use accesskit_consumer::{Node, TextPosition, TextRange};
use objc2::foundation::{NSPoint, NSRange, NSRect, NSSize};

use crate::appkit::*;

pub(crate) fn from_ns_range<'a>(node: &'a Node<'a>, ns_range: NSRange) -> Option<TextRange<'a>> {
    let pos = node.text_position_from_global_utf16_index(ns_range.location)?;
    let mut range = pos.to_degenerate_range();
    if ns_range.length > 0 {
        let end =
            node.text_position_from_global_utf16_index(ns_range.location + ns_range.length)?;
        range.set_end(end);
    }
    Some(range)
}

pub(crate) fn to_ns_range(range: &TextRange) -> NSRange {
    let start = range.start().to_global_utf16_index();
    let end = range.end().to_global_utf16_index();
    NSRange::from(start..end)
}

pub(crate) fn to_ns_range_for_character(pos: &TextPosition) -> NSRange {
    let mut range = pos.to_degenerate_range();
    if !pos.is_document_end() {
        range.set_end(pos.forward_to_character_end());
    }
    to_ns_range(&range)
}

pub(crate) fn from_ns_point(view: &NSView, node: &Node, point: NSPoint) -> Point {
    let window = view.window().unwrap();
    let point = window.convert_point_from_screen(point);
    let point = view.convert_point_from_view(point, None);
    // AccessKit coordinates are in physical (DPI-dependent) pixels, but
    // macOS provides logical (DPI-independent) coordinates here.
    let factor = view.backing_scale_factor();
    let point = Point::new(
        point.x * factor,
        if view.is_flipped() {
            point.y * factor
        } else {
            let view_bounds = view.bounds();
            (view_bounds.size.height - point.y) * factor
        },
    );
    node.transform().inverse() * point
}

pub(crate) fn to_ns_rect(view: &NSView, rect: Rect) -> NSRect {
    // AccessKit coordinates are in physical (DPI-dependent)
    // pixels, but macOS expects logical (DPI-independent)
    // coordinates here.
    let factor = view.backing_scale_factor();
    let rect = NSRect {
        origin: NSPoint {
            x: rect.x0 / factor,
            y: if view.is_flipped() {
                rect.y0 / factor
            } else {
                let view_bounds = view.bounds();
                view_bounds.size.height - rect.y1 / factor
            },
        },
        size: NSSize {
            width: rect.width() / factor,
            height: rect.height() / factor,
        },
    };
    let rect = view.convert_rect_to_view(rect, None);
    let window = view.window().unwrap();
    window.convert_rect_to_screen(rect)
}
