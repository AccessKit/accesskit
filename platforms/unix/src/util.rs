// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Point, Rect};
use atspi::CoordType;
#[cfg(feature = "tokio")]
use once_cell::sync::Lazy;

#[cfg(not(feature = "tokio"))]
pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
    zbus::block_on(future)
}

#[cfg(feature = "tokio")]
pub(crate) static TOKIO_RT: Lazy<tokio::runtime::Handle> = Lazy::new(|| {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");
    let handle = rt.handle().clone();
    std::thread::Builder::new()
        .name("accesskit-tokio".into())
        .spawn(move || {
            rt.block_on(async {
                let duration = std::time::Duration::from_secs(86400);
                loop {
                    tokio::time::sleep(duration).await;
                }
            });
        })
        .expect("launch tokio runtime thread");
    handle
});

#[cfg(feature = "tokio")]
pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
    TOKIO_RT.block_on(future)
}

#[derive(Clone, Copy, Default)]
pub(crate) struct WindowBounds {
    pub(crate) outer: Rect,
    pub(crate) inner: Rect,
}

impl WindowBounds {
    pub(crate) fn new(outer: Rect, inner: Rect) -> Self {
        Self { outer, inner }
    }

    pub(crate) fn top_left(&self, coord_type: CoordType, is_root: bool) -> Point {
        match coord_type {
            CoordType::Screen if is_root => self.outer.origin(),
            CoordType::Screen => self.inner.origin(),
            CoordType::Window if is_root => Point::ZERO,
            CoordType::Window => {
                let outer_position = self.outer.origin();
                let inner_position = self.inner.origin();
                Point::new(
                    inner_position.x - outer_position.x,
                    inner_position.y - outer_position.y,
                )
            }
            _ => unimplemented!(),
        }
    }
}
