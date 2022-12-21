// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

mod responder;
pub(crate) use responder::NSResponder;

mod view;
pub(crate) use view::NSView;

mod window;
pub(crate) use window::NSWindow;

mod accessibility_constants;
pub(crate) use accessibility_constants::*;

mod accessibility_element;
pub(crate) use accessibility_element::NSAccessibilityElement;

mod accessibility_functions;
pub(crate) use accessibility_functions::*;
