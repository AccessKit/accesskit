// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Point, Rect};
use atspi::CoordType;

#[cfg(not(feature = "tokio"))]
pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
    futures_lite::future::block_on(future)
}

#[cfg(feature = "tokio")]
pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("launch of single-threaded tokio runtime");
    runtime.block_on(future)
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
