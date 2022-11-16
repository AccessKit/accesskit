// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{
    foundation::{NSObject, NSRect},
    ClassType, extern_class, extern_methods,
};

use super::NSResponder;

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
    }
);
