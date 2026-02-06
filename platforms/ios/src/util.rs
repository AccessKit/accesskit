// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::Node;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2_foundation::{CGPoint, CGRect, CGSize, NSInteger};
use objc2_ui_kit::UIView;

// TODO: Remove once we update to objc2 0.6
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct UIAccessibilityExpandedStatus(pub NSInteger);

#[allow(non_upper_case_globals)]
impl UIAccessibilityExpandedStatus {
    pub(crate) const Unsupported: Self = Self(0);
    pub(crate) const Expanded: Self = Self(1);
    pub(crate) const Collapsed: Self = Self(2);
}

unsafe impl Encode for UIAccessibilityExpandedStatus {
    const ENCODING: Encoding = NSInteger::ENCODING;
}

unsafe impl RefEncode for UIAccessibilityExpandedStatus {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

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
