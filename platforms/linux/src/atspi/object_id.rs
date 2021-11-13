// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use zvariant::{
    derive::{Type, Value},
    Str
};

#[derive(Clone, Debug, Deserialize, Serialize, Type, Value)]
pub struct ObjectId<'a>(#[serde(borrow)] Str<'a>);

impl<'a> ObjectId<'a> {
    pub unsafe fn from_str_unchecked(id: &'a str) -> ObjectId<'a> {
        Self(Str::from(id))
    }

    pub fn root() -> ObjectId<'static> {
        ObjectId(Str::from("root"))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_owned(&self) -> ObjectId<'static> {
        ObjectId(self.0.to_owned())
    }
}

impl From<NonZeroU64> for ObjectId<'static> {
    fn from(value: NonZeroU64) -> Self {
        Self(Str::from(value.to_string()))
    }
}
