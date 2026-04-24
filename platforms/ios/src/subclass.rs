// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActivationHandler, TreeUpdate};
use objc2::{
    ClassType, DeclaredClass,
    declare::ClassBuilder,
    declare_class,
    ffi::{
        OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_getAssociatedObject, objc_setAssociatedObject,
        object_setClass,
    },
    msg_send, msg_send_id,
    mutability::MainThreadOnly,
    rc::Retained,
    runtime::{AnyClass, AnyObject, Bool, Sel},
    sel,
};
use objc2_foundation::{CGPoint, MainThreadMarker, NSArray, NSObject};
use objc2_ui_kit::{UIView, UIWindow};
use std::{cell::RefCell, ffi::c_void, ptr::null_mut, sync::Mutex};

use crate::{Adapter, event::QueuedEvents};

static SUBCLASSES: Mutex<Vec<(&'static AnyClass, &'static AnyClass)>> = Mutex::new(Vec::new());

static ASSOCIATED_OBJECT_KEY: u8 = 0;

fn associated_object_key() -> *const c_void {
    (&ASSOCIATED_OBJECT_KEY as *const u8).cast()
}

struct AssociatedObjectState {
    adapter: Adapter,
    activation_handler: Box<dyn ActivationHandler>,
}

struct AssociatedObjectIvars {
    state: RefCell<AssociatedObjectState>,
    prev_class: &'static AnyClass,
}

declare_class!(
    struct AssociatedObject;

    unsafe impl ClassType for AssociatedObject {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "AccessKitSubclassAssociatedObject";
    }

    impl DeclaredClass for AssociatedObject {
        type Ivars = AssociatedObjectIvars;
    }
);

impl AssociatedObject {
    fn new(
        adapter: Adapter,
        activation_handler: impl 'static + ActivationHandler,
        prev_class: &'static AnyClass,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let state = RefCell::new(AssociatedObjectState {
            adapter,
            activation_handler: Box::new(activation_handler),
        });
        let this = mtm
            .alloc::<Self>()
            .set_ivars(AssociatedObjectIvars { state, prev_class });

        unsafe { msg_send_id![super(this), init] }
    }
}

fn associated_object(view: &UIView) -> Option<&AssociatedObject> {
    unsafe {
        (objc_getAssociatedObject(view as *const UIView as *const _, associated_object_key())
            as *const AssociatedObject)
            .as_ref()
    }
}

// Some view classes assume that they are the lowest subclass,
// and call [self superclass] to get their superclass.
// Give them the answer they need.
unsafe extern "C" fn superclass(this: &UIView, _cmd: Sel) -> Option<&AnyClass> {
    let associated = associated_object(this)?;
    associated.ivars().prev_class.superclass()
}

// UIAccessibilityContainer methods

unsafe extern "C" fn is_accessibility_element(this: &UIView, _cmd: Sel) -> Bool {
    let Some(associated) = associated_object(this) else {
        return Bool::YES;
    };
    let mut state = associated.ivars().state.borrow_mut();
    let state_mut = &mut *state;
    Bool::new(
        state_mut
            .adapter
            .is_accessibility_element(&mut *state_mut.activation_handler),
    )
}

unsafe extern "C" fn accessibility_elements(this: &UIView, _cmd: Sel) -> *mut NSArray<NSObject> {
    let Some(associated) = associated_object(this) else {
        return Retained::autorelease_return(NSArray::new());
    };
    let mut state = associated.ivars().state.borrow_mut();
    let state_mut = &mut *state;
    state_mut
        .adapter
        .accessibility_elements(&mut *state_mut.activation_handler)
}

// UIAccessibilityHitTest methods

unsafe extern "C" fn accessibility_hit_test(
    this: &UIView,
    _cmd: Sel,
    point: CGPoint,
) -> *mut AnyObject {
    let Some(associated) = associated_object(this) else {
        return null_mut();
    };
    let mut state = associated.ivars().state.borrow_mut();
    let state_mut = &mut *state;
    state_mut
        .adapter
        .hit_test(point, &mut *state_mut.activation_handler) as *mut AnyObject
}

// UIView lifecycle

unsafe extern "C" fn did_move_to_window(this: &UIView, _cmd: Sel) {
    let Some(associated) = associated_object(this) else {
        return;
    };
    let prev_class = associated.ivars().prev_class;
    unsafe {
        let _: () = msg_send![super(this, prev_class), didMoveToWindow];
    }

    if this.window().is_none() {
        return;
    }

    let Some(associated) = associated_object(this) else {
        return;
    };
    let events = {
        let mut state = associated.ivars().state.borrow_mut();
        let state_mut = &mut *state;
        state_mut
            .adapter
            .view_did_appear(&mut *state_mut.activation_handler)
    };
    if let Some(events) = events {
        events.raise();
    }
}

/// Uses dynamic Objective-C subclassing to implement the `UIView`
/// accessibility methods when normal subclassing isn't an option.
pub struct SubclassingAdapter {
    view: Retained<UIView>,
    associated: Retained<AssociatedObject>,
}

impl SubclassingAdapter {
    /// Create an adapter that dynamically subclasses the specified view.
    /// This must be done before the view is shown or focused for
    /// the first time.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to a `UIView`.
    pub unsafe fn new(
        view: *mut c_void,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        let view = view as *mut UIView;
        let retained_view = unsafe { Retained::retain(view) }.unwrap();
        Self::new_internal(retained_view, activation_handler, action_handler)
    }

    fn new_internal(
        retained_view: Retained<UIView>,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        let mtm = MainThreadMarker::new().unwrap();
        let view = Retained::as_ptr(&retained_view) as *mut UIView;
        if !unsafe {
            objc_getAssociatedObject(view as *const UIView as *const _, associated_object_key())
        }
        .is_null()
        {
            panic!("subclassing adapter already instantiated on view {view:?}");
        }
        let adapter = unsafe { Adapter::new(view as *mut c_void, action_handler) };
        // Cast to a pointer and back to force the lifetime to 'static
        // SAFETY: We know the class will live as long as the instance,
        // and we only use this reference while the instance is alive.
        let prev_class = unsafe { &*((*view).class() as *const AnyClass) };
        let associated = AssociatedObject::new(adapter, activation_handler, prev_class, mtm);
        unsafe {
            objc_setAssociatedObject(
                view as *mut _,
                associated_object_key(),
                Retained::as_ptr(&associated) as *mut _,
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            )
        };
        let mut subclasses = SUBCLASSES.lock().unwrap();
        let subclass = match subclasses.iter().find(|entry| entry.0 == prev_class) {
            Some(entry) => entry.1,
            None => {
                let name = format!("AccessKitSubclassOf{}", prev_class.name());
                let mut builder = ClassBuilder::new(&name, prev_class).unwrap();
                unsafe {
                    builder.add_method(
                        sel!(superclass),
                        superclass as unsafe extern "C" fn(_, _) -> _,
                    );
                    builder.add_method(
                        sel!(isAccessibilityElement),
                        is_accessibility_element as unsafe extern "C" fn(_, _) -> _,
                    );
                    builder.add_method(
                        sel!(accessibilityElements),
                        accessibility_elements as unsafe extern "C" fn(_, _) -> _,
                    );
                    builder.add_method(
                        sel!(accessibilityHitTest:),
                        accessibility_hit_test as unsafe extern "C" fn(_, _, _) -> _,
                    );
                    builder.add_method(
                        sel!(didMoveToWindow),
                        did_move_to_window as unsafe extern "C" fn(_, _),
                    );
                }
                let class = builder.register();
                subclasses.push((prev_class, class));
                class
            }
        };
        // SAFETY: Changing the view's class is only safe because
        // the subclass doesn't add any instance variables;
        // it uses an associated object instead.
        unsafe { object_setClass(view as *mut _, (subclass as *const AnyClass).cast()) };
        let result = Self {
            view: retained_view,
            associated,
        };
        // UIKit won't replay `didMoveToWindow` for a view that is already
        // attached to its window; catch up manually.
        if result.view.window().is_some() {
            let events = {
                let mut state = result.associated.ivars().state.borrow_mut();
                let state_mut = &mut *state;
                state_mut
                    .adapter
                    .view_did_appear(&mut *state_mut.activation_handler)
            };
            if let Some(events) = events {
                events.raise();
            }
        }
        result
    }

    /// Create an adapter that dynamically subclasses the root view
    /// of the specified window.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `window` must be a valid, unreleased pointer to a `UIWindow`.
    ///
    /// # Panics
    ///
    /// This function panics if the specified window doesn't currently have
    /// a root view controller with a view.
    pub unsafe fn for_window(
        window: *mut c_void,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        let window = unsafe { &*(window as *const UIWindow) };
        let root_view_controller = window
            .rootViewController()
            .expect("window has no root view controller");
        let retained_view = root_view_controller
            .view()
            .expect("root view controller has no view");
        Self::new_internal(retained_view, activation_handler, action_handler)
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
        let mut state = self.associated.ivars().state.borrow_mut();
        state.adapter.update_if_active(update_factory)
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        let prev_class = self.associated.ivars().prev_class;
        let view = Retained::as_ptr(&self.view) as *mut UIView;
        unsafe { object_setClass(view as *mut _, (prev_class as *const AnyClass).cast()) };
        unsafe {
            objc_setAssociatedObject(
                view as *mut _,
                associated_object_key(),
                std::ptr::null_mut(),
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            )
        };
    }
}
