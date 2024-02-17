// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use atspi_common::{Live, Role, State};

use crate::{PlatformNode, PlatformNodeOrRoot, Rect};

pub enum Event {
    Object {
        target: PlatformNodeOrRoot,
        event: ObjectEvent,
    },
    Window {
        target: PlatformNode,
        name: String,
        event: WindowEvent,
    },
}

pub enum Property {
    Name(String),
    Description(String),
    Parent(PlatformNodeOrRoot),
    Role(Role),
    Value(f64),
}

#[allow(clippy::enum_variant_names)]
pub enum ObjectEvent {
    ActiveDescendantChanged(PlatformNode),
    Announcement(String, Live),
    BoundsChanged(Rect),
    ChildAdded(usize, PlatformNode),
    ChildRemoved(PlatformNode),
    PropertyChanged(Property),
    StateChanged(State, bool),
}

pub enum WindowEvent {
    Activated,
    Deactivated,
}
