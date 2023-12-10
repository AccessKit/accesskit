// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::Tree;
use async_lock::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard};
use async_once_cell::OnceCell as AsyncOnceCell;
use atspi::proxy::bus::StatusProxy;
use futures_lite::StreamExt;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use zbus::{Connection, Task};

use crate::{
    adapter::LazyAdapter,
    atspi::{Bus, OwnedObjectAddress},
    util::WindowBounds,
};

pub(crate) struct Context {
    pub(crate) tree: RwLock<Tree>,
    pub(crate) action_handler: Mutex<Box<dyn ActionHandler + Send>>,
    pub(crate) root_window_bounds: RwLock<WindowBounds>,
}

impl Context {
    pub(crate) fn new(
        tree: Tree,
        action_handler: Box<dyn ActionHandler + Send>,
        root_window_bounds: WindowBounds,
    ) -> Arc<Self> {
        Arc::new(Self {
            tree: RwLock::new(tree),
            action_handler: Mutex::new(action_handler),
            root_window_bounds: RwLock::new(root_window_bounds),
        })
    }

    pub(crate) fn read_tree(&self) -> RwLockReadGuard<'_, Tree> {
        self.tree.read().unwrap()
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
    pub(crate) atspi_bus: Option<Bus>,
    pub(crate) name: Option<String>,
    pub(crate) toolkit_name: Option<String>,
    pub(crate) toolkit_version: Option<String>,
    pub(crate) id: Option<i32>,
    pub(crate) desktop_address: Option<OwnedObjectAddress>,
    pub(crate) adapters: Vec<AdapterAndContext>,
}

impl AppContext {
    fn get_or_init<'a>() -> RwLockWriteGuard<'a, Self> {
        APP_CONTEXT
            .get_or_init(|| {
                Arc::new(RwLock::new(Self {
                    atspi_bus: None,
                    name: None,
                    toolkit_name: None,
                    toolkit_version: None,
                    id: None,
                    desktop_address: None,
                    adapters: Vec::new(),
                }))
            })
            .write()
            .unwrap()
    }

    pub(crate) fn read<'a>() -> RwLockReadGuard<'a, AppContext> {
        APP_CONTEXT.get().unwrap().read().unwrap()
    }

    pub(crate) fn write<'a>() -> RwLockWriteGuard<'a, AppContext> {
        APP_CONTEXT.get().unwrap().write().unwrap()
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

pub(crate) struct ActivationContext {
    _monitoring_task: Task<()>,
    adapters: Vec<LazyAdapter>,
}

static ACTIVATION_CONTEXT: AsyncOnceCell<Arc<AsyncMutex<ActivationContext>>> = AsyncOnceCell::new();

impl ActivationContext {
    async fn get_or_init<'a>() -> AsyncMutexGuard<'a, ActivationContext> {
        ACTIVATION_CONTEXT
            .get_or_init(async {
                let session_bus = Connection::session().await.unwrap();
                let session_bus_copy = session_bus.clone();
                let monitoring_task = session_bus.executor().spawn(
                    async move {
                        let _ = monitor_a11y_status(session_bus_copy).await;
                    },
                    "accesskit_a11y_monitoring_task",
                );
                Arc::new(AsyncMutex::new(ActivationContext {
                    _monitoring_task: monitoring_task,
                    adapters: Vec::new(),
                }))
            })
            .await
            .lock()
            .await
    }

    pub(crate) async fn activate_eventually(adapter: LazyAdapter) {
        let mut activation_context = ActivationContext::get_or_init().await;
        activation_context.adapters.push(adapter);
        let adapter = activation_context.adapters.last().unwrap();
        let is_a11y_enabled = AppContext::get_or_init().atspi_bus.is_some();
        if is_a11y_enabled {
            adapter.as_ref().await;
        }
    }
}

async fn monitor_a11y_status(session_bus: Connection) -> zbus::Result<()> {
    let status = StatusProxy::new(&session_bus).await?;
    let mut changes = status.receive_is_enabled_changed().await;

    while let Some(change) = changes.next().await {
        if let Ok(true) = change.get().await {
            let atspi_bus = Bus::a11y_bus().await;
            let mut app_context = AppContext::get_or_init();
            app_context.atspi_bus = atspi_bus;
        }
    }

    Ok(())
}
