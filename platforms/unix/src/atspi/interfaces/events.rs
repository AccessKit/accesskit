// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, ObjectRef};
use atspi::{accessible::Role, State};

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

pub(crate) enum Property {
    Name(String),
    Description(String),
    Parent(Option<ObjectRef>),
    Role(Role),
    Value(f64),
}

pub(crate) enum ObjectEvent {
    StateChanged(State, bool),
    PropertyChanged(Property),
}

pub(crate) enum WindowEvent {
    Activated,
    Deactivated,
}
