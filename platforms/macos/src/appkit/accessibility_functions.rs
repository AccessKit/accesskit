// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::foundation::{NSDictionary, NSObject, NSString};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    pub(crate) fn NSAccessibilityPostNotification(element: &NSObject, notification: &NSString);
    pub(crate) fn NSAccessibilityPostNotificationWithUserInfo(
        element: &NSObject,
        notification: &NSString,
        user_info: &NSDictionary<NSString, NSObject>,
    );
}
