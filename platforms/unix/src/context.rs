// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::Tree;
#[cfg(not(feature = "tokio"))]
use async_channel::{Receiver, Sender};
use atspi::proxy::bus::StatusProxy;
#[cfg(not(feature = "tokio"))]
use futures_util::{pin_mut as pin, select, StreamExt};
use once_cell::sync::OnceCell;
use std::{
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
    thread,
};
#[cfg(feature = "tokio")]
use tokio::{
    pin, select,
    sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender},
};
#[cfg(feature = "tokio")]
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use zbus::{Connection, ConnectionBuilder};

use crate::{
    adapter::{LazyAdapter, Message},
    atspi::{interfaces::Event, map_or_ignoring_broken_pipe, Bus, OwnedObjectAddress},
    executor::Executor,
    util::{block_on, WindowBounds},
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
    pub(crate) messages: Sender<Message>,
    pub(crate) name: Option<String>,
    pub(crate) toolkit_name: Option<String>,
    pub(crate) toolkit_version: Option<String>,
    pub(crate) id: Option<i32>,
    pub(crate) desktop_address: Option<OwnedObjectAddress>,
    pub(crate) adapters: Vec<AdapterAndContext>,
}

impl AppContext {
    fn get_or_init<'a>() -> &'a Arc<RwLock<Self>> {
        APP_CONTEXT.get_or_init(|| {
            #[cfg(not(feature = "tokio"))]
            let (tx, rx) = async_channel::unbounded();
            #[cfg(feature = "tokio")]
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

            thread::spawn(|| {
                let executor = Executor::new();
                block_on(executor.run(async {
                    if let Ok(session_bus) = ConnectionBuilder::session() {
                        if let Ok(session_bus) = session_bus.internal_executor(false).build().await
                        {
                            run_event_loop(&executor, session_bus, rx).await.unwrap();
                        }
                    }
                }))
            });

            Arc::new(RwLock::new(Self {
                messages: tx,
                name: None,
                toolkit_name: None,
                toolkit_version: None,
                id: None,
                desktop_address: None,
                adapters: Vec::new(),
            }))
        })
    }

    pub(crate) fn read<'a>() -> RwLockReadGuard<'a, AppContext> {
        AppContext::get_or_init().read().unwrap()
    }

    pub(crate) fn write<'a>() -> RwLockWriteGuard<'a, AppContext> {
        AppContext::get_or_init().write().unwrap()
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

async fn run_event_loop(
    executor: &Executor<'_>,
    session_bus: Connection,
    rx: Receiver<Message>,
) -> zbus::Result<()> {
    let session_bus_copy = session_bus.clone();
    let _session_bus_task = executor.spawn(
        async move {
            loop {
                session_bus_copy.executor().tick().await;
            }
        },
        "accesskit_session_bus_task",
    );

    let status = StatusProxy::new(&session_bus).await?;
    let changes = status.receive_is_enabled_changed().await.fuse();
    pin!(changes);

    #[cfg(not(feature = "tokio"))]
    let messages = rx.fuse();
    #[cfg(feature = "tokio")]
    let messages = UnboundedReceiverStream::new(rx).fuse();
    pin!(messages);

    let mut atspi_bus = None;
    let mut adapters: Vec<(usize, LazyAdapter)> = Vec::new();

    loop {
        select! {
            change = changes.next() => {
                atspi_bus = None;
                if let Some(change) = change {
                    if change.get().await? {
                        atspi_bus = map_or_ignoring_broken_pipe(Bus::new(&session_bus, executor).await, None, Some)?;
                    }
                }
                if atspi_bus.is_some() {
                    for (_, adapter) in &adapters {
                        adapter.register_tree();
                    }
                }
            }
            message = messages.next() => {
                if let Some(message) = message {
                    process_adapter_message(&atspi_bus, &mut adapters, message).await?;
                }
            }
        }
    }
}

async fn process_adapter_message(
    atspi_bus: &Option<Bus>,
    adapters: &mut Vec<(usize, LazyAdapter)>,
    message: Message,
) -> zbus::Result<()> {
    match message {
        Message::AddAdapter { id, adapter } => {
            adapters.push((id, adapter));
            if atspi_bus.is_some() {
                let adapter = &adapters.last_mut().unwrap().1;
                adapter.register_tree();
            }
        }
        Message::RemoveAdapter { id } => {
            if let Ok(index) = adapters.binary_search_by(|adapter| adapter.0.cmp(&id)) {
                adapters.remove(index);
            }
        }
        Message::RegisterInterfaces {
            adapter_id,
            context,
            node_id,
            interfaces,
        } => {
            if let Some(bus) = atspi_bus {
                bus.register_interfaces(adapter_id, context, node_id, interfaces)
                    .await?
            }
        }
        Message::UnregisterInterfaces {
            adapter_id,
            node_id,
            interfaces,
        } => {
            if let Some(bus) = atspi_bus {
                bus.unregister_interfaces(adapter_id, node_id, interfaces)
                    .await?
            }
        }
        Message::EmitEvent(Event::Object { target, event }) => {
            if let Some(bus) = atspi_bus {
                bus.emit_object_event(target, event).await?
            }
        }
        Message::EmitEvent(Event::Window {
            target,
            name,
            event,
        }) => {
            if let Some(bus) = atspi_bus {
                bus.emit_window_event(target, name, event).await?;
            }
        }
    }

    Ok(())
}
