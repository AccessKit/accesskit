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

#[derive(Clone, Debug, Serialize, Deserialize, Type, Value)]
pub struct ObjectAddress<'a> {
    #[serde(borrow)]
    bus_name: UniqueName<'a>,
    #[serde(borrow)]
    path: ObjectPath<'a>,
}

impl<'a> ObjectAddress<'a> {
    pub fn new(bus_name: UniqueName<'a>, path: ObjectPath<'a>) -> ObjectAddress<'a> {
        Self { bus_name, path }
    }

    pub fn accessible(bus_name: UniqueName<'a>, id: &ObjectId) -> ObjectAddress<'a> {
        Self {
            bus_name,
            path: ObjectPath::from_string_unchecked(format!(
                "{}{}",
                ACCESSIBLE_PATH_PREFIX,
                id.as_str()
            )),
        }
    }

    pub fn root(bus_name: UniqueName<'a>) -> ObjectAddress<'a> {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(ROOT_PATH),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, OwnedValue, Type, Value)]
pub struct OwnedObjectAddress {
    bus_name: OwnedUniqueName,
    path: OwnedObjectPath,
}

impl OwnedObjectAddress {
    pub fn new(bus_name: OwnedUniqueName, path: OwnedObjectPath) -> Self {
        Self { bus_name, path }
    }

    pub fn accessible(bus_name: OwnedUniqueName, id: ObjectId) -> Self {
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

    pub fn null(bus_name: OwnedUniqueName) -> Self {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(NULL_PATH).into(),
        }
    }
}

impl From<ObjectAddress<'_>> for OwnedObjectAddress {
    fn from(value: ObjectAddress) -> OwnedObjectAddress {
        OwnedObjectAddress {
            bus_name: value.bus_name.into(),
            path: value.path.into(),
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
