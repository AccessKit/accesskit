// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Str, Type, Value};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Type, Value)]
pub(crate) struct ObjectId<'a>(#[serde(borrow)] Str<'a>);

impl<'a> ObjectId<'a> {
    pub(crate) fn root() -> ObjectId<'static> {
        ObjectId(Str::from("root"))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<NodeId> for ObjectId<'static> {
    fn from(value: NodeId) -> Self {
        Self(Str::from(value.0.to_string()))
    }
}
