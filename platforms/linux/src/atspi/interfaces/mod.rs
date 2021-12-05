// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use enumflags2::{BitFlags, bitflags};
mod accessible;
mod application;
mod events;

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Interface {
    Accessible,
    Application,
    FocusEvents,
    ObjectEvents,
    WindowEvents,
}

pub type Interfaces = BitFlags<Interface, u8>;

pub use accessible::*;
pub use application::*;
pub use events::*;
