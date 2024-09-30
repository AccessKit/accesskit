// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use atspi::ObjectRef;
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
    pub(crate) fn new(bus_name: &UniqueName, path: OwnedObjectPath) -> Self {
        Self {
            bus_name: bus_name.to_owned().into(),
            path,
        }
    }

    pub(crate) fn null() -> Self {
        Self {
            bus_name: UniqueName::from_static_str("").unwrap().into(),
            path: ObjectPath::from_str_unchecked(NULL_PATH).into(),
        }
    }
}

impl From<ObjectRef> for OwnedObjectAddress {
    fn from(object: ObjectRef) -> Self {
        Self {
            bus_name: object.name,
            path: object.path,
        }
    }
}
