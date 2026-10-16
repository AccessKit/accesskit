// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::{PlatformNode, Rect};
use atspi::CoordType;
use zbus::{fdo, interface};

pub(crate) struct ImageInterface(PlatformNode);

impl ImageInterface {
    pub fn new(node: PlatformNode) -> Self {
        Self(node)
    }

    fn map_error(&self) -> impl '_ + FnOnce(accesskit_atspi_common::Error) -> fdo::Error {
        |error| crate::util::map_error_from_node(&self.0, error)
    }
}

#[interface(name = "org.a11y.atspi.Image")]
impl ImageInterface {
    #[zbus(property)]
    fn image_description(&self) -> fdo::Result<String> {
        self.0.image_description().map_err(self.map_error())
    }

    #[zbus(property)]
    fn image_locale(&self) -> fdo::Result<String> {
        Err(fdo::Error::NotSupported(
            "image locale is not supported".into(),
        ))
    }

    fn get_image_extents(&self, coord_type: CoordType) -> fdo::Result<(Rect,)> {
        self.0
            .image_extents(coord_type)
            .map(|rect| (rect,))
            .map_err(self.map_error())
    }

    fn get_image_position(&self, coord_type: CoordType) -> fdo::Result<(i32, i32)> {
        self.0.image_position(coord_type).map_err(self.map_error())
    }

    fn get_image_size(&self) -> fdo::Result<(i32, i32)> {
        self.0.image_size().map_err(self.map_error())
    }
}
