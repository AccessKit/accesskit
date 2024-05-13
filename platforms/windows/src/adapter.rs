// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    ActionHandler, ActivationHandler, Live, NodeBuilder, NodeId, Role, Tree as TreeData, TreeUpdate,
};
use accesskit_consumer::{FilterResult, Node, Tree, TreeChangeHandler};
use std::{
    collections::HashSet,
    sync::{atomic::Ordering, Arc},
};
use windows::Win32::{
    Foundation::*,
    UI::{Accessibility::*, WindowsAndMessaging::*},
};

use crate::{
    context::{ActionHandlerNoMut, ActionHandlerWrapper, Context},
    filters::filter,
    node::{NodeWrapper, PlatformNode},
    util::QueuedEvent,
};

fn focus_event(context: &Arc<Context>, node_id: NodeId) -> QueuedEvent {
    let platform_node = PlatformNode::new(context, node_id);
    let element: IRawElementProviderSimple = platform_node.into();
    QueuedEvent::Simple {
        element,
        event_id: UIA_AutomationFocusChangedEventId,
    }
}

struct AdapterChangeHandler<'a> {
    context: &'a Arc<Context>,
    queue: Vec<QueuedEvent>,
    text_changed: HashSet<NodeId>,
}

impl<'a> AdapterChangeHandler<'a> {
    fn new(context: &'a Arc<Context>) -> Self {
        Self {
            context,
            queue: Vec::new(),
            text_changed: HashSet::new(),
        }
    }
}

impl AdapterChangeHandler<'_> {
    fn insert_text_change_if_needed_parent(&mut self, node: Node) {
        if !node.supports_text_ranges() {
            return;
        }
        let id = node.id();
        if self.text_changed.contains(&id) {
            return;
        }
        let platform_node = PlatformNode::new(self.context, node.id());
        let element: IRawElementProviderSimple = platform_node.into();
        // Text change events must come before selection change
        // events. It doesn't matter if text change events come
        // before other events.
        self.queue.insert(
            0,
            QueuedEvent::Simple {
                element,
                event_id: UIA_Text_TextChangedEventId,
            },
        );
        self.text_changed.insert(id);
    }

    fn insert_text_change_if_needed(&mut self, node: &Node) {
        if node.role() != Role::InlineTextBox {
            return;
        }
        if let Some(node) = node.filtered_parent(&filter) {
            self.insert_text_change_if_needed_parent(node);
        }
    }
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
    fn node_added(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
        if filter(node) != FilterResult::Include {
            return;
        }
        if node.name().is_some() && node.live() != Live::Off {
            let platform_node = PlatformNode::new(self.context, node.id());
            let element: IRawElementProviderSimple = platform_node.into();
            self.queue.push(QueuedEvent::Simple {
                element,
                event_id: UIA_LiveRegionChangedEventId,
            });
        }
    }

    fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
        if old_node.raw_value() != new_node.raw_value() {
            self.insert_text_change_if_needed(new_node);
        }
        if filter(new_node) != FilterResult::Include {
            return;
        }
        let platform_node = PlatformNode::new(self.context, new_node.id());
        let element: IRawElementProviderSimple = platform_node.into();
        let old_wrapper = NodeWrapper(old_node);
        let new_wrapper = NodeWrapper(new_node);
        new_wrapper.enqueue_property_changes(&mut self.queue, &element, &old_wrapper);
        if new_node.name().is_some()
            && new_node.live() != Live::Off
            && (new_node.name() != old_node.name()
                || new_node.live() != old_node.live()
                || filter(old_node) != FilterResult::Include)
        {
            self.queue.push(QueuedEvent::Simple {
                element,
                event_id: UIA_LiveRegionChangedEventId,
            });
        }
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            self.queue.push(focus_event(self.context, new_node.id()));
        }
    }

    fn node_removed(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
    }

    // TODO: handle other events (#20)
}

const PLACEHOLDER_ROOT_ID: NodeId = NodeId(0);

enum State {
    Inactive {
        hwnd: HWND,
        is_window_focused: bool,
        action_handler: Arc<dyn ActionHandlerNoMut + Send + Sync>,
    },
    Placeholder(Arc<Context>),
    Active(Arc<Context>),
}

pub struct Adapter {
    state: State,
}

impl Adapter {
    /// Creates a new Windows platform adapter.
    ///
    /// The action handler may or may not be called on the thread that owns
    /// the window.
    ///
    /// This must not be called while handling the `WM_GETOBJECT` message,
    /// because this function must initialize UI Automation before
    /// that message is handled. This is necessary to prevent a race condition
    /// that leads to nested `WM_GETOBJECT` messages and, in some cases,
    /// assistive technologies not realizing that the window natively implements.
    /// UIA. See [AccessKit issue #37](https://github.com/AccessKit/accesskit/issues/37)
    /// for more details.
    pub fn new(
        hwnd: HWND,
        is_window_focused: bool,
        action_handler: impl 'static + ActionHandler + Send,
    ) -> Self {
        Self::with_wrapped_action_handler(
            hwnd,
            is_window_focused,
            Arc::new(ActionHandlerWrapper::new(action_handler)),
        )
    }

    // Currently required by the test infrastructure
    pub(crate) fn with_wrapped_action_handler(
        hwnd: HWND,
        is_window_focused: bool,
        action_handler: Arc<dyn ActionHandlerNoMut + Send + Sync>,
    ) -> Self {
        init_uia();

        let state = State::Inactive {
            hwnd,
            is_window_focused,
            action_handler,
        };
        Self { state }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update_if_active(
        &mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        match &self.state {
            State::Inactive { .. } => None,
            State::Placeholder(context) => {
                let is_window_focused = context.read_tree().state().is_host_focused();
                let tree = Tree::new(update_factory(), is_window_focused);
                *context.tree.write().unwrap() = tree;
                context.is_placeholder.store(false, Ordering::SeqCst);
                let result = context
                    .read_tree()
                    .state()
                    .focus_id()
                    .map(|id| QueuedEvents(vec![focus_event(context, id)]));
                self.state = State::Active(Arc::clone(context));
                result
            }
            State::Active(context) => {
                let mut handler = AdapterChangeHandler::new(context);
                let mut tree = context.tree.write().unwrap();
                tree.update_and_process_changes(update_factory(), &mut handler);
                Some(QueuedEvents(handler.queue))
            }
        }
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update_window_focus_state(&mut self, is_focused: bool) -> Option<QueuedEvents> {
        match &mut self.state {
            State::Inactive {
                is_window_focused, ..
            } => {
                *is_window_focused = is_focused;
                None
            }
            State::Placeholder(context) => {
                let mut handler = AdapterChangeHandler::new(context);
                let mut tree = context.tree.write().unwrap();
                tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
                Some(QueuedEvents(handler.queue))
            }
            State::Active(context) => {
                let mut handler = AdapterChangeHandler::new(context);
                let mut tree = context.tree.write().unwrap();
                tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
                Some(QueuedEvents(handler.queue))
            }
        }
    }

    /// Handle the `WM_GETOBJECT` window message. The accessibility tree
    /// is lazily initialized if necessary using the provided
    /// [`ActivationHandler`] implementation.
    ///
    /// This returns an `Option` so the caller can pass the message
    /// to `DefWindowProc` if AccessKit decides not to handle it.
    /// The optional value is an `Into<LRESULT>` rather than simply an `LRESULT`
    /// so the necessary call to UIA, which may lead to a nested `WM_GETOBJECT`
    /// message, can be done outside of any lock that the caller might hold
    /// on the `Adapter` or window state, while still abstracting away
    /// the details of that call to UIA.
    pub fn handle_wm_getobject<H: ActivationHandler + ?Sized>(
        &mut self,
        wparam: WPARAM,
        lparam: LPARAM,
        activation_handler: &mut H,
    ) -> Option<impl Into<LRESULT>> {
        // Don't bother with MSAA object IDs that are asking for something other
        // than the client area of the window. DefWindowProc can handle those.
        // First, cast the lparam to i32, to handle inconsistent conversion
        // behavior in senders.
        let objid = normalize_objid(lparam);
        if objid < 0 && objid != UiaRootObjectId && objid != OBJID_CLIENT.0 {
            return None;
        }

        let (hwnd, platform_node) = match &self.state {
            State::Inactive {
                hwnd,
                is_window_focused,
                action_handler,
            } => match activation_handler.request_initial_tree() {
                Some(initial_state) => {
                    let hwnd = *hwnd;
                    let tree = Tree::new(initial_state, *is_window_focused);
                    let context = Context::new(hwnd, tree, Arc::clone(action_handler), false);
                    let node_id = context.read_tree().state().root_id();
                    let platform_node = PlatformNode::new(&context, node_id);
                    self.state = State::Active(context);
                    (hwnd, platform_node)
                }
                None => {
                    let hwnd = *hwnd;
                    let placeholder_update = TreeUpdate {
                        nodes: vec![(PLACEHOLDER_ROOT_ID, NodeBuilder::new(Role::Window).build())],
                        tree: Some(TreeData::new(PLACEHOLDER_ROOT_ID)),
                        focus: PLACEHOLDER_ROOT_ID,
                    };
                    let placeholder_tree = Tree::new(placeholder_update, *is_window_focused);
                    let context =
                        Context::new(hwnd, placeholder_tree, Arc::clone(action_handler), true);
                    let platform_node = PlatformNode::unspecified_root(&context);
                    self.state = State::Placeholder(context);
                    (hwnd, platform_node)
                }
            },
            State::Placeholder(context) => (context.hwnd, PlatformNode::unspecified_root(context)),
            State::Active(context) => {
                let node_id = context.read_tree().state().root_id();
                (context.hwnd, PlatformNode::new(context, node_id))
            }
        };
        let el: IRawElementProviderSimple = platform_node.into();
        Some(WmGetObjectResult {
            hwnd,
            wparam,
            lparam,
            el,
        })
    }
}

fn init_uia() {
    // `UiaLookupId` is a cheap way of forcing UIA to initialize itself.
    unsafe {
        UiaLookupId(
            AutomationIdentifierType_Property,
            &ControlType_Property_GUID,
        )
    };
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn normalize_objid(lparam: LPARAM) -> i32 {
    (lparam.0 & 0xFFFFFFFF) as _
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn normalize_objid(lparam: LPARAM) -> i32 {
    lparam.0 as _
}

struct WmGetObjectResult {
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    el: IRawElementProviderSimple,
}

impl From<WmGetObjectResult> for LRESULT {
    fn from(this: WmGetObjectResult) -> Self {
        unsafe { UiaReturnRawElementProvider(this.hwnd, this.wparam, this.lparam, &this.el) }
    }
}

/// Events generated by a tree update.
#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents(Vec<QueuedEvent>);

impl QueuedEvents {
    /// Raise all queued events synchronously.
    ///
    /// The window may receive `WM_GETOBJECT` messages during this call.
    /// This means that any locks required by the `WM_GETOBJECT` handler
    /// must not be held when this method is called.
    ///
    /// This method should be called on the thread that owns the window.
    /// It's not clear whether this is a strict requirement of UIA itself,
    /// but based on the known behavior of UIA, MSAA, and some ATs,
    /// it's strongly recommended.
    pub fn raise(self) {
        for event in self.0 {
            match event {
                QueuedEvent::Simple { element, event_id } => {
                    unsafe { UiaRaiseAutomationEvent(&element, event_id) }.unwrap();
                }
                QueuedEvent::PropertyChanged {
                    element,
                    property_id,
                    old_value,
                    new_value,
                } => {
                    unsafe {
                        UiaRaiseAutomationPropertyChangedEvent(
                            &element,
                            property_id,
                            &old_value,
                            &new_value,
                        )
                    }
                    .unwrap();
                }
            }
        }
    }
}

// We explicitly want to allow the queued events to be sent to the UI thread,
// so implement Send even though windows-rs doesn't implement it for all
// contained types. This is safe because we're not using COM threading.
unsafe impl Send for QueuedEvents {}
