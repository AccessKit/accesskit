// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use atspi::Accessible;
use serde::{Deserialize, Serialize};
use zbus::{
    names::{OwnedUniqueName, UniqueName},
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Type, Value},
};

const NULL_PATH: &str = "/org/a11y/atspi/null";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, OwnedValue, Type, Value)]
pub(crate) struct OwnedObjectAddress {
    bus_name: OwnedUniqueName,
    path: OwnedObjectPath,
}

impl OwnedObjectAddress {
    pub(crate) fn new(bus_name: OwnedUniqueName, path: OwnedObjectPath) -> Self {
        Self { bus_name, path }
    }

    pub(crate) fn null(bus_name: OwnedUniqueName) -> Self {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(NULL_PATH).into(),
        }
    }
}

impl From<Accessible> for OwnedObjectAddress {
    fn from(object: Accessible) -> Self {
        Self {
            bus_name: OwnedUniqueName::from(UniqueName::from_string_unchecked(object.name)),
            path: object.path,
        }
    }
}
