// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{
    declare::{ClassBuilder, Ivar, IvarDrop},
    declare_class,
    ffi::{
        objc_getAssociatedObject, objc_setAssociatedObject, object_setClass,
        OBJC_ASSOCIATION_ASSIGN,
    },
    foundation::{NSArray, NSObject, NSPoint},
    msg_send_id,
    rc::{Id, Owned, WeakId},
    runtime::{Class, Sel},
    sel, ClassType,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{collections::HashMap, ffi::c_void, ops::Deref};

use crate::{appkit::NSView, Adapter};

static SUBCLASSES: Lazy<Mutex<HashMap<&'static Class, &'static Class>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Declare as mutable to ensure the address is unique.
static mut ASSOCIATED_OBJECT_KEY: u8 = 0;

fn associated_object_key() -> *const c_void {
    unsafe { &ASSOCIATED_OBJECT_KEY as *const u8 as *const _ }
}

declare_class!(
    struct AssociatedObject {
        // Safety: These are set in AssociatedObject::new, immediately after
        // the object is created.
        adapter: IvarDrop<Box<Adapter>>,
        prev_class: &'static Class,
    }

    unsafe impl ClassType for AssociatedObject {
        type Super = NSObject;
        const NAME: &'static str = "AccessKitSubclassAssociatedObject";
    }
);

impl AssociatedObject {
    fn new(adapter: Adapter, prev_class: &'static Class) -> Id<Self, Owned> {
        unsafe {
            let mut object: Id<Self, Owned> = msg_send_id![Self::class(), new];
            Ivar::write(&mut object.adapter, Box::new(adapter));
            Ivar::write(&mut object.prev_class, prev_class);
            object
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
    associated.adapter.view_children()
}

unsafe extern "C" fn focus(this: &NSView, _cmd: Sel) -> *mut NSObject {
    let associated = associated_object(this);
    associated.adapter.focus()
}

unsafe extern "C" fn hit_test(this: &NSView, _cmd: Sel, point: NSPoint) -> *mut NSObject {
    let associated = associated_object(this);
    associated.adapter.hit_test(point)
}

/// Uses dynamic Objective-C subclassing to implement the NSView
/// accessibility methods when normal subclassing isn't an option.
pub struct SubclassingAdapter {
    view: WeakId<NSView>,
    associated: Id<AssociatedObject, Owned>,
}

impl SubclassingAdapter {
    /// Dynamically subclass the specified view to use the specified adapter.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    pub unsafe fn new(view: *mut c_void, adapter: Adapter) -> Self {
        let view = view as *mut NSView;
        // Cast to a pointer and back to force the lifetime to 'static
        // SAFETY: We know the class will live as long as the instance,
        // and we only use this reference while the instance is alive.
        let prev_class = unsafe { &*((*view).class() as *const Class) };
        let mut associated = AssociatedObject::new(adapter, prev_class);
        unsafe {
            objc_setAssociatedObject(
                view as *mut _,
                associated_object_key(),
                Id::as_mut_ptr(&mut associated) as *mut _,
                OBJC_ASSOCIATION_ASSIGN,
            )
        };
        let mut subclasses = SUBCLASSES.lock();
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
        unsafe { object_setClass(view as *mut _, (*subclass as *const Class).cast()) };
        let view = Id::retain(view).unwrap();
        let view = WeakId::new(&view);
        Self { view, associated }
    }

    pub fn inner(&self) -> &Adapter {
        &self.associated.adapter
    }
}

impl Deref for SubclassingAdapter {
    type Target = Adapter;

    fn deref(&self) -> &Adapter {
        self.inner()
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        if let Some(view) = self.view.load() {
            let prev_class = *self.associated.prev_class;
            let view = Id::as_ptr(&view) as *mut NSView;
            unsafe { object_setClass(view as *mut _, (prev_class as *const Class).cast()) };
            unsafe {
                objc_setAssociatedObject(
                    view as *mut _,
                    associated_object_key(),
                    std::ptr::null_mut(),
                    OBJC_ASSOCIATION_ASSIGN,
                )
            };
        }
    }
}
