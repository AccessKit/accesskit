// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::Tree;
use async_channel::Sender;
use async_lock::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard};
use async_once_cell::OnceCell as AsyncOnceCell;
use atspi::proxy::bus::StatusProxy;
use futures_util::{pin_mut, select, StreamExt};
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use zbus::{Connection, Task};

use crate::{
    adapter::{LazyAdapter, Message},
    atspi::{interfaces::Event, map_or_ignoring_broken_pipe, Bus, OwnedObjectAddress},
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
    pub(crate) messages: Option<Sender<Message>>,
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
                    messages: None,
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

    pub(crate) fn push_adapter(&mut self, id: usize, context: &Arc<Context>) {
        self.adapters
            .push(AdapterAndContext(id, Arc::downgrade(context)));
    }

    pub(crate) fn remove_adapter(&mut self, id: usize) {
        if let Ok(index) = self.adapter_index(id) {
            self.adapters.remove(index);
        }
    }
}

pub(crate) struct ActivationContext {
    _task: Option<Task<()>>,
    adapters: Vec<(usize, LazyAdapter)>,
}

static ACTIVATION_CONTEXT: AsyncOnceCell<Arc<AsyncMutex<ActivationContext>>> = AsyncOnceCell::new();

impl ActivationContext {
    async fn get_or_init<'a>() -> AsyncMutexGuard<'a, ActivationContext> {
        ACTIVATION_CONTEXT
            .get_or_init(async {
                let task = Connection::session().await.ok().map(|session_bus| {
                    let session_bus_copy = session_bus.clone();
                    session_bus.executor().spawn(
                        async move {
                            listen(session_bus_copy).await.unwrap();
                        },
                        "accesskit_task",
                    )
                });
                Arc::new(AsyncMutex::new(ActivationContext {
                    _task: task,
                    adapters: Vec::new(),
                }))
            })
            .await
            .lock()
            .await
    }

    pub(crate) async fn activate_eventually(id: usize, adapter: LazyAdapter) {
        let mut activation_context = ActivationContext::get_or_init().await;
        activation_context.adapters.push((id, adapter));
        let is_a11y_enabled = AppContext::get_or_init().messages.is_some();
        if is_a11y_enabled {
            let adapter = &activation_context.adapters.last().unwrap().1;
            adapter.as_ref().await;
        }
    }

    pub(crate) async fn remove_adapter(id: usize) {
        if let Some(activation_context) = ACTIVATION_CONTEXT.get() {
            let mut context = activation_context.lock().await;
            if let Ok(index) = context
                .adapters
                .binary_search_by(|adapter| adapter.0.cmp(&id))
            {
                context.adapters.remove(index);
            }
        }
    }
}

async fn listen(session_bus: Connection) -> zbus::Result<()> {
    let status = StatusProxy::new(&session_bus).await?;
    let changes = status.receive_is_enabled_changed().await.fuse();
    pin_mut!(changes);
    let (tx, rx) = async_channel::unbounded();
    let messages = rx.fuse();
    pin_mut!(messages);
    let mut atspi_bus = None;

    loop {
        select! {
            change = changes.next() => {
                atspi_bus = if let Some(change) = change {
                    if change.get().await? {
                        map_or_ignoring_broken_pipe(Bus::new(&session_bus).await, None, Some)?
                    } else {
                        None
                    }
                } else {
                    None
                };
                {
                    let mut app_context = AppContext::get_or_init();
                    app_context.messages = Some(tx.clone());
                }
                if atspi_bus.is_some() {
                    if let Some(activation_context) = ACTIVATION_CONTEXT.get() {
                        let activation_context = activation_context.lock().await;
                        for (_, adapter) in &activation_context.adapters {
                            adapter.as_ref().await.register_tree().await;
                        }
                    }
                }
            }
            message = messages.next() => {
                if let Some((message, atspi_bus)) = message.zip(atspi_bus.as_ref()) {
                    process_adapter_message(atspi_bus, message).await?;
                }
            }
            complete => return Ok(()),
        }
    }
}

async fn process_adapter_message(bus: &Bus, message: Message) -> zbus::Result<()> {
    match message {
        Message::RegisterInterfaces {
            adapter_id,
            context,
            node_id,
            interfaces,
        } => {
            bus.register_interfaces(adapter_id, context, node_id, interfaces)
                .await?
        }
        Message::UnregisterInterfaces {
            adapter_id,
            node_id,
            interfaces,
        } => {
            bus.unregister_interfaces(adapter_id, node_id, interfaces)
                .await?
        }
        Message::EmitEvent(Event::Object { target, event }) => {
            bus.emit_object_event(target, event).await?
        }
        Message::EmitEvent(Event::Window {
            target,
            name,
            event,
        }) => bus.emit_window_event(target, name, event).await?,
    }

    Ok(())
}
