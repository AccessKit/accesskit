// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

// Based loosely on winit's src/platform_impl/mod.rs.

pub use self::platform::*;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;

#[cfg(all(not(target_os = "windows"), not(target_os = "macos"), not(target_os = "linux")))]
#[cfg(all(not(target_os = "windows"), not(target_os = "linux"),))]
#[path = "null.rs"]
mod platform;
