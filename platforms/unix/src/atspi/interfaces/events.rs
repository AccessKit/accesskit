// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, Rect};
use atspi::{Live, Role, State};

pub(crate) enum Event {
    Object {
        target: ObjectId,
        event: ObjectEvent,
    },
    Window {
        target: ObjectId,
        name: String,
        event: WindowEvent,
    },
}

pub(crate) enum Property {
    Name(String),
    Description(String),
    Parent(ObjectId),
    Role(Role),
    Value(f64),
}

#[allow(clippy::enum_variant_names)]
pub(crate) enum ObjectEvent {
    ActiveDescendantChanged(ObjectId),
    Announcement(String, Live),
    BoundsChanged(Rect),
    ChildAdded(usize, ObjectId),
    ChildRemoved(ObjectId),
    PropertyChanged(Property),
    StateChanged(State, bool),
}

pub(crate) enum WindowEvent {
    Activated,
    Deactivated,
}
