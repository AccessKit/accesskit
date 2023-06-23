// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{
    extern_class, extern_methods,
    foundation::{NSObject, NSPoint, NSRect},
    msg_send_id,
    rc::{Id, Shared},
    ClassType,
};

use super::{NSResponder, NSView};

extern_class!(
    #[derive(Debug)]
    pub(crate) struct NSWindow;

    unsafe impl ClassType for NSWindow {
        #[inherits(NSObject)]
        type Super = NSResponder;
    }
);

extern_methods!(
    unsafe impl NSWindow {
        #[sel(convertRectToScreen:)]
        pub(crate) fn convert_rect_to_screen(&self, rect: NSRect) -> NSRect;

        #[sel(convertPointFromScreen:)]
        pub(crate) fn convert_point_from_screen(&self, point: NSPoint) -> NSPoint;

        pub(crate) fn content_view(&self) -> Option<Id<NSView, Shared>> {
            unsafe { msg_send_id![self, contentView] }
        }
    }
);
