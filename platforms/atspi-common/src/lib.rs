// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

mod action;
mod adapter;
mod callback;
mod context;
mod error;
mod events;
mod filters;
mod node;
mod rect;
pub mod simplified;
mod util;

pub use atspi_common::{CoordType, InterfaceSet, Layer, Role, State, StateSet};

pub use action::*;
pub use adapter::{Adapter, AdapterIdToken};
pub use callback::AdapterCallback;
pub use context::AppContext;
pub use error::*;
pub use events::*;
pub use node::{NodeIdOrRoot, PlatformNode, PlatformRoot};
pub use rect::*;
pub use util::WindowBounds;
