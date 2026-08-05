// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

//! iOS adapter for AccessKit.
//!
//! This crate provides two adapters for exposing an AccessKit accessibility
//! tree to iOS via UIKit's accessibility API:
//!
//! - [`Adapter`] is the low-level adapter. It gives you full control over
//!   when the accessibility tree is initialized and updated, but requires you
//!   to manually forward UIKit accessibility methods
//!   (`isAccessibilityElement`, `accessibilityElements`,
//!   `accessibilityHitTest:`) from your `UIView` subclass to the adapter.
//!   Use this when you own the `UIView` subclass and can override these
//!   methods directly.
//!
//! - [`SubclassingAdapter`] wraps [`Adapter`] and uses dynamic Objective-C
//!   subclassing to automatically override the accessibility methods on an
//!   existing `UIView`. Use this when you cannot subclass the view yourself,
//!   for example when integrating with a framework that creates views on your
//!   behalf.

#![deny(unsafe_op_in_unsafe_fn)]

mod context;
mod filters;
mod node;
mod util;

mod adapter;
pub use adapter::Adapter;

mod event;
pub use event::QueuedEvents;

mod subclass;
pub use subclass::SubclassingAdapter;

pub use objc2_foundation::{CGPoint, NSArray, NSInteger, NSObject};
