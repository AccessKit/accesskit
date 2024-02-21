// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use atspi_common::{Live, Role, State};

use crate::{NodeIdOrRoot, Rect};

#[derive(Debug)]
pub enum Event {
    Object {
        target: NodeIdOrRoot,
        event: ObjectEvent,
    },
    Window {
        target: NodeId,
        name: String,
        event: WindowEvent,
    },
}

#[derive(Debug)]
pub enum Property {
    Name(String),
    Description(String),
    Parent(NodeIdOrRoot),
    Role(Role),
    Value(f64),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum ObjectEvent {
    ActiveDescendantChanged(NodeId),
    Announcement(String, Live),
    BoundsChanged(Rect),
    ChildAdded(usize, NodeId),
    ChildRemoved(NodeId),
    PropertyChanged(Property),
    StateChanged(State, bool),
}

#[derive(Debug)]
pub enum WindowEvent {
    Activated,
    Deactivated,
}
