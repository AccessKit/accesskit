// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{
    cell::Cell,
    ffi::c_void,
    panic::{self, AssertUnwindSafe},
    rc::Rc,
};

use accesskit_ios::SubclassingAdapter;
use objc2::declare::ClassBuilder;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Bool, Sel};
use objc2::{class, msg_send, msg_send_id, sel};
use objc2_foundation::{CGPoint, NSArray, NSObject, NSString};
use objc2_ui_kit::UIView;

use crate::harness::{Test, tests};
use crate::util::*;

pub const TESTS: &[Test] = tests![
    the_view_is_subclassed,
    querying_the_view_requests_the_tree,
    dropping_restores_the_view_class,
    dropping_releases_the_handlers,
    a_view_can_get_a_new_adapter,
    observers_can_be_removed_after_dropping,
    a_view_left_subclassed_behaves_like_its_original_class,
    an_observed_view_can_get_a_new_adapter,
    a_view_observed_before_the_adapter_is_rejected,
];

fn adapter_for(view: &UIView) -> (SubclassingAdapter, Rc<HandlerEvents>) {
    let events = Rc::new(HandlerEvents::default());
    let adapter = unsafe {
        SubclassingAdapter::new(
            view as *const UIView as *mut c_void,
            NoTree(Rc::clone(&events)),
            NoActions,
            NoDeactivation,
        )
    };
    (adapter, events)
}

fn query_accessibility(view: &UIView) {
    let _: Bool = unsafe { msg_send![view, isAccessibilityElement] };
    let _: *mut NSArray<NSObject> = unsafe { msg_send![view, accessibilityElements] };
    // UIKit doesn't implement this method on plain views.
    let responds: Bool =
        unsafe { msg_send![view, respondsToSelector: sel!(accessibilityHitTest:)] };
    if responds.as_bool() {
        let _: *mut AnyObject =
            unsafe { msg_send![view, accessibilityHitTest: CGPoint::new(0., 0.)] };
    }
    let superclass: &AnyClass = unsafe { msg_send![view, superclass] };
    assert_eq!(superclass.name(), "UIResponder");
}

fn query_accessibility_without_adapter(view: &UIView) {
    query_accessibility(view);
    let _: () = unsafe { msg_send![view, didMoveToWindow] };
}

struct Observation {
    observer: Retained<NSObject>,
    key_path: Retained<NSString>,
}

// Apps observe their views' geometry this way.
fn observe(view: &UIView) -> Observation {
    let observer = NSObject::new();
    let key_path = NSString::from_str("frame");
    let no_context: *mut c_void = std::ptr::null_mut();
    let _: () = unsafe {
        msg_send![view, addObserver: &*observer, forKeyPath: &*key_path, options: 0usize, context: no_context]
    };
    Observation { observer, key_path }
}

fn stop_observing(view: &UIView, observation: Observation) {
    let _: () = unsafe {
        msg_send![view, removeObserver: &*observation.observer, forKeyPath: &*observation.key_path]
    };
}

thread_local! {
    static MOVED_TO_WINDOW: Cell<bool> = const { Cell::new(false) };
}

extern "C" fn record_did_move_to_window(_this: &UIView, _cmd: Sel) {
    MOVED_TO_WINDOW.set(true);
}

fn new_recording_view() -> Retained<UIView> {
    let class = AnyClass::get("AccessKitTestRecordingView").unwrap_or_else(|| {
        let mut builder = ClassBuilder::new("AccessKitTestRecordingView", class!(UIView)).unwrap();
        unsafe {
            builder.add_method(
                sel!(didMoveToWindow),
                record_did_move_to_window as extern "C" fn(_, _),
            )
        };
        builder.register()
    });
    unsafe { msg_send_id![class, new] }
}

fn the_view_is_subclassed() {
    let view = new_view();
    let (_adapter, _) = adapter_for(&view);
    assert_eq!(class_name(&view), "AccessKitSubclassOfUIView");
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
    assert_eq!(class_name(&view), "UIView");
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
    assert_eq!(class_name(&view), "AccessKitSubclassOfUIView");
    query_accessibility(&view);
    assert!(events.tree_requested.get());
    drop(adapter);
    assert_eq!(class_name(&view), "UIView");
}

fn observers_can_be_removed_after_dropping() {
    let view = new_view();
    let (adapter, events) = adapter_for(&view);
    let observation = observe(&view);
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfUIView"
    );

    drop(adapter);
    assert!(events.dropped.get());
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfUIView"
    );
    query_accessibility_without_adapter(&view);

    stop_observing(&view, observation);
    assert_eq!(class_name(&view), "AccessKitSubclassOfUIView");
    query_accessibility_without_adapter(&view);
}

fn a_view_left_subclassed_behaves_like_its_original_class() {
    let view = new_recording_view();
    let (adapter, _) = adapter_for(&view);
    let observation = observe(&view);
    drop(adapter);
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfAccessKitTestRecordingView"
    );

    let is_element: Bool = unsafe { msg_send![&view, isAccessibilityElement] };
    assert!(!is_element.as_bool());
    let elements: *mut NSArray<NSObject> = unsafe { msg_send![&view, accessibilityElements] };
    assert!(elements.is_null());
    MOVED_TO_WINDOW.set(false);
    let _: () = unsafe { msg_send![&view, didMoveToWindow] };
    assert!(MOVED_TO_WINDOW.get());

    stop_observing(&view, observation);
}

fn an_observed_view_can_get_a_new_adapter() {
    let view = new_view();
    let (adapter, _) = adapter_for(&view);
    let observation = observe(&view);
    drop(adapter);

    let (adapter, events) = adapter_for(&view);
    assert_eq!(
        class_name(&view),
        "NSKVONotifying_AccessKitSubclassOfUIView"
    );
    query_accessibility(&view);
    assert!(events.tree_requested.get());
    drop(adapter);
    assert!(events.dropped.get());

    stop_observing(&view, observation);
    let (adapter, events) = adapter_for(&view);
    assert_eq!(class_name(&view), "AccessKitSubclassOfUIView");
    query_accessibility(&view);
    assert!(events.tree_requested.get());
    drop(adapter);
    assert_eq!(class_name(&view), "UIView");
}

fn a_view_observed_before_the_adapter_is_rejected() {
    let view = new_view();
    let observation = observe(&view);
    let result = panic::catch_unwind(AssertUnwindSafe(|| adapter_for(&view)));
    assert!(result.is_err());
    assert_eq!(class_name(&view), "NSKVONotifying_UIView");

    stop_observing(&view, observation);
    assert_eq!(class_name(&view), "UIView");
    query_accessibility_without_adapter(&view);
}
