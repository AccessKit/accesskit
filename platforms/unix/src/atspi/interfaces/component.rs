// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    atspi::{OwnedObjectAddress, Rect},
    unknown_object,
    util::WindowBounds,
    PlatformNode,
};
use atspi::{component::Layer, CoordType};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use zbus::{fdo, MessageHeader};

pub(crate) struct ComponentInterface {
    node: PlatformNode,
    root_window_bounds: Weak<RwLock<WindowBounds>>,
}

impl ComponentInterface {
    pub(crate) fn new(node: PlatformNode, root_window_bounds: &Arc<RwLock<WindowBounds>>) -> Self {
        Self {
            node,
            root_window_bounds: Arc::downgrade(root_window_bounds),
        }
    }

    fn upgrade_bounds(&self) -> fdo::Result<Arc<RwLock<WindowBounds>>> {
        if let Some(bounds) = self.root_window_bounds.upgrade() {
            Ok(bounds)
        } else {
            Err(unknown_object(&self.node.accessible_id()))
        }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Component")]
impl ComponentInterface {
    fn contains(&self, x: i32, y: i32, coord_type: CoordType) -> fdo::Result<bool> {
        let window_bounds = self.upgrade_bounds()?;
        let contains = self.node.contains(&window_bounds.read(), x, y, coord_type);
        contains
    }

    fn get_accessible_at_point(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        x: i32,
        y: i32,
        coord_type: CoordType,
    ) -> fdo::Result<(OwnedObjectAddress,)> {
        let window_bounds = self.upgrade_bounds()?;
        let accessible =
            self.node
                .get_accessible_at_point(&window_bounds.read(), x, y, coord_type)?;
        super::object_address(hdr.destination()?, accessible)
    }

    fn get_extents(&self, coord_type: CoordType) -> fdo::Result<(Rect,)> {
        let window_bounds = self.upgrade_bounds()?;
        let extents = self.node.get_extents(&window_bounds.read(), coord_type);
        extents
    }

    fn get_layer(&self) -> fdo::Result<Layer> {
        self.node.get_layer()
    }

    fn grab_focus(&self) -> fdo::Result<bool> {
        self.node.grab_focus()
    }

    fn scroll_to_point(&self, coord_type: CoordType, x: i32, y: i32) -> fdo::Result<bool> {
        let window_bounds = self.upgrade_bounds()?;
        let scrolled = self
            .node
            .scroll_to_point(&window_bounds.read(), coord_type, x, y);
        scrolled
    }
}
