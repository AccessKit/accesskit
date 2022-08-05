// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::ObjectId;
use serde::{Deserialize, Serialize};
use zbus::names::{OwnedUniqueName, UniqueName};
use zvariant::{ObjectPath, OwnedObjectPath, Type, Value};

pub const ACCESSIBLE_PATH_PREFIX: &'static str = "/org/a11y/atspi/accessible/";
pub const NULL_PATH: &'static str = "/org/a11y/atspi/null";
pub const ROOT_PATH: &'static str = "/org/a11y/atspi/accessible/root";

#[derive(Clone, Debug, Deserialize, Serialize, Type, Value)]
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

    pub fn accessible(bus_name: UniqueName<'a>, id: ObjectId) -> ObjectAddress<'a> {
        Self {
            bus_name,
            path: ObjectPath::from_string_unchecked(format!(
                "{}{}",
                ACCESSIBLE_PATH_PREFIX,
                id.as_str()
            )),
        }
    }

    pub fn null(bus_name: UniqueName<'a>) -> ObjectAddress<'a> {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(NULL_PATH),
        }
    }

    pub fn root(bus_name: UniqueName<'a>) -> ObjectAddress<'a> {
        Self {
            bus_name,
            path: ObjectPath::from_str_unchecked(ROOT_PATH),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, Value)]
pub struct OwnedObjectAddress {
    bus_name: OwnedUniqueName,
    path: OwnedObjectPath,
}

impl From<ObjectAddress<'_>> for OwnedObjectAddress {
    fn from(value: ObjectAddress) -> OwnedObjectAddress {
        OwnedObjectAddress {
            bus_name: value.bus_name.into(),
            path: value.path.into(),
        }
    }
}
