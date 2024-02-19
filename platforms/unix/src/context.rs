// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::{AppContext, Event};
#[cfg(not(feature = "tokio"))]
use async_channel::{Receiver, Sender};
use atspi::proxy::bus::StatusProxy;
#[cfg(not(feature = "tokio"))]
use futures_util::{pin_mut as pin, select, StreamExt};
use once_cell::sync::{Lazy, OnceCell};
use std::{
    sync::{Arc, RwLock},
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
    atspi::{map_or_ignoring_broken_pipe, Bus},
    executor::Executor,
    util::block_on,
};

static APP_CONTEXT: OnceCell<Arc<RwLock<AppContext>>> = OnceCell::new();
static MESSAGES: OnceCell<Sender<Message>> = OnceCell::new();

pub(crate) fn get_or_init_app_context<'a>() -> &'a Arc<RwLock<AppContext>> {
    APP_CONTEXT.get_or_init(AppContext::new)
}

pub(crate) fn get_or_init_messages() -> Sender<Message> {
    MESSAGES
        .get_or_init(|| {
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

            tx
        })
        .clone()
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
                        Lazy::force(adapter);
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
                Lazy::force(adapter);
            }
        }
        Message::RemoveAdapter { id } => {
            if let Ok(index) = adapters.binary_search_by(|adapter| adapter.0.cmp(&id)) {
                adapters.remove(index);
            }
        }
        Message::RegisterInterfaces { node, interfaces } => {
            if let Some(bus) = atspi_bus {
                bus.register_interfaces(node, interfaces).await?
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
        Message::EmitEvent {
            adapter_id,
            event: Event::Object { target, event },
        } => {
            if let Some(bus) = atspi_bus {
                bus.emit_object_event(adapter_id, target, event).await?
            }
        }
        Message::EmitEvent {
            adapter_id,
            event:
                Event::Window {
                    target,
                    name,
                    event,
                },
        } => {
            if let Some(bus) = atspi_bus {
                bus.emit_window_event(adapter_id, target, name, event)
                    .await?;
            }
        }
    }

    Ok(())
}
