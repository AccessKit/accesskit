// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::Tree;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, Weak};

use crate::{atspi::OwnedObjectAddress, util::WindowBounds};

pub(crate) struct Context {
    pub(crate) tree: RwLock<Tree>,
    pub(crate) action_handler: Mutex<Box<dyn ActionHandler + Send>>,
    pub(crate) app_context: Arc<RwLock<AppContext>>,
    pub(crate) root_window_bounds: RwLock<WindowBounds>,
}

impl Context {
    pub(crate) fn new(
        tree: Tree,
        action_handler: Box<dyn ActionHandler + Send>,
        app_context: &Arc<RwLock<AppContext>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            tree: RwLock::new(tree),
            action_handler: Mutex::new(action_handler),
            app_context: app_context.clone(),
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

    pub fn do_action(&self, request: ActionRequest) {
        self.action_handler.lock().unwrap().do_action(request);
    }
}

pub(crate) struct AdapterAndContext(usize, Weak<Context>);

impl AdapterAndContext {
    pub(crate) fn upgrade(&self) -> Option<(usize, Arc<Context>)> {
        self.1.upgrade().map(|context| (self.0, context))
    }
}

static APP_CONTEXT: OnceCell<Arc<RwLock<AppContext>>> = OnceCell::new();

pub(crate) struct AppContext {
    pub(crate) name: String,
    pub(crate) toolkit_name: String,
    pub(crate) toolkit_version: String,
    pub(crate) id: Option<i32>,
    pub(crate) desktop_address: Option<OwnedObjectAddress>,
    pub(crate) adapters: Vec<AdapterAndContext>,
}

impl AppContext {
    pub(crate) fn get_or_init(
        name: String,
        toolkit_name: String,
        toolkit_version: String,
    ) -> Arc<RwLock<Self>> {
        APP_CONTEXT
            .get_or_init(|| {
                Arc::new(RwLock::new(Self {
                    name,
                    toolkit_name,
                    toolkit_version,
                    id: None,
                    desktop_address: None,
                    adapters: Vec::new(),
                }))
            })
            .clone()
    }

    pub(crate) fn adapter_index(&self, id: usize) -> Result<usize, usize> {
        self.adapters.binary_search_by(|adapter| adapter.0.cmp(&id))
    }

    pub(crate) fn push_adapter(&mut self, id: usize, context: &Arc<Context>) -> usize {
        let index = self.adapters.len();
        self.adapters
            .push(AdapterAndContext(id, Arc::downgrade(context)));
        index
    }

    pub(crate) fn remove_adapter(&mut self, id: usize) {
        if let Ok(index) = self.adapter_index(id) {
            self.adapters.remove(index);
        }
    }
}
