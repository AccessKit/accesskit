// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from rustls-ffi.
// Copyright (c) 2021, Jacob Hoffman-Andrews <jsha@letsencrypt.org>
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file), the ISC license (found in
// the LICENSE-ISC file), or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::ptr::{null, null_mut};

// We wrap all function calls in an ffi_panic_boundary! macro, which catches
// panics and early-returns from the function. For functions that return
// rustls_result, we return a dedicated error code: `Panic`. For functions
// that don't return rustls_result, we return a default value: false, 0, or
// null. This trait provides that logic.
pub(crate) trait PanicOrDefault {
    fn value() -> Self;
}

// This trait is like PanicOrDefault, but returns rustls_result::NullParameter
// rather than `Panic`.
pub(crate) trait NullParameterOrDefault {
    fn value() -> Self;
}

// Defaultable is a subset of Default that can be returned by rustls-ffi.
// We use this rather than Default directly so that we can do a blanket
// impl for `T: Defaultable`. The compiler disallows a blanket impl for
// `T: Default` because `std::default` could later implement `Default`
// for `*mut T` and `*const T`.
pub(crate) trait Defaultable: Default {}

impl Defaultable for u16 {}
impl Defaultable for usize {}
impl Defaultable for bool {}
impl Defaultable for () {}
impl Defaultable for accesskit::Role {}
impl<T> Defaultable for Option<T> {}

impl<T: Defaultable> PanicOrDefault for T {
    fn value() -> Self {
        Default::default()
    }
}

impl<T> PanicOrDefault for *mut T {
    fn value() -> Self {
        null_mut()
    }
}

impl<T> PanicOrDefault for *const T {
    fn value() -> Self {
        null()
    }
}

impl<T: Defaultable> NullParameterOrDefault for T {
    fn value() -> Self {
        Default::default()
    }
}

impl<T> NullParameterOrDefault for *mut T {
    fn value() -> Self {
        null_mut()
    }
}

impl<T> NullParameterOrDefault for *const T {
    fn value() -> Self {
        null()
    }
}
