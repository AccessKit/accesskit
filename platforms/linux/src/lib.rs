// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![feature(unix_socket_abstract)]

#[macro_use] extern crate zbus;

mod adapter;
mod atspi;
mod node;

pub use adapter::Adapter;
