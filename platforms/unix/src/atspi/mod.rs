// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

mod bus;
pub(crate) mod interfaces;
mod object_address;
mod object_id;
mod object_ref;

#[derive(Debug, Default, Serialize, Deserialize, Type)]
pub(crate) struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl From<accesskit::kurbo::Rect> for Rect {
    fn from(value: accesskit::kurbo::Rect) -> Rect {
        Rect {
            x: value.x0 as i32,
            y: value.y0 as i32,
            width: value.width() as i32,
            height: value.height() as i32,
        }
    }
}

pub(crate) use bus::Bus;
pub(crate) use object_address::*;
pub(crate) use object_id::*;
pub(crate) use object_ref::*;
