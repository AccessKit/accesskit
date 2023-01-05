// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::ObjectId;
use serde::{Deserialize, Serialize};
use zbus::{
    names::{OwnedUniqueName, UniqueName},
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Type, Value},
};

pub(crate) const ACCESSIBLE_PATH_PREFIX: &str = "/org/a11y/atspi/accessible/";
pub(crate) const NULL_PATH: &str = "/org/a11y/atspi/null";
pub(crate) const ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, OwnedValue, Type, Value)]
pub(crate) struct OwnedObjectAddress {
    bus_name: OwnedUniqueName,
    path: OwnedObjectPath,
}

impl OwnedObjectAddress {
    pub(crate) fn accessible(bus_name: OwnedUniqueName, id: ObjectId) -> Self {
        Self {
            bus_name,
            path: ObjectPath::from_string_unchecked(format!(
                "{}{}",
                ACCESSIBLE_PATH_PREFIX,
                id.as_str()
            ))
            .into(),
        }
    }

    pub(crate) fn null(bus_name: OwnedUniqueName) -> Self {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(NULL_PATH).into(),
        }
    }

    pub(crate) fn root(bus_name: OwnedUniqueName) -> Self {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(ROOT_PATH).into(),
        }
    }
}

impl From<(String, OwnedObjectPath)> for OwnedObjectAddress {
    fn from(value: (String, OwnedObjectPath)) -> Self {
        Self {
            bus_name: OwnedUniqueName::from(UniqueName::from_string_unchecked(value.0)),
            path: value.1,
        }
    }
}
