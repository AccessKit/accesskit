// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#[macro_use]
extern crate zbus;

mod adapter;
mod atspi;
mod context;
mod node;
mod util;

pub use adapter::Adapter;
pub(crate) use node::{unknown_object, PlatformNode, PlatformRootNode};
