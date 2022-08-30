// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, ObjectRef, Role, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use zvariant::{OwnedValue, Type};

pub(crate) enum QueuedEvent {
    Object {
        target: ObjectId<'static>,
        event: ObjectEvent,
    },
    Window {
        target: ObjectId<'static>,
        name: String,
        event: WindowEvent,
    },
}

#[derive(AsRefStr)]
pub(crate) enum Property {
    #[strum(serialize = "accessible-name")]
    Name(String),
    #[strum(serialize = "accessible-description")]
    Description(String),
    #[strum(serialize = "accessible-parent")]
    Parent(Option<ObjectRef>),
    #[strum(serialize = "accessible-role")]
    Role(Role),
}

#[derive(AsRefStr)]
pub(crate) enum ObjectEvent {
    StateChanged(State, bool),
    #[strum(serialize = "PropertyChange")]
    PropertyChanged(Property),
}

#[derive(AsRefStr)]
pub(crate) enum WindowEvent {
    #[strum(serialize = "Activate")]
    Activated,
    #[strum(serialize = "Close")]
    Closed,
    #[strum(serialize = "Create")]
    Created,
    #[strum(serialize = "Deactivate")]
    Deactivated,
    #[strum(serialize = "Destroy")]
    Destroyed,
}

#[derive(Deserialize, Serialize, Type)]
pub(crate) struct EventData<'a> {
    pub minor: &'a str,
    pub detail1: i32,
    pub detail2: i32,
    pub any_data: OwnedValue,
    pub properties: HashMap<String, OwnedValue>,
}
