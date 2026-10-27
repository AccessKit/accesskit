// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{
    ffi::c_void,
    panic::{self, AssertUnwindSafe},
    rc::Rc,
};

use accesskit_macos::SubclassingAdapter;
use objc2::msg_send;
use objc2::rc::Id;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_app_kit::NSView;
use objc2_foundation::{NSArray, NSObject, NSPoint, NSString};

use crate::harness::{Test, tests};
use crate::util::*;

pub const TESTS: &[Test] = tests![
    the_view_is_subclassed,
    querying_the_view_requests_the_tree,
    dropping_restores_the_view_class,
    dropping_releases_the_handlers,
    a_view_can_get_a_new_adapter,
    observers_can_be_removed_after_dropping,
    an_observed_view_can_get_a_new_adapter,
    a_view_observed_before_the_adapter_is_rejected,
];

fn adapter_for(view: &NSView) -> (SubclassingAdapter, Rc<HandlerEvents>) {
    let events = Rc::new(HandlerEvents::default());
    let adapter = unsafe {
        SubclassingAdapter::new(
            view as *const NSView as *mut c_void,
            NoTree(Rc::clone(&events)),
            NoActions,
        )
    };
    (adapter, events)
}

fn query_accessibility(view: &NSView) {
    let _: *mut NSArray<NSObject> = unsafe { msg_send![view, accessibilityChildren] };
    let _: *mut AnyObject = unsafe { msg_send![view, accessibilityFocusedUIElement] };
    let superclass: &AnyClass = unsafe { msg_send![view, superclass] };
    assert_eq!(superclass.name(), "NSResponder");
}

// The adapter can only hit test a view that is in a window, but the
// fallback doesn't need one.
fn query_accessibility_without_adapter(view: &NSView) {
    query_accessibility(view);
    let _: *mut AnyObject = unsafe { msg_send![view, accessibilityHitTest: NSPoint::new(0., 0.)] };
}

struct Observation {
    observer: Id<NSObject>,
    key_path: Id<NSString>,
}

// AppKit observes the first responder this way.
fn observe(view: &NSView) -> Observation {
    let observer = NSObject::new();
    let key_path = NSString::from_str("nextResponder");
    let no_context: *mut c_void = std::ptr::null_mut();
    let _: () = unsafe {
        msg_send![view, addObserver: &*observer, forKeyPath: &*key_path, options: 0usize, context: no_context]
    };
    Observation { observer, key_path }
}

fn stop_observing(view: &NSView, observation: Observation) {
    let _: () = unsafe {
        msg_send![view, removeObserver: &*observation.observer, forKeyPath: &*observation.key_path]
    };
}

fn the_view_is_subclassed() {
    let view = new_view();
    let (_adapter, _) = adapter_for(&view);
    assert_eq!(class_name(&view), "AccessKitSubclassOfNSView");
}

fn querying_the_view_requests_the_tree() {
    let view = new_view();
    let (_adapter, events) = adapter_for(&view);
    assert!(!events.tree_requested.get());
    query_accessibility(&view);
    assert!(events.tree_requested.get());
}

fn dropping_restores_the_view_class() {
    let view = new_view();
    let (adapter, _) = adapter_for(&view);
    drop(adapter);
    assert_eq!(class_name(&view), "NSView");
    query_accessibility_without_adapter(&view);
}

fn dropping_releases_the_handlers() {
    let view = new_view();
    let (adapter, events) = adapter_for(&view);
    drop(adapter);
    assert!(events.dropped.get());
}

fn a_view_can_get_a_new_adapter() {
    let view = new_view();
    drop(adapter_for(&view));
    let (adapter, events) = adapter_for(&view);
    assert_eq!(class_name(&view), "AccessKitSubclassOfNSView");
    query_accessibility(&view);
    assert!(events.tree_requested.get());
    drop(adapter);
    assert_eq!(class_name(&view), "NSView");
}

fn observers_can_be_removed_after_dropping() {
    let view = new_view();
    let (adapter, events) = adapter_for(&view);
    let observation = observe(&view);
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfNSView"
    );

    drop(adapter);
    assert!(events.dropped.get());
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfNSView"
    );
    query_accessibility_without_adapter(&view);

    stop_observing(&view, observation);
    assert_eq!(class_name(&view), "AccessKitSubclassOfNSView");
    query_accessibility_without_adapter(&view);
}

fn an_observed_view_can_get_a_new_adapter() {
    let view = new_view();
    let (adapter, _) = adapter_for(&view);
    let observation = observe(&view);
    drop(adapter);

    let (adapter, events) = adapter_for(&view);
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfNSView"
    );
    query_accessibility(&view);
    assert!(events.tree_requested.get());
    drop(adapter);
    assert!(events.dropped.get());

    stop_observing(&view, observation);
    let (adapter, events) = adapter_for(&view);
    assert_eq!(class_name(&view), "AccessKitSubclassOfNSView");
    query_accessibility(&view);
    assert!(events.tree_requested.get());
    drop(adapter);
    assert_eq!(class_name(&view), "NSView");
}

fn a_view_observed_before_the_adapter_is_rejected() {
    let view = new_view();
    let observation = observe(&view);
    let result = panic::catch_unwind(AssertUnwindSafe(|| adapter_for(&view)));
    assert!(result.is_err());
    assert_eq!(class_name(&view), "NSKVONotifying_NSView");

    stop_observing(&view, observation);
    assert_eq!(class_name(&view), "NSView");
    query_accessibility_without_adapter(&view);
}
