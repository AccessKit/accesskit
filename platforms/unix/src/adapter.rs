// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, NodeId, Rect, TreeUpdate};
use accesskit_atspi_common::{
    Adapter as AdapterImpl, AdapterCallback, AdapterIdToken, Event, PlatformNode, WindowBounds,
};
#[cfg(not(feature = "tokio"))]
use async_channel::Sender;
use atspi::InterfaceSet;
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
#[cfg(feature = "tokio")]
use tokio::sync::mpsc::UnboundedSender as Sender;

use crate::context::{get_or_init_app_context, get_or_init_messages};

struct Callback {
    messages: Sender<Message>,
}

impl Callback {
    fn send_message(&self, message: Message) {
        #[cfg(not(feature = "tokio"))]
        let _ = self.messages.try_send(message);
        #[cfg(feature = "tokio")]
        let _ = self.messages.send(message);
    }
}

impl AdapterCallback for Callback {
    fn register_interfaces(&self, adapter: &AdapterImpl, id: NodeId, interfaces: InterfaceSet) {
        let node = adapter.platform_node(id);
        self.send_message(Message::RegisterInterfaces { node, interfaces });
    }

    fn unregister_interfaces(&self, adapter: &AdapterImpl, id: NodeId, interfaces: InterfaceSet) {
        self.send_message(Message::UnregisterInterfaces {
            adapter_id: adapter.id(),
            node_id: id,
            interfaces,
        })
    }

    fn emit_event(&self, adapter: &AdapterImpl, event: Event) {
        self.send_message(Message::EmitEvent {
            adapter_id: adapter.id(),
            event,
        });
    }
}

pub(crate) type LazyAdapter = Arc<Lazy<AdapterImpl, Box<dyn FnOnce() -> AdapterImpl + Send>>>;

pub struct Adapter {
    messages: Sender<Message>,
    id: usize,
    r#impl: LazyAdapter,
    is_window_focused: Arc<AtomicBool>,
    root_window_bounds: Arc<Mutex<WindowBounds>>,
}

impl Adapter {
    /// Create a new Unix adapter.
    pub fn new(
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let id_token = AdapterIdToken::next();
        let id = id_token.id();
        let messages = get_or_init_messages();
        let is_window_focused = Arc::new(AtomicBool::new(false));
        let root_window_bounds = Arc::new(Mutex::new(Default::default()));
        let r#impl: LazyAdapter = Arc::new(Lazy::new(Box::new({
            let messages = messages.clone();
            let is_window_focused = Arc::clone(&is_window_focused);
            let root_window_bounds = Arc::clone(&root_window_bounds);
            move || {
                AdapterImpl::with_id(
                    id_token,
                    get_or_init_app_context(),
                    Box::new(Callback { messages }),
                    source(),
                    is_window_focused.load(Ordering::Relaxed),
                    *root_window_bounds.lock().unwrap(),
                    action_handler,
                )
            }
        })));
        let adapter = Self {
            id,
            messages,
            r#impl: r#impl.clone(),
            is_window_focused,
            root_window_bounds,
        };
        adapter.send_message(Message::AddAdapter {
            id,
            adapter: r#impl,
        });
        adapter
    }

    pub(crate) fn send_message(&self, message: Message) {
        #[cfg(not(feature = "tokio"))]
        let _ = self.messages.try_send(message);
        #[cfg(feature = "tokio")]
        let _ = self.messages.send(message);
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        let new_bounds = WindowBounds::new(outer, inner);
        {
            let mut bounds = self.root_window_bounds.lock().unwrap();
            *bounds = new_bounds;
        }
        if let Some(r#impl) = Lazy::get(&self.r#impl) {
            r#impl.set_root_window_bounds(new_bounds);
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    pub fn update_if_active(&self, update_factory: impl FnOnce() -> TreeUpdate) {
        if let Some(r#impl) = Lazy::get(&self.r#impl) {
            r#impl.update(update_factory());
        }
    }

    /// Update the tree state based on whether the window is focused.
    pub fn update_window_focus_state(&self, is_focused: bool) {
        self.is_window_focused.store(is_focused, Ordering::SeqCst);
        if let Some(r#impl) = Lazy::get(&self.r#impl) {
            r#impl.update_window_focus_state(is_focused);
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        self.send_message(Message::RemoveAdapter { id: self.id });
    }
}

pub(crate) enum Message {
    AddAdapter {
        id: usize,
        adapter: LazyAdapter,
    },
    RemoveAdapter {
        id: usize,
    },
    RegisterInterfaces {
        node: PlatformNode,
        interfaces: InterfaceSet,
    },
    UnregisterInterfaces {
        adapter_id: usize,
        node_id: NodeId,
        interfaces: InterfaceSet,
    },
    EmitEvent {
        adapter_id: usize,
        event: Event,
    },
}
