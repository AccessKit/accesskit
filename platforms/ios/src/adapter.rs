// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node as NodeProvider,
    NodeId, Role, Tree as TreeData, TreeId, TreeUpdate,
};
use accesskit_consumer::{FilterResult, Tree};
use objc2::{
    ClassType, DeclaredClass, declare_class, msg_send_id,
    mutability::MainThreadOnly,
    rc::{Retained, WeakId},
    runtime::AnyObject,
    sel,
};
use objc2_foundation::{
    CGPoint, MainThreadMarker, NSArray, NSNotification, NSNotificationCenter, NSNotificationName,
    NSObject,
};
use objc2_ui_kit::{
    UIAccessibilityBoldTextStatusDidChangeNotification,
    UIAccessibilityDarkerSystemColorsStatusDidChangeNotification,
    UIAccessibilityInvertColorsStatusDidChangeNotification, UIAccessibilityIsSpeakScreenEnabled,
    UIAccessibilityIsSwitchControlRunning, UIAccessibilityIsVoiceOverRunning,
    UIAccessibilityOnOffSwitchLabelsDidChangeNotification,
    UIAccessibilityReduceMotionStatusDidChangeNotification,
    UIAccessibilitySpeakScreenStatusDidChangeNotification,
    UIAccessibilitySwitchControlStatusDidChangeNotification,
    UIAccessibilityVideoAutoplayStatusDidChangeNotification,
    UIAccessibilityVoiceOverStatusDidChangeNotification, UIView,
};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::{Rc, Weak};
use std::{ffi::c_void, ptr::null_mut};

use crate::{
    context::{ActionHandlerNoMut, ActionHandlerWrapper, Context},
    event::{EventGenerator, QueuedEvents, layout_event, screen_changed_event},
    filters::filter,
    node::PlatformNode,
    util::from_cg_point,
};

const PLACEHOLDER_ROOT_ID: NodeId = NodeId(0);

enum State {
    Inactive {
        view: WeakId<UIView>,
        action_handler: Rc<dyn ActionHandlerNoMut>,
        mtm: MainThreadMarker,
    },
    Placeholder {
        placeholder_context: Rc<Context>,
        action_handler: Rc<dyn ActionHandlerNoMut>,
    },
    Active(Rc<Context>),
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Inactive {
                view,
                action_handler: _,
                mtm,
            } => f
                .debug_struct("Inactive")
                .field("view", view)
                .field("mtm", mtm)
                .finish(),
            State::Placeholder {
                placeholder_context,
                action_handler: _,
            } => f
                .debug_struct("Placeholder")
                .field("placeholder_context", placeholder_context)
                .finish(),
            State::Active(context) => f.debug_struct("Active").field("context", context).finish(),
        }
    }
}

struct PlaceholderActionHandler;

impl ActionHandler for PlaceholderActionHandler {
    fn do_action(&mut self, _request: ActionRequest) {}
}

fn any_assistive_tech_running() -> bool {
    unsafe {
        UIAccessibilityIsVoiceOverRunning().as_bool()
            || UIAccessibilityIsSwitchControlRunning().as_bool()
            || UIAccessibilityIsSpeakScreenEnabled().as_bool()
    }
}

fn observed_notification_names() -> [&'static NSNotificationName; 9] {
    unsafe {
        [
            UIAccessibilityVoiceOverStatusDidChangeNotification,
            UIAccessibilitySwitchControlStatusDidChangeNotification,
            UIAccessibilitySpeakScreenStatusDidChangeNotification,
            UIAccessibilityInvertColorsStatusDidChangeNotification,
            UIAccessibilityReduceMotionStatusDidChangeNotification,
            UIAccessibilityBoldTextStatusDidChangeNotification,
            UIAccessibilityDarkerSystemColorsStatusDidChangeNotification,
            UIAccessibilityOnOffSwitchLabelsDidChangeNotification,
            UIAccessibilityVideoAutoplayStatusDidChangeNotification,
        ]
    }
}

type ActivationHandlerCell = RefCell<Box<dyn ActivationHandler>>;
type DeactivationHandlerCell = RefCell<Box<dyn DeactivationHandler>>;

/// Ensure `state` is `Active` or `Placeholder`. If currently `Inactive`,
/// call the activation handler outside the state borrow and commit the
/// resulting transition on re-borrow. If a re-entrant path moved us out
/// of `Inactive` while the handler was running, the existing state wins.
fn get_or_init_context(
    state: &RefCell<State>,
    activation_handler: &ActivationHandlerCell,
) -> Rc<Context> {
    let (view, action_handler, mtm) = {
        let state = state.borrow();
        match &*state {
            State::Active(context) => return Rc::clone(context),
            State::Placeholder {
                placeholder_context,
                ..
            } => return Rc::clone(placeholder_context),
            State::Inactive {
                view,
                action_handler,
                mtm,
            } => (view.clone(), Rc::clone(action_handler), *mtm),
        }
    };

    let initial = activation_handler.borrow_mut().request_initial_tree();

    let mut state = state.borrow_mut();
    match &*state {
        State::Active(context) => Rc::clone(context),
        State::Placeholder {
            placeholder_context,
            ..
        } => Rc::clone(placeholder_context),
        State::Inactive { .. } => match initial {
            Some(initial_state) => {
                let tree = Tree::new(initial_state, true);
                let context = Context::new(view, tree, action_handler, mtm);
                let result = Rc::clone(&context);
                *state = State::Active(context);
                result
            }
            None => {
                let placeholder_update = TreeUpdate {
                    nodes: vec![(PLACEHOLDER_ROOT_ID, NodeProvider::new(Role::Window))],
                    tree: Some(TreeData::new(PLACEHOLDER_ROOT_ID)),
                    tree_id: TreeId::ROOT,
                    focus: PLACEHOLDER_ROOT_ID,
                };
                let placeholder_tree = Tree::new(placeholder_update, true);
                let placeholder_context = Context::new(
                    view,
                    placeholder_tree,
                    Rc::new(ActionHandlerWrapper::new(PlaceholderActionHandler {})),
                    mtm,
                );
                let result = Rc::clone(&placeholder_context);
                *state = State::Placeholder {
                    placeholder_context,
                    action_handler,
                };
                result
            }
        },
    }
}

fn try_activate(
    state: &RefCell<State>,
    activation_handler: &ActivationHandlerCell,
) -> Option<QueuedEvents> {
    if !any_assistive_tech_running() {
        return None;
    }
    if !matches!(&*state.borrow(), State::Inactive { .. }) {
        return None;
    }
    let context = get_or_init_context(state, activation_handler);
    if !matches!(&*state.borrow(), State::Active(_)) {
        return None;
    }
    let focus_id = context.tree.borrow().state().focus().map(|node| node.id());
    focus_id.map(|id| QueuedEvents::new(context, vec![layout_event(Some(id))]))
}

fn try_deactivate(state: &RefCell<State>, deactivation_handler: &DeactivationHandlerCell) {
    let transitioned = {
        let mut state = state.borrow_mut();
        let (view, action_handler, mtm) = match &*state {
            State::Inactive { .. } => return,
            State::Placeholder {
                placeholder_context,
                action_handler,
            } => (
                placeholder_context.view.clone(),
                Rc::clone(action_handler),
                placeholder_context.mtm,
            ),
            State::Active(context) => (
                context.view.clone(),
                Rc::clone(&context.action_handler),
                context.mtm,
            ),
        };
        *state = State::Inactive {
            view,
            action_handler,
            mtm,
        };
        true
    };
    if transitioned {
        deactivation_handler.borrow_mut().deactivate_accessibility();
    }
}

struct StatusObserverIvars {
    state: Weak<RefCell<State>>,
    activation_handler: Weak<ActivationHandlerCell>,
    deactivation_handler: Weak<DeactivationHandlerCell>,
}

declare_class!(
    struct StatusObserver;

    unsafe impl ClassType for StatusObserver {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "AccessKitAccessibilityStatusObserver";
    }

    impl DeclaredClass for StatusObserver {
        type Ivars = StatusObserverIvars;
    }

    unsafe impl StatusObserver {
        #[method(accessibilityStatusChanged:)]
        fn accessibility_status_changed(&self, _notification: &NSNotification) {
            let ivars = self.ivars();
            if any_assistive_tech_running() {
                if let (Some(state), Some(activation_handler)) =
                    (ivars.state.upgrade(), ivars.activation_handler.upgrade())
                {
                    let _ = get_or_init_context(&state, &activation_handler);
                }
            } else if let (Some(state), Some(deactivation_handler)) =
                (ivars.state.upgrade(), ivars.deactivation_handler.upgrade())
            {
                try_deactivate(&state, &deactivation_handler);
            }
        }
    }
);

impl StatusObserver {
    fn new(
        state: Weak<RefCell<State>>,
        activation_handler: Weak<ActivationHandlerCell>,
        deactivation_handler: Weak<DeactivationHandlerCell>,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(StatusObserverIvars {
            state,
            activation_handler,
            deactivation_handler,
        });
        unsafe { msg_send_id![super(this), init] }
    }
}

/// An AccessKit adapter for an owned `UIView`.
///
/// The adapter bridges an AccessKit tree to UIKit's informal accessibility
/// container protocol. Because UIKit dispatches accessibility queries directly
/// to the view, the caller must own a `UIView` subclass and forward the
/// relevant messages to the adapter. The view must be retained for at least
/// as long as the adapter.
///
/// A typical setup looks like this:
///
/// 1. In the view's initializer, create an `Adapter` with
///    [`Adapter::new`], passing a pointer to the view and the required
///    handlers. Store the adapter alongside the view (e.g. in an
///    associated object or a Rust-side wrapper).
/// 2. Override `isAccessibilityElement` to return the result of
///    [`Adapter::is_accessibility_element`].
/// 3. Override `accessibilityElements` to return the result of
///    [`Adapter::accessibility_elements`].
/// 4. Override `accessibilityHitTest:` to return the result of
///    [`Adapter::hit_test`].
/// 5. In `viewDidAppear:`, call [`Adapter::view_did_appear`] and raise
///    the returned events.
/// 6. Whenever the application's accessibility tree changes, call
///    [`Adapter::update_if_active`] and raise the returned events.
///
/// All adapter methods must be called on the main thread.
pub struct Adapter {
    state: Rc<RefCell<State>>,
    activation_handler: Rc<ActivationHandlerCell>,
    #[allow(dead_code)]
    deactivation_handler: Rc<DeactivationHandlerCell>,
    status_observer: Retained<StatusObserver>,
}

impl Debug for Adapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Adapter")
            .field("state", &self.state)
            .finish()
    }
}

impl Adapter {
    /// Create a new iOS adapter. This function must be called on
    /// the main thread.
    ///
    /// All handlers will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to a `UIView`.
    pub unsafe fn new(
        view: *mut c_void,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
        deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        let view = unsafe { Retained::retain(view as *mut UIView) }.unwrap();
        let view = WeakId::from_retained(&view);
        let mtm = MainThreadMarker::new().unwrap();

        let state = Rc::new(RefCell::new(State::Inactive {
            view,
            action_handler: Rc::new(ActionHandlerWrapper::new(action_handler)),
            mtm,
        }));
        let activation_handler: Rc<ActivationHandlerCell> =
            Rc::new(RefCell::new(Box::new(activation_handler)));
        let deactivation_handler: Rc<DeactivationHandlerCell> =
            Rc::new(RefCell::new(Box::new(deactivation_handler)));

        let status_observer = StatusObserver::new(
            Rc::downgrade(&state),
            Rc::downgrade(&activation_handler),
            Rc::downgrade(&deactivation_handler),
            mtm,
        );
        let center = unsafe { NSNotificationCenter::defaultCenter() };
        for name in observed_notification_names() {
            unsafe {
                center.addObserver_selector_name_object(
                    status_observer.as_ref().as_ref(),
                    sel!(accessibilityStatusChanged:),
                    Some(name),
                    None,
                );
            }
        }

        Self {
            state,
            activation_handler,
            deactivation_handler,
            status_observer,
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    pub fn update_if_active(
        &self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        if matches!(&*self.state.borrow(), State::Inactive { .. }) {
            return None;
        }
        let update = update_factory();
        let mut state = self.state.borrow_mut();
        match &mut *state {
            State::Inactive { .. } => None,
            State::Placeholder {
                placeholder_context,
                action_handler,
            } => {
                let tree = Tree::new(update, true);
                let context = Context::new(
                    placeholder_context.view.clone(),
                    tree,
                    Rc::clone(action_handler),
                    placeholder_context.mtm,
                );
                let focus_id = context.tree.borrow().state().focus().map(|node| node.id());
                let queued_events = focus_id.map(|id| {
                    let events = vec![screen_changed_event(Some(id))];
                    QueuedEvents::new(Rc::clone(&context), events)
                });
                *state = State::Active(context);
                queued_events
            }
            State::Active(context) => {
                let mut event_generator = EventGenerator::new(context.clone());
                {
                    let mut tree = context.tree.borrow_mut();
                    tree.update_and_process_changes(update, &mut event_generator);
                }
                Some(event_generator.into_result())
            }
        }
    }

    /// Called when the host view has just appeared on screen. If an assistive
    /// technology is running, this proactively builds the accessibility tree.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    pub fn view_did_appear(&self) -> Option<QueuedEvents> {
        try_activate(&self.state, &self.activation_handler)
    }

    // UIAccessibilityContainer methods

    /// Indicates whether the view itself is an accessibility element.
    /// This corresponds to `isAccessibilityElement`.
    pub fn is_accessibility_element(&self) -> bool {
        let _ = get_or_init_context(&self.state, &self.activation_handler);
        false
    }

    /// Returns all accessibility elements in the container.
    /// This corresponds to `accessibilityElements`.
    pub fn accessibility_elements(&self) -> *mut NSArray<NSObject> {
        let context = get_or_init_context(&self.state, &self.activation_handler);
        let tree = context.tree.borrow();
        let state = tree.state();
        let node = state.root();

        let platform_nodes = if filter(&node) == FilterResult::Include {
            context
                .get_or_create_platform_node(node.id())
                .map(PlatformNode::into_ns_object)
                .into_iter()
                .collect::<Vec<Retained<NSObject>>>()
        } else {
            node.filtered_children(filter)
                .filter_map(|node| context.get_or_create_platform_node(node.id()))
                .map(PlatformNode::into_ns_object)
                .collect::<Vec<Retained<NSObject>>>()
        };

        let array = NSArray::from_vec(platform_nodes);
        Retained::autorelease_return(array)
    }

    // UIAccessibilityHitTest methods

    /// Returns the accessibility element at the specified point.
    /// This corresponds to `accessibilityHitTest:`.
    pub fn hit_test(&self, point: CGPoint) -> *mut NSObject {
        let context = get_or_init_context(&self.state, &self.activation_handler);
        let view = match context.view.load() {
            Some(view) => view,
            None => return null_mut(),
        };
        let tree = context.tree.borrow();
        let tree_state = tree.state();
        let root = tree_state.root();
        let Some(point) = from_cg_point(&view, &root, point) else {
            return null_mut();
        };
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        match context.get_or_create_platform_node(node.id()) {
            Some(platform_node) => Retained::autorelease_return(platform_node) as *mut _,
            None => null_mut(),
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        let center = unsafe { NSNotificationCenter::defaultCenter() };
        unsafe {
            let observer: &AnyObject = self.status_observer.as_ref().as_ref();
            center.removeObserver(observer);
        }
    }
}
