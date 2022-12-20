// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, ObjectRef};
use atspi::{accessible::Role, State};
use strum::AsRefStr;

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
    #[strum(serialize = "Deactivate")]
    Deactivated,
}
