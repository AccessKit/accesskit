// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, Node as NodeProvider, NodeId, Role,
    Tree as TreeData, TreeId, TreeUpdate,
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
    UIAccessibilityOnOffSwitchLabelsDidChangeNotification, UIAccessibilityPostNotification,
    UIAccessibilityReduceMotionStatusDidChangeNotification,
    UIAccessibilityScreenChangedNotification,
    UIAccessibilitySpeakScreenStatusDidChangeNotification,
    UIAccessibilitySwitchControlStatusDidChangeNotification,
    UIAccessibilityVideoAutoplayStatusDidChangeNotification,
    UIAccessibilityVoiceOverStatusDidChangeNotification, UIView,
};
use std::fmt::{Debug, Formatter};
use std::{ffi::c_void, ptr::null_mut, rc::Rc};

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

struct StatusObserverIvars {
    view: WeakId<UIView>,
}

declare_class!(
    #[derive(Debug)]
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
            if !any_assistive_tech_running() {
                return;
            }
            if self.ivars().view.load().is_none() {
                return;
            }
            unsafe {
                UIAccessibilityPostNotification(UIAccessibilityScreenChangedNotification, None);
            }
        }
    }
);

impl StatusObserver {
    fn new(view: WeakId<UIView>, mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(StatusObserverIvars { view });
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
///    [`Adapter::new`], passing a pointer to the view and an
///    [`ActionHandler`]. Store the adapter alongside the view (e.g. in
///    an associated object or a Rust-side wrapper).
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
#[derive(Debug)]
pub struct Adapter {
    state: State,
    status_observer: Retained<StatusObserver>,
}

impl Adapter {
    /// Create a new iOS adapter. This function must be called on
    /// the main thread.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to a `UIView`.
    pub unsafe fn new(view: *mut c_void, action_handler: impl 'static + ActionHandler) -> Self {
        let view = unsafe { Retained::retain(view as *mut UIView) }.unwrap();
        let view = WeakId::from_retained(&view);
        let mtm = MainThreadMarker::new().unwrap();

        let status_observer = StatusObserver::new(view.clone(), mtm);
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

        let state = State::Inactive {
            view,
            action_handler: Rc::new(ActionHandlerWrapper::new(action_handler)),
            mtm,
        };
        Self {
            state,
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
        &mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        match &self.state {
            State::Inactive { .. } => None,
            State::Placeholder {
                placeholder_context,
                action_handler,
            } => {
                let tree = Tree::new(update_factory(), true);
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
                self.state = State::Active(context);
                queued_events
            }
            State::Active(context) => {
                let mut event_generator = EventGenerator::new(context.clone());
                let mut tree = context.tree.borrow_mut();
                tree.update_and_process_changes(update_factory(), &mut event_generator);
                Some(event_generator.into_result())
            }
        }
    }

    /// Called when the host view has just appeared on screen. If an assistive
    /// technology is running, this proactively builds the accessibility tree.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    pub fn view_did_appear<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> Option<QueuedEvents> {
        if !any_assistive_tech_running() {
            return None;
        }
        if !matches!(self.state, State::Inactive { .. }) {
            return None;
        }
        let context = self.get_or_init_context(activation_handler);
        if !matches!(self.state, State::Active(_)) {
            return None;
        }
        let focus_id = context.tree.borrow().state().focus().map(|node| node.id());
        focus_id.map(|id| QueuedEvents::new(context, vec![layout_event(Some(id))]))
    }

    fn get_or_init_context<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> Rc<Context> {
        match &self.state {
            State::Inactive {
                view,
                action_handler,
                mtm,
            } => match activation_handler.request_initial_tree() {
                Some(initial_state) => {
                    let tree = Tree::new(initial_state, true);
                    let context = Context::new(view.clone(), tree, Rc::clone(action_handler), *mtm);
                    let result = Rc::clone(&context);
                    self.state = State::Active(context);
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
                        view.clone(),
                        placeholder_tree,
                        Rc::new(ActionHandlerWrapper::new(PlaceholderActionHandler {})),
                        *mtm,
                    );
                    let result = Rc::clone(&placeholder_context);
                    self.state = State::Placeholder {
                        placeholder_context,
                        action_handler: Rc::clone(action_handler),
                    };
                    result
                }
            },
            State::Placeholder {
                placeholder_context,
                ..
            } => Rc::clone(placeholder_context),
            State::Active(context) => Rc::clone(context),
        }
    }

    fn weak_view(&self) -> &WeakId<UIView> {
        match &self.state {
            State::Inactive { view, .. } => view,
            State::Placeholder {
                placeholder_context,
                ..
            } => &placeholder_context.view,
            State::Active(context) => &context.view,
        }
    }

    // UIAccessibilityContainer methods

    /// Indicates whether the view itself is an accessibility element.
    /// This corresponds to `isAccessibilityElement`.
    pub fn is_accessibility_element<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> bool {
        let _ = self.get_or_init_context(activation_handler);
        false
    }

    /// Returns all accessibility elements in the container.
    /// This corresponds to `accessibilityElements`.
    pub fn accessibility_elements<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> *mut NSArray<NSObject> {
        let context = self.get_or_init_context(activation_handler);
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
    pub fn hit_test<H: ActivationHandler + ?Sized>(
        &mut self,
        point: CGPoint,
        activation_handler: &mut H,
    ) -> *mut NSObject {
        let view = match self.weak_view().load() {
            Some(view) => view,
            None => {
                return null_mut();
            }
        };

        let context = self.get_or_init_context(activation_handler);
        let tree = context.tree.borrow();
        let state = tree.state();
        let root = state.root();
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
        if !matches!(self.state, State::Inactive { .. }) {
            unsafe {
                UIAccessibilityPostNotification(UIAccessibilityScreenChangedNotification, None);
            }
        }
    }
}
