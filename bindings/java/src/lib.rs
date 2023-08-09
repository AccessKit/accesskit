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

use jni::{
    objects::{JByteArray, JFloatArray},
    sys::{jfloat, jlong},
    JNIEnv,
};

mod common;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub use common::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
pub use windows::*;

pub(crate) fn into_jptr<T>(source: T) -> jlong {
    Box::into_raw(Box::new(source)) as jlong
}

pub(crate) fn ref_from_jptr<'a, T>(ptr: jlong) -> &'a T {
    unsafe { &*(ptr as *const T) }
}

pub(crate) fn mut_from_jptr<'a, T>(ptr: jlong) -> &'a mut T {
    unsafe { &mut *(ptr as *mut T) }
}

pub(crate) fn box_from_jptr<T>(ptr: jlong) -> Box<T> {
    unsafe { Box::from_raw(ptr as *mut T) }
}

pub(crate) fn box_str_from_utf8_jbytes(env: &JNIEnv, bytes: JByteArray) -> Box<str> {
    let bytes = env.convert_byte_array(bytes).unwrap();
    unsafe { String::from_utf8_unchecked(bytes) }.into()
}

pub(crate) fn convert_float_array(
    env: &JNIEnv,
    value: JFloatArray,
) -> jni::errors::Result<Vec<jfloat>> {
    let len = env.get_array_length(&value)? as usize;
    let mut buf = vec![0.0; len];
    env.get_float_array_region(value, 0, &mut buf)?;
    Ok(buf)
}
