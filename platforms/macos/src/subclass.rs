// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{
    declare::ClassBuilder,
    foundation::{NSArray, NSObject},
    rc::Id,
    runtime::{Class, Sel},
    sel,
};
use objc_sys::object_setClass;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{collections::HashMap, ops::Deref};

use crate::{appkit::NSView, Adapter};

static SUBCLASSES: Lazy<Mutex<HashMap<&'static Class, &'static Class>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct Instance {
    adapter: Adapter,
    prev_class: Option<&'static Class>,
}

#[derive(PartialEq, Eq, Hash)]
struct ViewKey(*const NSView);
unsafe impl Send for ViewKey {}
unsafe impl Sync for ViewKey {}

struct InstancePtr(*const Instance);
unsafe impl Send for InstancePtr {}
unsafe impl Sync for InstancePtr {}

static INSTANCES: Lazy<Mutex<HashMap<ViewKey, InstancePtr>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Some view classes, like the one in winit 0.27, assume that they are the
// lowest subclass, and call [self superclass] to get their superclass.
// Give them the answer they need.
unsafe extern "C" fn get_superclass(this: &NSView, _cmd: Sel) -> Option<&Class> {
    let key = ViewKey(this as *const _);
    let instances = INSTANCES.lock();
    let instance = instances.get(&key).unwrap();
    (*instance.0).prev_class.as_ref().unwrap().superclass()
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

impl Instance {
    fn new(adapter: Adapter) -> Box<Self> {
        Box::new(Self {
            adapter,
            prev_class: None,
        })
    }

    fn install(&mut self) {
        let view_ptr = Id::as_ptr(&self.adapter.context.view);
        let key = ViewKey(view_ptr);
        INSTANCES.lock().insert(key, InstancePtr(self as *const _));
        // Cast to a pointer and back to force the lifetime to 'static
        // SAFETY: We know the class will live as long as the instance,
        // and we own a reference to the instance.
        let superclass = unsafe { &*(self.adapter.context.view.class() as *const Class) };
        self.prev_class = Some(superclass);
        let mut subclasses = SUBCLASSES.lock();
        let entry = subclasses.entry(superclass);
        let subclass = entry.or_insert_with(|| {
            let name = format!("AccessKitSubclassOf{}", superclass.name());
            let mut builder = ClassBuilder::new(&name, superclass).unwrap();
            unsafe {
                builder.add_method(
                    sel!(superclass),
                    get_superclass as unsafe extern "C" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(accessibilityChildren),
                    children as unsafe extern "C" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(accessibilityFocusedUIElement),
                    focus as unsafe extern "C" fn(_, _) -> _,
                );
            }
            builder.register()
        });
        unsafe { object_setClass(view_ptr as *mut _, (*subclass as *const Class).cast()) };
    }

    fn uninstall(&self) {
        let view_ptr = Id::as_ptr(&self.adapter.context.view);
        unsafe {
            object_setClass(
                view_ptr as *mut _,
                (self.prev_class.unwrap() as *const Class).cast(),
            )
        };
        let key = ViewKey(view_ptr);
        INSTANCES.lock().remove(&key);
    }
}

/// Uses dynamic Objective-C subclassing to implement the NSView
/// accessibility methods when normal subclassing isn't an option.
#[repr(transparent)]
pub struct SubclassingAdapter(Option<Box<Instance>>);
unsafe impl Send for SubclassingAdapter {}
unsafe impl Sync for SubclassingAdapter {}

impl SubclassingAdapter {
    pub fn new(adapter: Adapter) -> Self {
        let mut instance = Instance::new(adapter);
        instance.install();
        Self(Some(instance))
    }

    pub fn inner(&self) -> &Adapter {
        &self.0.as_ref().unwrap().adapter
    }

    pub fn into_inner(mut self) -> Adapter {
        let instance = self.0.take().unwrap();
        instance.uninstall();
        instance.adapter
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
        if let Some(instance) = self.0.as_ref() {
            instance.uninstall();
        }
    }
}
