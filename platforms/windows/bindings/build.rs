// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

fn main() {
    windows::build!(
        Windows::Win32::{
            Foundation::*,
            Graphics::Gdi::ValidateRect,
            System::{
                LibraryLoader::GetModuleHandleA,
                OleAutomation::*,
            },
            UI::{
                Accessibility::*,
                WindowsAndMessaging::*,
            },
        },
    );
}
