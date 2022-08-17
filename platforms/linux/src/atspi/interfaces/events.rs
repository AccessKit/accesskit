// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use zvariant::{OwnedValue, Type, Value};

pub struct QueuedEvent {
    pub target: ObjectId<'static>,
    pub kind: EventKind,
}

pub enum EventKind {
    Focus,
    Object(ObjectEvent),
    Window {
        window_name: String,
        event: WindowEvent,
    },
}

#[derive(AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Property {
    AccessibleName,
    AccessibleDescription,
    AccessibleParent,
    AccessibleRole,
}

#[derive(AsRefStr)]
pub enum ObjectEvent {
    StateChanged(State, bool),
    #[strum(serialize = "PropertyChange")]
    PropertyChanged(Property, OwnedValue),
}

#[derive(AsRefStr)]
pub enum WindowEvent {
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
pub struct EventData<'a> {
    pub minor: &'a str,
    pub detail1: i32,
    pub detail2: i32,
    pub any_data: OwnedValue,
    pub properties: HashMap<String, OwnedValue>,
}
