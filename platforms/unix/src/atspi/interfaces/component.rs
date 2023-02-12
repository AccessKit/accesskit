// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{OwnedObjectAddress, Rect},
    PlatformNode,
};
use atspi::{component::Layer, CoordType};
use zbus::{fdo, MessageHeader};

pub(crate) struct ComponentInterface {
    node: PlatformNode,
}

impl ComponentInterface {
    pub(crate) fn new(node: PlatformNode) -> Self {
        Self { node }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Component")]
impl ComponentInterface {
    fn contains(&self, x: i32, y: i32, coord_type: CoordType) -> fdo::Result<bool> {
        self.node.contains(x, y, coord_type)
    }

    fn get_accessible_at_point(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        x: i32,
        y: i32,
        coord_type: CoordType,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        let accessible = self.node.get_accessible_at_point(x, y, coord_type)?;
        super::object_address(hdr.destination()?, accessible)
    }

    fn get_extents(&self, coord_type: CoordType) -> fdo::Result<(Rect,)> {
        self.node.get_extents(coord_type)
    }

    fn get_layer(&self) -> fdo::Result<Layer> {
        self.node.get_layer()
    }

    fn grab_focus(&self) -> fdo::Result<bool> {
        self.node.grab_focus()
    }

    fn scroll_to_point(&self, coord_type: CoordType, x: i32, y: i32) -> fdo::Result<bool> {
        self.node.scroll_to_point(coord_type, x, y)
    }
}
