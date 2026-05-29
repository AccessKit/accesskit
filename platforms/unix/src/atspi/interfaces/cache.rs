// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::PlatformRoot;
use atspi::CacheItem;
use zbus::{fdo, interface, names::OwnedUniqueName};

pub(crate) struct CacheInterface {
    bus_name: OwnedUniqueName,
    root: PlatformRoot,
}

impl CacheInterface {
    pub fn new(bus_name: OwnedUniqueName, root: PlatformRoot) -> Self {
        Self { bus_name, root }
    }
}

#[interface(name = "org.a11y.atspi.Cache")]
impl CacheInterface {
    fn get_items(&self) -> fdo::Result<Vec<CacheItem>> {
        todo!()
    }
}
