// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use atspi::Accessible;
use serde::{Deserialize, Serialize};
use zbus::{
    names::UniqueName,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Type, Value},
};

const NULL_PATH: &str = "/org/a11y/atspi/null";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, OwnedValue, Type, Value)]
pub(crate) struct OwnedObjectAddress {
    bus_name: String,
    path: OwnedObjectPath,
}

impl OwnedObjectAddress {
    pub(crate) fn new(bus_name: &UniqueName, path: OwnedObjectPath) -> Self {
        Self {
            bus_name: bus_name.to_string(),
            path,
        }
    }

    pub(crate) fn null() -> Self {
        Self {
            bus_name: String::new(),
            path: ObjectPath::from_str_unchecked(NULL_PATH).into(),
        }
    }
}

impl From<Accessible> for OwnedObjectAddress {
    fn from(object: Accessible) -> Self {
        Self {
            bus_name: object.name,
            path: object.path,
        }
    }
}
