// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::Tree;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use crate::util::{AppContext, WindowBounds};

pub(crate) struct Context {
    pub(crate) tree: RwLock<Tree>,
    pub(crate) action_handler: Mutex<Box<dyn ActionHandler + Send>>,
    pub(crate) app_context: RwLock<AppContext>,
    pub(crate) root_window_bounds: RwLock<WindowBounds>,
}

impl Context {
    pub(crate) fn new(
        tree: Tree,
        action_handler: Box<dyn ActionHandler + Send>,
        app_context: AppContext,
    ) -> Arc<Self> {
        Arc::new(Self {
            tree: RwLock::new(tree),
            action_handler: Mutex::new(action_handler),
            app_context: RwLock::new(app_context),
            root_window_bounds: RwLock::new(Default::default()),
        })
    }

    pub(crate) fn read_tree(&self) -> RwLockReadGuard<'_, Tree> {
        self.tree.read().unwrap()
    }

    pub(crate) fn read_app_context(&self) -> RwLockReadGuard<'_, AppContext> {
        self.app_context.read().unwrap()
    }

    pub(crate) fn read_root_window_bounds(&self) -> RwLockReadGuard<'_, WindowBounds> {
        self.root_window_bounds.read().unwrap()
    }

    pub(crate) fn do_action(&self, request: ActionRequest) {
        self.action_handler.lock().unwrap().do_action(request);
    }
}
