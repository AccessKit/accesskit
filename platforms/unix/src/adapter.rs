// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, NodeId, Rect, TreeUpdate};
use accesskit_atspi_common::{
    next_adapter_id, ActionHandlerNoMut, ActionHandlerWrapper, Adapter as AdapterImpl,
    AdapterCallback, Event, PlatformNode, WindowBounds,
};
#[cfg(not(feature = "tokio"))]
use async_channel::Sender;
use atspi::InterfaceSet;
use std::sync::{Arc, Mutex};
#[cfg(feature = "tokio")]
use tokio::sync::mpsc::UnboundedSender as Sender;

use crate::context::{get_or_init_app_context, get_or_init_messages};

pub(crate) struct Callback {
    messages: Sender<Message>,
}

impl Callback {
    pub(crate) fn new() -> Self {
        let messages = get_or_init_messages();
        Self { messages }
    }

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

pub(crate) enum AdapterState {
    Inactive {
        is_window_focused: bool,
        root_window_bounds: WindowBounds,
        action_handler: Arc<dyn ActionHandlerNoMut + Send + Sync>,
    },
    Pending {
        is_window_focused: bool,
        root_window_bounds: WindowBounds,
        action_handler: Arc<dyn ActionHandlerNoMut + Send + Sync>,
    },
    Active(AdapterImpl),
}

pub struct Adapter {
    messages: Sender<Message>,
    id: usize,
    state: Arc<Mutex<AdapterState>>,
}

impl Adapter {
    /// Create a new Unix adapter.
    ///
    /// All of the handlers will always be called from another thread.
    pub fn new(
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        deactivation_handler: impl 'static + DeactivationHandler + Send,
    ) -> Self {
        let id = next_adapter_id();
        let messages = get_or_init_messages();
        let state = Arc::new(Mutex::new(AdapterState::Inactive {
            is_window_focused: false,
            root_window_bounds: Default::default(),
            action_handler: Arc::new(ActionHandlerWrapper::new(action_handler)),
        }));
        let adapter = Self {
            id,
            messages,
            state: Arc::clone(&state),
        };
        adapter.send_message(Message::AddAdapter {
            id,
            activation_handler: Box::new(activation_handler),
            deactivation_handler: Box::new(deactivation_handler),
            state,
        });
        adapter
    }

    pub(crate) fn send_message(&self, message: Message) {
        #[cfg(not(feature = "tokio"))]
        let _ = self.messages.try_send(message);
        #[cfg(feature = "tokio")]
        let _ = self.messages.send(message);
    }

    pub fn set_root_window_bounds(&mut self, outer: Rect, inner: Rect) {
        let new_bounds = WindowBounds::new(outer, inner);
        let mut state = self.state.lock().unwrap();
        match &mut *state {
            AdapterState::Inactive {
                root_window_bounds, ..
            } => {
                *root_window_bounds = new_bounds;
            }
            AdapterState::Pending {
                root_window_bounds, ..
            } => {
                *root_window_bounds = new_bounds;
            }
            AdapterState::Active(r#impl) => r#impl.set_root_window_bounds(new_bounds),
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        let mut state = self.state.lock().unwrap();
        match &mut *state {
            AdapterState::Inactive { .. } => (),
            AdapterState::Pending {
                is_window_focused,
                root_window_bounds,
                action_handler,
            } => {
                let initial_state = update_factory();
                let r#impl = AdapterImpl::with_wrapped_action_handler(
                    self.id,
                    get_or_init_app_context(),
                    Callback::new(),
                    initial_state,
                    *is_window_focused,
                    *root_window_bounds,
                    Arc::clone(action_handler),
                );
                *state = AdapterState::Active(r#impl);
            }
            AdapterState::Active(r#impl) => r#impl.update(update_factory()),
        }
    }

    /// Notifies the adapter that the window is active.
    ///
    /// # Discussion
    ///
    /// The correct use of this function is critical to make your application accessible on Linux.
    ///
    /// Linux's orca screen reader will not read your accessibility tree unless:
    ///
    /// 1.  An immediate child of the root object is in the ATSPI active state
    /// 2.  No two children report the active state with "too much" temporal overlap, including
    ///     across applications.
    ///
    /// Note that there is **no warning or diagnostic in accesskit or platform accessibility inspectors to alert
    /// you to these problems, but your software won't work with the screen reader**.
    ///
    /// ## The ATSPI active state
    ///
    /// `accesskit` only reports the ATSPI active state for the [accesskit::Role::Window] role.
    /// Accordingly, this is the only practical choice for the [accesskit::Tree] `root` field.  Use
    /// of another role in this position is sometimes seen on Linux but not supported by accesskit.
    ///
    /// accesskit reports the ATSPI active state only after this function is called with `true`.
    ///
    /// ## Temporal overlap
    ///
    /// orca gets much of its information about which window to read not from the compositor but by
    /// the self-report of the applications via the ATSPI active state.
    ///
    /// Thus when two eligible children both report the active state, it is unclear which one orca
    /// should read.  orca will retry after a short period to see if the overlap is a transient condition.
    /// However when two children persist in reporting the active state, orca must choose one of them
    /// to win and be read, and one to lose and to be treated as inactive regardless of its report of
    /// active.
    ///
    /// Once orca concludes that the loser's active state was erroneous, this **latches until the losing
    /// state changes**.  That is: when the winner relinquishes its active state, the loser does not
    /// regain it.  Applications that enter this state are likely to be completely inaccessible
    /// to screen reader users unless they call this function with `false` and `true` again.
    ///
    /// Wayland does not provide a portable event to indicate that a surface becomes active/inactive
    /// such as to drive calling this function.  In practice, keyboard Enter and Leave events are
    /// often used instead.
    pub fn update_window_focus_state(&mut self, is_focused: bool) {
        let mut state = self.state.lock().unwrap();
        match &mut *state {
            AdapterState::Inactive {
                is_window_focused, ..
            } => {
                *is_window_focused = is_focused;
            }
            AdapterState::Pending {
                is_window_focused, ..
            } => {
                *is_window_focused = is_focused;
            }
            AdapterState::Active(r#impl) => r#impl.update_window_focus_state(is_focused),
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
        activation_handler: Box<dyn ActivationHandler + Send>,
        deactivation_handler: Box<dyn DeactivationHandler + Send>,
        state: Arc<Mutex<AdapterState>>,
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
