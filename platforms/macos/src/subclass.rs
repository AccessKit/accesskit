// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use objc2::{
    declare::{ClassBuilder, Ivar, IvarDrop},
    declare_class,
    ffi::{
        objc_getAssociatedObject, objc_setAssociatedObject, object_setClass,
        OBJC_ASSOCIATION_RETAIN_NONATOMIC,
    },
    foundation::{NSArray, NSObject, NSPoint},
    msg_send_id,
    rc::{Id, Owned, Shared},
    runtime::{Class, Sel},
    sel, ClassType,
};
use once_cell::{sync::Lazy as SyncLazy, unsync::Lazy};
use std::{collections::HashMap, ffi::c_void, sync::Mutex};

use crate::{
    appkit::{NSView, NSWindow},
    event::QueuedEvents,
    Adapter,
};

static SUBCLASSES: SyncLazy<Mutex<HashMap<&'static Class, &'static Class>>> =
    SyncLazy::new(|| Mutex::new(HashMap::new()));

// Declare as mutable to ensure the address is unique.
static mut ASSOCIATED_OBJECT_KEY: u8 = 0;

fn associated_object_key() -> *const c_void {
    unsafe { &ASSOCIATED_OBJECT_KEY as *const u8 as *const _ }
}

type LazyAdapter = Lazy<Adapter, Box<dyn FnOnce() -> Adapter>>;

declare_class!(
    struct AssociatedObject {
        // SAFETY: These are set in AssociatedObject::new, immediately after
        // the object is created.
        adapter: IvarDrop<Box<LazyAdapter>>,
        prev_class: &'static Class,
    }

    unsafe impl ClassType for AssociatedObject {
        type Super = NSObject;
        const NAME: &'static str = "AccessKitSubclassAssociatedObject";
    }
);

impl AssociatedObject {
    fn new(adapter: LazyAdapter, prev_class: &'static Class) -> Id<Self, Shared> {
        unsafe {
            let mut object: Id<Self, Owned> = msg_send_id![Self::class(), new];
            Ivar::write(&mut object.adapter, Box::new(adapter));
            Ivar::write(&mut object.prev_class, prev_class);
            object.into()
        }
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
unsafe extern "C" fn superclass(this: &NSView, _cmd: Sel) -> Option<&Class> {
    let associated = associated_object(this);
    associated.prev_class.superclass()
}

unsafe extern "C" fn children(this: &NSView, _cmd: Sel) -> *mut NSArray<NSObject> {
    let associated = associated_object(this);
    let adapter = Lazy::force(&associated.adapter);
    adapter.view_children()
}

unsafe extern "C" fn focus(this: &NSView, _cmd: Sel) -> *mut NSObject {
    let associated = associated_object(this);
    let adapter = Lazy::force(&associated.adapter);
    adapter.focus()
}

unsafe extern "C" fn hit_test(this: &NSView, _cmd: Sel, point: NSPoint) -> *mut NSObject {
    let associated = associated_object(this);
    let adapter = Lazy::force(&associated.adapter);
    adapter.hit_test(point)
}

/// Uses dynamic Objective-C subclassing to implement the NSView
/// accessibility methods when normal subclassing isn't an option.
pub struct SubclassingAdapter {
    view: Id<NSView, Shared>,
    associated: Id<AssociatedObject, Shared>,
}

impl SubclassingAdapter {
    /// Create an adapter that dynamically subclasses the specified view.
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
        retained_view: Id<NSView, Shared>,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let adapter: LazyAdapter = {
            let retained_view = retained_view.clone();
            Lazy::new(Box::new(move || {
                let view = Id::as_ptr(&retained_view) as *mut c_void;
                unsafe { Adapter::new(view, source(), action_handler) }
            }))
        };
        let view = Id::as_ptr(&retained_view) as *mut NSView;
        // Cast to a pointer and back to force the lifetime to 'static
        // SAFETY: We know the class will live as long as the instance,
        // and we only use this reference while the instance is alive.
        let prev_class = unsafe { &*((*view).class() as *const Class) };
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
        unsafe { object_setClass(view as *mut _, (*subclass as *const Class).cast()) };
        Self {
            view: retained_view,
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
        let retained_view = window.content_view().unwrap();
        Self::new_internal(retained_view, source, action_handler)
    }

    /// Initialize the tree if it hasn't been initialized already, then apply
    /// the provided update.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let adapter = Lazy::force(&self.associated.adapter);
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
        Lazy::get(&self.associated.adapter).map(|adapter| adapter.update(update_factory()))
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        let prev_class = *self.associated.prev_class;
        let view = Id::as_ptr(&self.view) as *mut NSView;
        unsafe { object_setClass(view as *mut _, (prev_class as *const Class).cast()) };
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
