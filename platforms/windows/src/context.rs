// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, Point};
use accesskit_consumer::Tree;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use windows::Win32::Foundation::*;

use crate::util::*;

pub(crate) struct Context {
    pub(crate) hwnd: HWND,
    pub(crate) tree: RwLock<Tree>,
    pub(crate) action_handler: Box<dyn ActionHandler + Send + Sync>,
}

impl Context {
    pub(crate) fn new(
        hwnd: HWND,
        tree: Tree,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(Self {
            hwnd,
            tree: RwLock::new(tree),
            action_handler,
        })
    }

    pub(crate) fn read_tree(&self) -> RwLockReadGuard<'_, Tree> {
        self.tree.read().unwrap()
    }

    pub(crate) fn client_top_left(&self) -> Point {
        client_top_left(self.hwnd)
    }
}
