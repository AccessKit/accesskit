// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use icrate::{
    AppKit::{NSView, NSWindow},
    Foundation::{NSArray, NSObject, NSPoint},
};
use objc2::{
    declare::ClassBuilder,
    declare_class,
    ffi::{
        objc_getAssociatedObject, objc_setAssociatedObject, object_setClass,
        OBJC_ASSOCIATION_RETAIN_NONATOMIC,
    },
    msg_send_id,
    mutability::InteriorMutable,
    rc::Id,
    runtime::{AnyClass, Sel},
    sel, ClassType, DeclaredClass,
};
use once_cell::{sync::Lazy as SyncLazy, unsync::Lazy};
use std::{cell::Cell, collections::HashMap, ffi::c_void, rc::Rc, sync::Mutex};

use crate::{event::QueuedEvents, Adapter};

static SUBCLASSES: SyncLazy<Mutex<HashMap<&'static AnyClass, &'static AnyClass>>> =
    SyncLazy::new(|| Mutex::new(HashMap::new()));

// Declare as mutable to ensure the address is unique.
static mut ASSOCIATED_OBJECT_KEY: u8 = 0;

fn associated_object_key() -> *const c_void {
    unsafe { &ASSOCIATED_OBJECT_KEY as *const u8 as *const _ }
}

type LazyAdapter = Lazy<Adapter, Box<dyn FnOnce() -> Adapter>>;

struct AssociatedObjectIvars {
    adapter: LazyAdapter,
    prev_class: &'static AnyClass,
}

declare_class!(
    struct AssociatedObject;

    unsafe impl ClassType for AssociatedObject {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "AccessKitSubclassAssociatedObject";
    }

    impl DeclaredClass for AssociatedObject {
        type Ivars = AssociatedObjectIvars;
    }
);

impl AssociatedObject {
    fn new(adapter: LazyAdapter, prev_class: &'static AnyClass) -> Id<Self> {
        let this = Self::alloc().set_ivars(AssociatedObjectIvars {
            adapter,
            prev_class,
        });

        unsafe { msg_send_id![super(this), init] }
    }
}

fn associated_object(view: &NSView) -> &AssociatedObject {
    unsafe {
        (objc_getAssociatedObject(view as *const NSView as *const _, associated_object_key())
            as *const AssociatedObject)
            .as_ref()
    }
    .unwrap()
}

// Some view classes, like the one in winit 0.27, assume that they are the
// lowest subclass, and call [self superclass] to get their superclass.
// Give them the answer they need.
unsafe extern "C" fn superclass(this: &NSView, _cmd: Sel) -> Option<&AnyClass> {
    let associated = associated_object(this);
    associated.ivars().prev_class.superclass()
}

unsafe extern "C" fn children(this: &NSView, _cmd: Sel) -> *mut NSArray<NSObject> {
    let associated = associated_object(this);
    let adapter = Lazy::force(&associated.ivars().adapter);
    adapter.view_children()
}

unsafe extern "C" fn focus(this: &NSView, _cmd: Sel) -> *mut NSObject {
    let associated = associated_object(this);
    let adapter = Lazy::force(&associated.ivars().adapter);
    adapter.focus()
}

unsafe extern "C" fn hit_test(this: &NSView, _cmd: Sel, point: NSPoint) -> *mut NSObject {
    let associated = associated_object(this);
    let adapter = Lazy::force(&associated.ivars().adapter);
    adapter.hit_test(point)
}

/// Uses dynamic Objective-C subclassing to implement the NSView
/// accessibility methods when normal subclassing isn't an option.
pub struct SubclassingAdapter {
    view: Id<NSView>,
    is_view_focused: Rc<Cell<bool>>,
    associated: Id<AssociatedObject>,
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
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    pub unsafe fn new(
        view: *mut c_void,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let view = view as *mut NSView;
        let retained_view = unsafe { Id::retain(view) }.unwrap();
        Self::new_internal(retained_view, source, action_handler)
    }

    fn new_internal(
        retained_view: Id<NSView>,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let is_view_focused = Rc::new(Cell::new(false));
        let adapter: LazyAdapter = {
            let retained_view = retained_view.clone();
            let is_view_focused = Rc::clone(&is_view_focused);
            Lazy::new(Box::new(move || {
                let view = Id::as_ptr(&retained_view) as *mut c_void;
                unsafe { Adapter::new(view, source(), is_view_focused.get(), action_handler) }
            }))
        };
        let view = Id::as_ptr(&retained_view) as *mut NSView;
        // Cast to a pointer and back to force the lifetime to 'static
        // SAFETY: We know the class will live as long as the instance,
        // and we only use this reference while the instance is alive.
        let prev_class = unsafe { &*((*view).class() as *const AnyClass) };
        let associated = AssociatedObject::new(adapter, prev_class);
        unsafe {
            objc_setAssociatedObject(
                view as *mut _,
                associated_object_key(),
                Id::as_ptr(&associated) as *mut _,
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            )
        };
        let mut subclasses = SUBCLASSES.lock().unwrap();
        let entry = subclasses.entry(prev_class);
        let subclass = entry.or_insert_with(|| {
            let name = format!("AccessKitSubclassOf{}", prev_class.name());
            let mut builder = ClassBuilder::new(&name, prev_class).unwrap();
            unsafe {
                builder.add_method(
                    sel!(superclass),
                    superclass as unsafe extern "C" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(accessibilityChildren),
                    children as unsafe extern "C" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(accessibilityFocusedUIElement),
                    focus as unsafe extern "C" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(accessibilityHitTest:),
                    hit_test as unsafe extern "C" fn(_, _, _) -> _,
                );
            }
            builder.register()
        });
        // SAFETY: Changing the view's class is only safe because
        // the subclass doesn't add any instance variables;
        // it uses an associated object instead.
        unsafe { object_setClass(view as *mut _, (*subclass as *const AnyClass).cast()) };
        Self {
            view: retained_view,
            is_view_focused,
            associated,
        }
    }

    /// Create an adapter that dynamically subclasses the content view
    /// of the specified window.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `window` must be a valid, unreleased pointer to an `NSWindow`.
    ///
    /// # Panics
    ///
    /// This function panics if the specified window doesn't currently have
    /// a content view.
    pub unsafe fn for_window(
        window: *mut c_void,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let window = unsafe { &*(window as *const NSWindow) };
        let retained_view = window.contentView().unwrap();
        Self::new_internal(retained_view, source, action_handler)
    }

    /// Initialize the tree if it hasn't been initialized already, then apply
    /// the provided update.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let adapter = Lazy::force(&self.associated.ivars().adapter);
        adapter.update(update)
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    pub fn update_if_active(
        &self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        Lazy::get(&self.associated.ivars().adapter).map(|adapter| adapter.update(update_factory()))
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    pub fn update_view_focus_state(&self, is_focused: bool) -> Option<QueuedEvents> {
        self.is_view_focused.set(is_focused);
        Lazy::get(&self.associated.ivars().adapter)
            .map(|adapter| adapter.update_view_focus_state(is_focused))
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        let prev_class = self.associated.ivars().prev_class;
        let view = Id::as_ptr(&self.view) as *mut NSView;
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
