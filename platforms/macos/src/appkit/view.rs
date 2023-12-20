// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use icrate::Foundation::{CGFloat, NSObject, NSPoint, NSRect};
use objc2::{
    extern_class, extern_methods, msg_send_id,
    rc::{Id, Shared},
    ClassType,
};

use super::{NSResponder, NSWindow};

extern_class!(
    #[derive(Debug)]
    pub(crate) struct NSView;

    unsafe impl ClassType for NSView {
        #[inherits(NSObject)]
        type Super = NSResponder;
    }
);

extern_methods!(
    unsafe impl NSView {
        pub(crate) fn window(&self) -> Option<Id<NSWindow, Shared>> {
            unsafe { msg_send_id![self, window] }
        }

        #[method(bounds)]
        pub(crate) fn bounds(&self) -> NSRect;

        #[method(convertRect:toView:)]
        pub(crate) fn convert_rect_to_view(&self, rect: NSRect, view: Option<&NSView>) -> NSRect;

        #[method(convertPoint:fromView:)]
        pub(crate) fn convert_point_from_view(
            &self,
            point: NSPoint,
            view: Option<&NSView>,
        ) -> NSPoint;

        #[method(isFlipped)]
        pub(crate) fn is_flipped(&self) -> bool;

        #[method(backingScaleFactor)]
        pub(crate) fn backing_scale_factor(&self) -> CGFloat;

        // NSView actually implements the full NSAccessibility protocol,
        // but since we don't have complete metadata for that, it's easier
        // to just expose the needed methods here.
        #[method(accessibilityFrame)]
        pub(crate) fn accessibility_frame(&self) -> NSRect;
        #[method(accessibilityParent)]
        pub(crate) fn accessibility_parent(&self) -> *mut NSObject;
    }
);
