// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::Node;
use objc2_foundation::{CGPoint, CGRect, CGSize};
use objc2_ui_kit::UIView;

pub(crate) fn from_cg_point(view: &UIView, node: &Node, point: CGPoint) -> Point {
    let local_point = unsafe { view.convertPoint_fromView(point, None) };
    let insets = view.safeAreaInsets();
    let factor = view.contentScaleFactor();
    let point = Point::new(
        (local_point.x - insets.left) * factor,
        (local_point.y - insets.top) * factor,
    );
    node.transform().inverse() * point
}

pub(crate) fn to_cg_rect(view: &UIView, rect: accesskit::Rect) -> CGRect {
    let insets = view.safeAreaInsets();
    let factor = view.contentScaleFactor();
    let local_rect = CGRect {
        origin: CGPoint {
            x: rect.x0 / factor + insets.left,
            y: rect.y0 / factor + insets.top,
        },
        size: CGSize {
            width: rect.width() / factor,
            height: rect.height() / factor,
        },
    };
    unsafe { view.convertRect_toView(local_rect, None) }
}
