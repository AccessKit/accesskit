// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use windows::Win32::UI::Accessibility::*;

/// Ensures that UI Automation has already been initialized before an instance
/// of [`crate::Adapter`] is created.
///
/// Forcing UIA initialization before the first `WM_GETOBJECT` message
/// prevents a race condition that leads to nested `WM_GETOBJECT` messages and,
/// in some cases, assistive technologies not realizing that the window natively
/// implements UIA. See [AccessKit issue #37](https://github.com/AccessKit/accesskit/issues/37)
/// for more details.
#[derive(Clone, Copy)]
pub struct UiaInitMarker;

impl UiaInitMarker {
    /// Force UIA to be initialized. This must be called outside of a handler
    /// for the `WM_GETOBJECT` message, before the first AccessKit-enabled
    /// window is shown. Calling this function multiple times is cheap,
    /// and because this function triggers global UIA initialization, there is
    /// no matching cleanup step.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // `UiaLookupId` is a cheap way of forcing UIA to initialize itself.
        unsafe {
            UiaLookupId(
                AutomationIdentifierType_Property,
                &ControlType_Property_GUID,
            )
        };
        Self
    }
}
