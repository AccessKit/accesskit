// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

//! `accessibilityWindow` and `accessibilityTopLevelUIElement` must return the
//! view's window even when the view is not the window's content view.
//!
//! AppKit objects can only be created on the process main thread, which the
//! default test harness does not provide, so this test uses `harness = false`.

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TreeId, TreeInfo,
    TreeUpdate,
};
use accesskit_macos::Adapter;
use objc2::{msg_send, rc::Id, runtime::AnyObject};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSScrollView, NSView, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSPoint, NSRect, NSSize};
use std::{ffi::c_void, ptr};

struct NoActions;

impl ActionHandler for NoActions {
    fn do_action(&mut self, _request: ActionRequest) {}
}

struct InitialTree;

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let mut root = Node::new(Role::Group);
        root.set_children(vec![NodeId(2)]);
        Some(TreeUpdate {
            nodes: vec![(NodeId(1), root), (NodeId(2), Node::new(Role::Button))],
            tree: Some(TreeInfo::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        })
    }
}

fn ownership(node: &NSObject) -> [*mut AnyObject; 2] {
    unsafe {
        [
            msg_send![node, accessibilityWindow],
            msg_send![node, accessibilityTopLevelUIElement],
        ]
    }
}

fn assert_owned_by(label: &str, nodes: &[&NSObject], window: Option<&NSWindow>) {
    let expected = window.map_or(ptr::null_mut(), |window| {
        window as *const NSWindow as *mut AnyObject
    });
    for node in nodes {
        assert_eq!(ownership(node), [expected; 2], "{label}");
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("window ownership test must run on the main thread");
    objc2::rc::autoreleasepool(|_| unsafe {
        let _app = NSApplication::sharedApplication(mtm);
        let frame = NSRect::new(NSPoint::new(0., 0.), NSSize::new(320., 240.));
        let make_window = || {
            let window = NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc::<NSWindow>(),
                frame,
                NSWindowStyleMask::Titled,
                NSBackingStoreType::NSBackingStoreBuffered,
                true,
            );
            window.setReleasedWhenClosed(false);
            window
        };
        let first = make_window();
        let second = make_window();
        let view = NSView::initWithFrame(mtm.alloc::<NSView>(), frame);
        let mut adapter = Adapter::new(&*view as *const NSView as *mut c_void, false, NoActions);
        let roots = Id::retain(adapter.view_children(&mut InitialTree)).unwrap();
        assert_eq!(roots.len(), 1);
        let root = roots.objectAtIndex(0);
        let children: *mut NSArray<NSObject> = msg_send![&*root, accessibilityChildren];
        let children = Id::retain(children).unwrap();
        assert_eq!(children.len(), 1);
        let child = children.objectAtIndex(0);
        let nodes = [&*root, &*child];

        first.setContentView(Some(&view));
        assert_owned_by("content view", &nodes, Some(&first));

        // Nest the view the way scroll and split views do. Its accessibility
        // parent is now the document view, not the window.
        let scroll = NSScrollView::initWithFrame(mtm.alloc::<NSScrollView>(), frame);
        let document = NSView::initWithFrame(mtm.alloc::<NSView>(), frame);
        let _: () = msg_send![&*document, setAccessibilityElement: true];
        first.setContentView(Some(&scroll));
        scroll.setDocumentView(Some(&document));
        document.addSubview(&view);
        let parent: *mut AnyObject = msg_send![&*view, accessibilityParent];
        assert_ne!(parent, &*first as *const NSWindow as *mut AnyObject);
        assert_owned_by("nested in a scroll view", &nodes, Some(&first));

        view.removeFromSuperview();
        assert_owned_by("detached", &nodes, None);

        second.contentView().unwrap().addSubview(&view);
        assert_owned_by("moved to another window", &nodes, Some(&second));

        view.removeFromSuperview();
        first.close();
        second.close();
        drop(adapter);
    });
    println!("window ownership: ok");
}
