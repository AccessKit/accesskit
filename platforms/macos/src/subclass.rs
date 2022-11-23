// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{
    declare::ClassBuilder,
    ffi::object_setClass,
    foundation::{NSArray, NSObject, NSPoint},
    rc::{Id, Shared},
    runtime::{Class, Sel},
    sel,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{collections::HashMap, ffi::c_void, ops::Deref};

use crate::{appkit::NSView, Adapter};

static SUBCLASSES: Lazy<Mutex<HashMap<&'static Class, &'static Class>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct Instance {
    view: Id<NSView, Shared>,
    adapter: Adapter,
    prev_class: &'static Class,
}

#[derive(PartialEq, Eq, Hash)]
struct ViewKey(*const NSView);
unsafe impl Send for ViewKey {}

struct InstancePtr(*const Instance);
unsafe impl Send for InstancePtr {}

static INSTANCES: Lazy<Mutex<HashMap<ViewKey, InstancePtr>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Some view classes, like the one in winit 0.27, assume that they are the
// lowest subclass, and call [self superclass] to get their superclass.
// Give them the answer they need.
unsafe extern "C" fn superclass(this: &NSView, _cmd: Sel) -> Option<&Class> {
    let key = ViewKey(this as *const _);
    let instances = INSTANCES.lock();
    let instance = instances.get(&key).unwrap();
    (*instance.0).prev_class.superclass()
}

unsafe extern "C" fn children(this: &NSView, _cmd: Sel) -> *mut NSArray<NSObject> {
    let key = ViewKey(this as *const _);
    let instances = INSTANCES.lock();
    let instance = instances.get(&key).unwrap();
    (*instance.0).adapter.view_children()
}

unsafe extern "C" fn focus(this: &NSView, _cmd: Sel) -> *mut NSObject {
    let key = ViewKey(this as *const _);
    let instances = INSTANCES.lock();
    let instance = instances.get(&key).unwrap();
    (*instance.0).adapter.focus()
}

unsafe extern "C" fn hit_test(this: &NSView, _cmd: Sel, point: NSPoint) -> *mut NSObject {
    let key = ViewKey(this as *const _);
    let instances = INSTANCES.lock();
    let instance = instances.get(&key).unwrap();
    (*instance.0).adapter.hit_test(point)
}

impl Instance {
    fn new(view: Id<NSView, Shared>, adapter: Adapter) -> Box<Self> {
        // Cast to a pointer and back to force the lifetime to 'static
        // SAFETY: We know the class will live as long as the instance,
        // and we own a reference to the instance.
        let prev_class = unsafe { &*(view.class() as *const Class) };
        Box::new(Self {
            view,
            adapter,
            prev_class,
        })
    }

    fn install(&mut self) {
        let view_ptr = Id::as_ptr(&self.view);
        let key = ViewKey(view_ptr);
        INSTANCES.lock().insert(key, InstancePtr(self as *const _));
        let mut subclasses = SUBCLASSES.lock();
        let entry = subclasses.entry(self.prev_class);
        let subclass = entry.or_insert_with(|| {
            let name = format!("AccessKitSubclassOf{}", self.prev_class.name());
            let mut builder = ClassBuilder::new(&name, self.prev_class).unwrap();
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
        unsafe { object_setClass(view_ptr as *mut _, (*subclass as *const Class).cast()) };
    }

    fn uninstall(&self) {
        let view_ptr = Id::as_ptr(&self.view);
        unsafe { object_setClass(view_ptr as *mut _, (self.prev_class as *const Class).cast()) };
        let key = ViewKey(view_ptr);
        INSTANCES.lock().remove(&key);
    }
}

/// Uses dynamic Objective-C subclassing to implement the NSView
/// accessibility methods when normal subclassing isn't an option.
#[repr(transparent)]
pub struct SubclassingAdapter(Box<Instance>);

impl SubclassingAdapter {
    /// Dynamically subclass the specified view to use the specified adapter.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    /// This method will retain an additional reference to `view`.
    pub unsafe fn new(view: *mut c_void, adapter: Adapter) -> Self {
        let view = Id::retain(view as *mut NSView).unwrap();
        let mut instance = Instance::new(view, adapter);
        instance.install();
        Self(instance)
    }

    pub fn inner(&self) -> &Adapter {
        &self.0.adapter
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
        self.0.uninstall();
    }
}
