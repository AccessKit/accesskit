// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
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
use std::{ffi::c_void, ptr::null_mut, sync::Mutex};

use crate::{Adapter, event::QueuedEvents};

static SUBCLASSES: Mutex<Vec<(&'static AnyClass, &'static AnyClass)>> = Mutex::new(Vec::new());

static ASSOCIATED_OBJECT_KEY: u8 = 0;

fn associated_object_key() -> *const c_void {
    (&ASSOCIATED_OBJECT_KEY as *const u8).cast()
}

struct AssociatedObjectIvars {
    adapter: Adapter,
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
        prev_class: &'static AnyClass,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(AssociatedObjectIvars {
            adapter,
            prev_class,
        });

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

// Unlike `[view class]`, this sees the subclass that key-value observing
// hides by overriding `class`.
fn runtime_class(view: &UIView) -> &'static AnyClass {
    let object: &AnyObject = view;
    object.class()
}

// Key-value observing may have subclassed our subclass in turn.
fn installed_subclass(view: &UIView) -> Option<(&'static AnyClass, &'static AnyClass)> {
    let subclasses = SUBCLASSES.lock().unwrap();
    let mut class: Option<&AnyClass> = Some(runtime_class(view));
    while let Some(current) = class {
        if let Some(entry) = subclasses
            .iter()
            .find(|entry| std::ptr::eq(entry.1, current))
        {
            return Some(*entry);
        }
        class = current.superclass();
    }
    None
}

fn prev_class(view: &UIView) -> &'static AnyClass {
    match associated_object(view) {
        Some(associated) => associated.ivars().prev_class,
        None => installed_subclass(view).unwrap().0,
    }
}

// Some view classes assume that they are the lowest subclass,
// and call [self superclass] to get their superclass.
// Give them the answer they need.
unsafe extern "C" fn superclass(this: &UIView, _cmd: Sel) -> Option<&AnyClass> {
    prev_class(this).superclass()
}

// UIAccessibilityContainer methods

unsafe extern "C" fn is_accessibility_element(this: &UIView, _cmd: Sel) -> Bool {
    let Some(associated) = associated_object(this) else {
        return unsafe { msg_send![super(this, prev_class(this)), isAccessibilityElement] };
    };
    Bool::new(associated.ivars().adapter.is_accessibility_element())
}

unsafe extern "C" fn accessibility_elements(this: &UIView, _cmd: Sel) -> *mut NSArray<NSObject> {
    let Some(associated) = associated_object(this) else {
        return unsafe { msg_send![super(this, prev_class(this)), accessibilityElements] };
    };
    associated.ivars().adapter.accessibility_elements()
}

// UIAccessibilityHitTest methods

unsafe extern "C" fn accessibility_hit_test(
    this: &UIView,
    _cmd: Sel,
    point: CGPoint,
) -> *mut AnyObject {
    let Some(associated) = associated_object(this) else {
        let prev_class = prev_class(this);
        if prev_class
            .instance_method(sel!(accessibilityHitTest:))
            .is_none()
        {
            return null_mut();
        }
        return unsafe { msg_send![super(this, prev_class), accessibilityHitTest: point] };
    };
    associated.ivars().adapter.hit_test(point) as *mut AnyObject
}

// UIView lifecycle

unsafe extern "C" fn did_move_to_window(this: &UIView, _cmd: Sel) {
    unsafe {
        let _: () = msg_send![super(this, prev_class(this)), didMoveToWindow];
    }

    if this.window().is_none() {
        return;
    }

    let Some(associated) = associated_object(this) else {
        return;
    };
    if let Some(events) = associated.ivars().adapter.view_did_appear() {
        events.raise();
    }
}

fn subclass_of(prev_class: &'static AnyClass) -> &'static AnyClass {
    let mut subclasses = SUBCLASSES.lock().unwrap();
    match subclasses
        .iter()
        .find(|entry| std::ptr::eq(entry.0, prev_class))
    {
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
    /// the first time, and before anything observes the view with
    /// key-value observing.
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
        let view = view as *mut UIView;
        let retained_view = unsafe { Retained::retain(view) }.unwrap();
        Self::new_internal(
            retained_view,
            activation_handler,
            action_handler,
            deactivation_handler,
        )
    }

    fn new_internal(
        retained_view: Retained<UIView>,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
        deactivation_handler: impl 'static + DeactivationHandler,
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
        // A previous adapter may have left its subclass on the view,
        // if key-value observing had subclassed it in turn.
        let installed = installed_subclass(unsafe { &*view });
        if installed.is_none() {
            // Key-value observing crashes when it adds or removes an observer
            // while our subclass sits on top of its own.
            let reported_class: &AnyClass = unsafe { msg_send![view, class] };
            let actual_class = runtime_class(unsafe { &*view });
            if !std::ptr::eq(reported_class, actual_class) {
                panic!(
                    "view {view:?} is already observed with key-value observing (its class {} reports itself as {}); create the subclassing adapter before anything observes the view",
                    actual_class.name(),
                    reported_class.name()
                );
            }
        }
        let adapter = unsafe {
            Adapter::new(
                view as *mut c_void,
                activation_handler,
                action_handler,
                deactivation_handler,
            )
        };
        let (prev_class, subclass) = installed.unwrap_or_else(|| {
            let prev_class = runtime_class(unsafe { &*view });
            (prev_class, subclass_of(prev_class))
        });
        let associated = AssociatedObject::new(adapter, prev_class, mtm);
        unsafe {
            objc_setAssociatedObject(
                view as *mut _,
                associated_object_key(),
                Retained::as_ptr(&associated) as *mut _,
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            )
        };
        // SAFETY: Changing the view's class is only safe because
        // the subclass doesn't add any instance variables;
        // it uses an associated object instead.
        if installed.is_none() {
            unsafe { object_setClass(view as *mut _, (subclass as *const AnyClass).cast()) };
        }
        let result = Self {
            view: retained_view,
            associated,
        };
        // UIKit won't replay `didMoveToWindow` for a view that is already
        // attached to its window; catch up manually.
        if result.view.window().is_some() {
            if let Some(events) = result.associated.ivars().adapter.view_did_appear() {
                events.raise();
            }
        }
        result
    }

    /// Create an adapter that dynamically subclasses the root view
    /// of the specified window.
    ///
    /// All handlers will always be called on the main thread.
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
        deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        let window = unsafe { &*(window as *const UIWindow) };
        let root_view_controller = window
            .rootViewController()
            .expect("window has no root view controller");
        let retained_view = root_view_controller
            .view()
            .expect("root view controller has no view");
        Self::new_internal(
            retained_view,
            activation_handler,
            action_handler,
            deactivation_handler,
        )
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
        self.associated
            .ivars()
            .adapter
            .update_if_active(update_factory)
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        let prev_class = self.associated.ivars().prev_class;
        let view = Retained::as_ptr(&self.view) as *mut UIView;
        unsafe {
            objc_setAssociatedObject(
                view as *mut _,
                associated_object_key(),
                std::ptr::null_mut(),
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            )
        };
        // Key-value observing may have subclassed our subclass in turn,
        // and resetting the class would then break those observers.
        // Our subclass is harmless without the associated object.
        let view = unsafe { &*view };
        let is_our_subclass = SUBCLASSES
            .lock()
            .unwrap()
            .iter()
            .any(|entry| std::ptr::eq(entry.1, runtime_class(view)));
        if is_our_subclass {
            unsafe {
                object_setClass(
                    view as *const _ as *mut _,
                    (prev_class as *const AnyClass).cast(),
                )
            };
        }
    }
}
