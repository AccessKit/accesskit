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

#![allow(non_camel_case_types)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod common;
mod geometry;

#[cfg(any(target_os = "macos", feature = "cbindgen"))]
mod macos;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    feature = "cbindgen"
))]
mod unix;
#[cfg(any(target_os = "windows", feature = "cbindgen"))]
mod windows;

pub use common::*;
pub use geometry::*;
#[cfg(any(target_os = "macos", feature = "cbindgen"))]
pub use macos::*;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    feature = "cbindgen"
))]
pub use unix::*;
#[cfg(any(target_os = "windows", feature = "cbindgen"))]
pub use windows::*;

/// `CastPtr` represents the relationship between a snake case type (like `node_class_set`)
/// and the corresponding Rust type (like `NodeClassSet`). For each matched pair of types, there
/// should be an `impl CastPtr for foo_bar { RustType = FooBar }`.
///
/// This allows us to avoid using `as` in most places, and ensure that when we cast, we're
/// preserving const-ness, and casting between the correct types.
/// Implementing this is required in order to use `ref_from_ptr!` or
/// `mut_from_ptr!`.
pub(crate) trait CastPtr {
    type RustType;

    fn cast_mut_ptr(ptr: *mut Self) -> *mut Self::RustType {
        ptr as *mut _
    }
}

/// `CastConstPtr` represents a subset of `CastPtr`, for when we can only treat
/// something as a const (for instance when dealing with `Arc`).
pub(crate) trait CastConstPtr {
    type RustType;

    fn cast_const_ptr(ptr: *const Self) -> *const Self::RustType {
        ptr as *const _
    }
}

/// Anything that qualifies for `CastPtr` also automatically qualifies for
/// `CastConstPtr`. Splitting out `CastPtr` vs `CastConstPtr` allows us to ensure
/// that `Arc`s are never cast to a mutable pointer.
impl<T, R> CastConstPtr for T
where
    T: CastPtr<RustType = R>,
{
    type RustType = R;
}

// An implementation of BoxCastPtr means that when we give C code a pointer to the relevant type,
// it is actually a Box.
pub(crate) trait BoxCastPtr: CastPtr + Sized {
    fn to_box(ptr: *mut Self) -> Box<Self::RustType> {
        assert!(!ptr.is_null());
        let rs_typed = Self::cast_mut_ptr(ptr);
        unsafe { Box::from_raw(rs_typed) }
    }

    fn to_mut_ptr(src: Self::RustType) -> *mut Self {
        Box::into_raw(Box::new(src)) as *mut _
    }

    fn to_nullable_mut_ptr(src: Option<Self::RustType>) -> *mut Self {
        src.map_or_else(std::ptr::null_mut, Self::to_mut_ptr)
    }

    fn set_mut_ptr(dst: *mut *mut Self, src: Self::RustType) {
        unsafe {
            *dst = Self::to_mut_ptr(src);
        }
    }
}

/// Turn a raw const pointer into a reference. This is a generic function
/// rather than part of the `CastPtr` trait because (a) const pointers can't act
/// as "self" for trait methods, and (b) we want to rely on type inference
/// against `T` (the cast-to type) rather than across `F` (the from type).
pub(crate) fn ref_from_ptr<'a, F, T>(from: *const F) -> &'a T
where
    F: CastConstPtr<RustType = T>,
{
    unsafe { F::cast_const_ptr(from).as_ref() }.unwrap()
}

/// Turn a raw mut pointer into a mutable reference.
pub(crate) fn mut_from_ptr<'a, F, T>(from: *mut F) -> &'a mut T
where
    F: CastPtr<RustType = T>,
{
    unsafe { F::cast_mut_ptr(from).as_mut() }.unwrap()
}

pub(crate) fn box_from_ptr<F, T>(from: *mut F) -> Box<T>
where
    F: BoxCastPtr<RustType = T>,
{
    F::to_box(from)
}

#[doc(hidden)]
#[macro_export]
macro_rules! opt_struct {
    ($struct_name:ident, $prop_type:ty) => {
        /// Represents an optional value.
        ///
        /// If `has_value` is false, do not read the `value` field.
        #[repr(C)]
        pub struct $struct_name {
            pub has_value: bool,
            pub value: std::mem::MaybeUninit<$prop_type>,
        }
        impl<T> From<Option<T>> for $struct_name
        where
            T: Into<$prop_type>,
        {
            fn from(value: Option<T>) -> $struct_name {
                match value {
                    None => $struct_name::default(),
                    Some(value) => $struct_name {
                        has_value: true,
                        value: std::mem::MaybeUninit::new(value.into()),
                    },
                }
            }
        }
        impl<T> From<$struct_name> for Option<T>
        where
            T: From<$prop_type>,
        {
            fn from(value: $struct_name) -> Self {
                match value.has_value {
                    true => Some(unsafe { T::from(value.value.assume_init()) }),
                    false => None,
                }
            }
        }
        impl Default for $struct_name {
            fn default() -> $struct_name {
                $struct_name {
                    has_value: false,
                    value: std::mem::MaybeUninit::uninit(),
                }
            }
        }
    };
}
