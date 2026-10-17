// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// This test doesn't use the default harness, because the adapter
// must be created on the main thread.

use std::ffi::c_void;

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};
use accesskit_macos::SubclassingAdapter;
use objc2::rc::Id;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send, msg_send_id};
use objc2_app_kit::NSView;
use objc2_foundation::{NSArray, NSObject, NSString};

struct NoTree;

impl ActivationHandler for NoTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        None
    }
}

struct NoActions;

impl ActionHandler for NoActions {
    fn do_action(&mut self, _request: ActionRequest) {}
}

fn new_view() -> Id<NSView> {
    unsafe { msg_send_id![class!(NSView), new] }
}

fn adapter_for(view: &NSView) -> SubclassingAdapter {
    unsafe { SubclassingAdapter::new(view as *const NSView as *mut c_void, NoTree, NoActions) }
}

fn class_name(view: &NSView) -> String {
    let object: &AnyObject = view;
    object.class().name().to_owned()
}

fn an_unobserved_view_gets_its_class_back() {
    let view = new_view();
    let adapter = adapter_for(&view);
    assert_eq!(class_name(&view), "AccessKitSubclassOfNSView");
    drop(adapter);
    assert_eq!(class_name(&view), "NSView");
    drop(adapter_for(&view));
    assert_eq!(class_name(&view), "NSView");
}

// AppKit observes the first responder and can remove its observers
// after the adapter is dropped.
fn an_observed_view_keeps_its_observers() {
    let view = new_view();
    let adapter = adapter_for(&view);
    let observer = NSObject::new();
    let key_path = NSString::from_str("nextResponder");
    let no_context: *mut c_void = std::ptr::null_mut();
    let _: () = unsafe {
        msg_send![&view, addObserver: &*observer, forKeyPath: &*key_path, options: 0usize, context: no_context]
    };
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfNSView"
    );

    drop(adapter);
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfNSView"
    );
    let _children: *mut NSArray<NSObject> = unsafe { msg_send![&view, accessibilityChildren] };
    let _: () = unsafe { msg_send![&view, removeObserver: &*observer, forKeyPath: &*key_path] };
    assert_eq!(class_name(&view), "AccessKitSubclassOfNSView");
}

fn main() {
    an_unobserved_view_gets_its_class_back();
    an_observed_view_keeps_its_observers();
}
