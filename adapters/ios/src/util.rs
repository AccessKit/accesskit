// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::NodeRef;
use objc2_foundation::{NSAttributedStringKey, NSPoint, NSRect, NSSize, NSString};
use objc2_ui_kit::{
    UIAccessibilityConvertFrameToScreenCoordinates, UIAccessibilityPriority, UIAccessibilityTraits,
    UICoordinateSpace, UIView,
};
use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

pub(crate) fn from_cg_point(view: &UIView, node: &NodeRef, point: NSPoint) -> Option<Point> {
    let window = view.window()?;
    let screen_space = window.screen().coordinateSpace();
    let local_point = view.convertPoint_fromCoordinateSpace(point, &screen_space);
    let factor = view.contentScaleFactor();
    let point = Point::new(local_point.x * factor, local_point.y * factor);
    Some(node.transform().inverse() * point)
}

pub(crate) fn to_screen_rect(view: &UIView, rect: NSRect) -> NSRect {
    UIAccessibilityConvertFrameToScreenCoordinates(rect, view)
}

pub(crate) fn to_cg_rect(view: &UIView, rect: accesskit::Rect) -> NSRect {
    let factor = view.contentScaleFactor();
    let local_rect = NSRect {
        origin: NSPoint {
            x: rect.x0 / factor,
            y: rect.y0 / factor,
        },
        size: NSSize {
            width: rect.width() / factor,
            height: rect.height() / factor,
        },
    };
    to_screen_rect(view, local_rect)
}

// This handle is used as a parameter to dlsym to perform symbol searches at
// runtime. When a symbol search returns null, its usage can be omitted on
// unsupported iOS versions. See:
// https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/dlsym.3.html
// and https://github.com/apple-oss-distributions/dyld/blob/main/include/dlfcn.h
// for more information.
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
