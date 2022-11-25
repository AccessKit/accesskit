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

        #[sel(bounds)]
        pub(crate) fn bounds(&self) -> NSRect;

        #[sel(convertRect:toView:)]
        pub(crate) fn convert_rect_to_view(&self, rect: NSRect, view: Option<&NSView>) -> NSRect;

        #[sel(convertPoint:fromView:)]
        pub(crate) fn convert_point_from_view(
            &self,
            point: NSPoint,
            view: Option<&NSView>,
        ) -> NSPoint;
    }
);