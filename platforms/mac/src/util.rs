// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use cocoa::base::{id, nil, BOOL, YES};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::{msg_send, sel, sel_impl};

// From druid-shell
pub(crate) fn from_nsstring(s: id) -> String {
    unsafe {
        let slice = std::slice::from_raw_parts(s.UTF8String() as *const _, s.len());
        let result = std::str::from_utf8_unchecked(slice);
        result.into()
    }
}

/// Create a new NSString from a &str.
// Also from druid-shell
pub(crate) fn make_nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s).autorelease() }
}

pub(crate) fn nsstrings_equal(a: id, b: id) -> bool {
    let equal: BOOL = unsafe { msg_send![a, isEqualToString: b] };
    equal == YES
}
