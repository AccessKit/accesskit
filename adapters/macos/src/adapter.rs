// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    context::{ActionHandlerNoMut, ActionHandlerWrapper, Context},
    event::{EventGenerator, QueuedEvents, focus_event},
    filters::filter,
    node::can_be_focused,
    util::*,
};
use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TreeId, TreeInfo,
    TreeUpdate,
};
use accesskit_consumer::{FilterResult, Tree};
use objc2::rc::{Id, WeakId};
use objc2_app_kit::NSView;
use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSPoint};
use std::fmt::{Debug, Formatter};
use std::{ffi::c_void, ptr::null_mut, rc::Rc};

const PLACEHOLDER_ROOT_ID: NodeId = NodeId(0);

/// Which phase of activation the adapter is in; see
/// [`Adapter::activation_state`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationState {
    /// No assistive technology has activated the adapter yet. The factory
    /// passed to [`Adapter::update_if_active`] is never called in this phase,
    /// so a host that prepares its updates eagerly can skip that work.
    Inactive,
    /// An assistive technology activated the adapter while
    /// [`ActivationHandler::request_initial_tree`] returned `None`. The
    /// assistive technology currently sees a placeholder tree, and the next
    /// update passed to [`Adapter::update_if_active`] must be a full tree.
    Placeholder,
    /// The tree is live; updates are applied incrementally.
    Active,
}

enum State {
    Inactive {
        view: WeakId<NSView>,
        is_view_focused: bool,
        action_handler: Rc<dyn ActionHandlerNoMut>,
        mtm: MainThreadMarker,
    },
    Placeholder {
        placeholder_context: Rc<Context>,
        is_view_focused: bool,
        action_handler: Rc<dyn ActionHandlerNoMut>,
    },
    Active(Rc<Context>),
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Inactive {
                view,
                is_view_focused,
                action_handler: _,
                mtm,
            } => f
                .debug_struct("Inactive")
                .field("view", view)
                .field("is_view_focused", is_view_focused)
                .field("mtm", mtm)
                .finish(),
            State::Placeholder {
                placeholder_context,
                is_view_focused,
                action_handler: _,
            } => f
                .debug_struct("Placeholder")
                .field("placeholder_context", placeholder_context)
                .field("is_view_focused", is_view_focused)
                .finish(),
            State::Active(context) => f.debug_struct("Active").field("context", context).finish(),
        }
    }
}

struct PlaceholderActionHandler;

impl ActionHandler for PlaceholderActionHandler {
    fn do_action(&mut self, _request: ActionRequest) {}
}

#[derive(Debug)]
pub struct Adapter {
    state: State,
}

impl Adapter {
    /// Create a new macOS adapter. This function must be called on
    /// the main thread.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    pub unsafe fn new(
        view: *mut c_void,
        is_view_focused: bool,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        let view = unsafe { Id::retain(view as *mut NSView) }.unwrap();
        let view = WeakId::from_id(&view);
        let mtm = MainThreadMarker::new().unwrap();
        let state = State::Inactive {
            view,
            is_view_focused,
            action_handler: Rc::new(ActionHandlerWrapper::new(action_handler)),
            mtm,
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
    pub fn update_if_active(
        &mut self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        match &self.state {
            State::Inactive { .. } => None,
            State::Placeholder {
                placeholder_context,
                is_view_focused,
                action_handler,
            } => {
                let tree = Tree::new(update_factory(), *is_view_focused);
                let context = Context::new(
                    placeholder_context.view.clone(),
                    tree,
                    Rc::clone(action_handler),
                    placeholder_context.mtm,
                );
                let result = context.tree.borrow().state().focus().map(|node| {
                    QueuedEvents::new(Rc::clone(&context), vec![focus_event(node.id())])
                });
                self.state = State::Active(context);
                result
            }
            State::Active(context) => {
                let mut event_generator = EventGenerator::new(context.clone());
                let mut tree = context.tree.borrow_mut();
                tree.update_and_process_changes(update_factory(), &mut event_generator);
                Some(event_generator.into_result())
            }
        }
    }

    /// The phase of activation the adapter is in.
    ///
    /// This is a read-only probe; it never changes the state. A host whose
    /// [`ActivationHandler::request_initial_tree`] returns `None` otherwise
    /// has to mirror the adapter's state machine itself to know when the
    /// one full tree that [`Adapter::update_if_active`] then requires is due,
    /// and a host that prepares its updates eagerly has no other way to tell
    /// that nobody will consume them yet.
    pub fn activation_state(&self) -> ActivationState {
        match &self.state {
            State::Inactive { .. } => ActivationState::Inactive,
            State::Placeholder { .. } => ActivationState::Placeholder,
            State::Active(_) => ActivationState::Active,
        }
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    pub fn update_view_focus_state(&mut self, is_focused: bool) -> Option<QueuedEvents> {
        match &mut self.state {
            State::Inactive {
                is_view_focused, ..
            } => {
                *is_view_focused = is_focused;
                None
            }
            State::Placeholder {
                is_view_focused, ..
            } => {
                *is_view_focused = is_focused;
                None
            }
            State::Active(context) => {
                let mut event_generator = EventGenerator::new(context.clone());
                let mut tree = context.tree.borrow_mut();
                tree.update_host_focus_state_and_process_changes(is_focused, &mut event_generator);
                Some(event_generator.into_result())
            }
        }
    }

    fn get_or_init_context<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> Rc<Context> {
        match &self.state {
            State::Inactive {
                view,
                is_view_focused,
                action_handler,
                mtm,
            } => match activation_handler.request_initial_tree() {
                Some(initial_state) => {
                    let tree = Tree::new(initial_state, *is_view_focused);
                    let context = Context::new(view.clone(), tree, Rc::clone(action_handler), *mtm);
                    let result = Rc::clone(&context);
                    self.state = State::Active(context);
                    result
                }
                None => {
                    let placeholder_update = TreeUpdate {
                        nodes: vec![(PLACEHOLDER_ROOT_ID, Node::new(Role::Window))],
                        tree: Some(TreeInfo::new(PLACEHOLDER_ROOT_ID)),
                        tree_id: TreeId::ROOT,
                        focus: PLACEHOLDER_ROOT_ID,
                    };
                    let placeholder_tree = Tree::new(placeholder_update, false);
                    let placeholder_context = Context::new(
                        view.clone(),
                        placeholder_tree,
                        Rc::new(ActionHandlerWrapper::new(PlaceholderActionHandler {})),
                        *mtm,
                    );
                    let result = Rc::clone(&placeholder_context);
                    self.state = State::Placeholder {
                        placeholder_context,
                        is_view_focused: *is_view_focused,
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

    pub fn view_children<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> *mut NSArray<NSObject> {
        let context = self.get_or_init_context(activation_handler);
        let tree = context.tree.borrow();
        let state = tree.state();
        let node = state.root();
        let platform_nodes = if filter(&node) == FilterResult::Include {
            vec![Id::into_super(Id::into_super(
                context.get_or_create_platform_node(node.id()),
            ))]
        } else {
            node.filtered_children(filter)
                .map(|node| {
                    Id::into_super(Id::into_super(
                        context.get_or_create_platform_node(node.id()),
                    ))
                })
                .collect::<Vec<Id<NSObject>>>()
        };
        let array = NSArray::from_vec(platform_nodes);
        Id::autorelease_return(array)
    }

    pub fn focus<H: ActivationHandler + ?Sized>(
        &mut self,
        activation_handler: &mut H,
    ) -> *mut NSObject {
        let context = self.get_or_init_context(activation_handler);
        let tree = context.tree.borrow();
        let state = tree.state();
        if let Some(node) = state.focus() {
            if can_be_focused(&node) {
                return Id::autorelease_return(context.get_or_create_platform_node(node.id()))
                    as *mut _;
            }
        }
        null_mut()
    }

    fn weak_view(&self) -> &WeakId<NSView> {
        match &self.state {
            State::Inactive { view, .. } => view,
            State::Placeholder {
                placeholder_context,
                ..
            } => &placeholder_context.view,
            State::Active(context) => &context.view,
        }
    }

    pub fn hit_test<H: ActivationHandler + ?Sized>(
        &mut self,
        point: NSPoint,
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
        let point = from_ns_point(&view, &root, point);
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        Id::autorelease_return(context.get_or_create_platform_node(node.id())) as *mut _
    }
}
