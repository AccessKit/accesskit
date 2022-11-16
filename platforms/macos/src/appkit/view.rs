// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{
    foundation::{NSObject, NSRect},
    rc::{Id, Shared},
    ClassType, extern_class, extern_methods, msg_send_id,
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

        #[sel(convertRect:toView:)]
        pub(crate) fn convert_rect_to_view(
            &self,
            rect: NSRect,
            view: Option<&NSView>,
        ) -> NSRect;
    }
);
