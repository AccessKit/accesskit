// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::Node;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2_foundation::{CGPoint, CGRect, CGSize, NSAttributedStringKey, NSInteger, NSString};
use objc2_ui_kit::{
    UIAccessibilityConvertFrameToScreenCoordinates, UIAccessibilityPriority, UIAccessibilityTraits,
    UICoordinateSpace, UIView,
};
use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

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

pub(crate) fn from_cg_point(view: &UIView, node: &Node, point: CGPoint) -> Option<Point> {
    let window = view.window()?;
    let screen_space = window.screen().coordinateSpace();
    let local_point = view.convertPoint_fromCoordinateSpace(point, &screen_space);
    let factor = view.contentScaleFactor();
    let point = Point::new(local_point.x * factor, local_point.y * factor);
    Some(node.transform().inverse() * point)
}

pub(crate) fn to_screen_rect(view: &UIView, rect: CGRect) -> CGRect {
    unsafe { UIAccessibilityConvertFrameToScreenCoordinates(rect, view) }
}

pub(crate) fn to_cg_rect(view: &UIView, rect: accesskit::Rect) -> CGRect {
    let factor = view.contentScaleFactor();
    let local_rect = CGRect {
        origin: CGPoint {
            x: rect.x0 / factor,
            y: rect.y0 / factor,
        },
        size: CGSize {
            width: rect.width() / factor,
            height: rect.height() / factor,
        },
    };
    to_screen_rect(view, local_rect)
}

const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

pub(crate) fn toggle_button_trait() -> UIAccessibilityTraits {
    static TRAIT: OnceLock<UIAccessibilityTraits> = OnceLock::new();
    *TRAIT.get_or_init(|| unsafe {
        let symbol = dlsym(RTLD_DEFAULT, c"UIAccessibilityTraitToggleButton".as_ptr());
        if symbol.is_null() {
            0
        } else {
            *symbol.cast::<UIAccessibilityTraits>()
        }
    })
}

fn resolve_nsstring_const(symbol: *const c_char) -> Option<&'static NSString> {
    unsafe {
        let slot = dlsym(RTLD_DEFAULT, symbol);
        if slot.is_null() {
            None
        } else {
            (*slot.cast::<*const NSString>()).as_ref()
        }
    }
}

pub(crate) fn announcement_priority_high() -> Option<&'static UIAccessibilityPriority> {
    resolve_nsstring_const(c"UIAccessibilityPriorityHigh".as_ptr())
}

pub(crate) fn announcement_priority_low() -> Option<&'static UIAccessibilityPriority> {
    resolve_nsstring_const(c"UIAccessibilityPriorityLow".as_ptr())
}

pub(crate) fn announcement_priority_key() -> Option<&'static NSAttributedStringKey> {
    resolve_nsstring_const(c"UIAccessibilitySpeechAttributeAnnouncementPriority".as_ptr())
}
