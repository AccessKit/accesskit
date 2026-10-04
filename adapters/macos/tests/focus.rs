// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate,
};
use accesskit_macos::Adapter;
use objc2::{
    msg_send,
    rc::{Id, autoreleasepool},
};
use objc2_app_kit::{NSApplication, NSBackingStoreType, NSView, NSWindow, NSWindowStyleMask};
use objc2_foundation::{MainThreadMarker, NSObject, NSRect};

const ROOT: NodeId = NodeId(0);
const FIRST: NodeId = NodeId(1);
const SECOND: NodeId = NodeId(2);
const LIST: NodeId = NodeId(3);
const OPTION: NodeId = NodeId(4);

struct NoActions;
impl ActionHandler for NoActions {
    fn do_action(&mut self, _: ActionRequest) {
        panic!("unexpected accessibility action");
    }
}

struct InitialTree;
impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let mut root = Node::new(Role::Window);
        root.set_children([FIRST, SECOND, LIST]);
        let mut list = Node::new(Role::ListBox);
        list.set_children([OPTION]);
        list.set_active_descendant(OPTION);
        Some(TreeUpdate {
            nodes: vec![
                (ROOT, root),
                (FIRST, Node::new(Role::TextInput)),
                (SECOND, Node::new(Role::TextInput)),
                (LIST, list),
                (OPTION, Node::new(Role::ListBoxOption)),
            ],
            tree: Some(TreeInfo::new(ROOT)),
            tree_id: TreeId::ROOT,
            focus: FIRST,
        })
    }
}

fn focused(node: &NSObject) -> bool {
    unsafe { msg_send![node, isAccessibilityFocused] }
}

fn set_focus(adapter: &mut Adapter, focus: NodeId) {
    adapter
        .update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus,
        })
        .unwrap()
        .raise();
}

fn assert_focus(adapter: &mut Adapter, expected: Option<&NSObject>) {
    let actual = adapter.focus(&mut InitialTree);
    assert_eq!(
        actual.cast_const(),
        expected.map_or(std::ptr::null(), |node| node as *const _)
    );
}

// AppKit requires the real main thread, so this test uses a custom harness.
// The views are never shown and cannot change the user's active window.
fn main() {
    autoreleasepool(|_| {
        let mtm = MainThreadMarker::new().expect("native test must run on the main thread");
        let _app = NSApplication::sharedApplication(mtm);
        let make_window = || unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc::<NSWindow>(),
                NSRect::ZERO,
                NSWindowStyleMask::Titled,
                NSBackingStoreType::NSBackingStoreBuffered,
                true,
            )
        };
        let first_window = make_window();
        let second_window = make_window();
        let first_view = unsafe { NSView::new(mtm) };
        let second_view = unsafe { NSView::new(mtm) };
        first_window.setContentView(Some(&first_view));
        second_window.setContentView(Some(&second_view));
        let mut first =
            unsafe { Adapter::new(Id::as_ptr(&first_view).cast_mut().cast(), false, NoActions) };
        let mut second =
            unsafe { Adapter::new(Id::as_ptr(&second_view).cast_mut().cast(), true, NoActions) };
        let roots = unsafe { Id::retain(first.view_children(&mut InitialTree)) }.unwrap();
        let root = unsafe { roots.objectAtIndex(0) };
        let children: Id<objc2_foundation::NSArray<NSObject>> =
            unsafe { objc2::msg_send_id![&root, accessibilityChildren] };
        let field1 = unsafe { children.objectAtIndex(0) };
        let field2 = unsafe { children.objectAtIndex(1) };
        let list = unsafe { children.objectAtIndex(2) };
        let options: Id<objc2_foundation::NSArray<NSObject>> =
            unsafe { objc2::msg_send_id![&list, accessibilityChildren] };
        let option = unsafe { options.objectAtIndex(0) };
        let other_focus = unsafe { Id::retain(second.focus(&mut InitialTree)) }.unwrap();

        assert!(
            focused(&field1),
            "inactive view must expose its first responder"
        );
        assert!(!focused(&field2));
        assert_focus(&mut first, Some(&field1));
        assert!(focused(&other_focus));

        println!("background focus change");
        set_focus(&mut first, SECOND);
        assert!(!focused(&field1));
        assert!(focused(&field2));
        assert_focus(&mut first, Some(&field2));
        assert_focus(&mut second, Some(&other_focus));
        assert!(
            focused(&other_focus),
            "background updates must not steal another view's focus"
        );

        for active in [true, false] {
            println!("host focus: {active}");
            first.update_view_focus_state(active).unwrap().raise();
            assert_focus(&mut first, Some(&field2));
            assert!(focused(&field2));
            set_focus(&mut first, LIST);
            assert!(
                !focused(&list),
                "active descendant owns accessibility focus"
            );
            assert!(focused(&option));
            assert_focus(&mut first, Some(&option));

            println!("clear active descendant: host focused={active}");
            first
                .update_if_active(|| {
                    let mut list = Node::new(Role::ListBox);
                    list.set_children([OPTION]);
                    TreeUpdate {
                        nodes: vec![(LIST, list)],
                        tree: None,
                        tree_id: TreeId::ROOT,
                        focus: LIST,
                    }
                })
                .unwrap()
                .raise();
            assert!(focused(&list));
            assert!(!focused(&option));
            assert_focus(&mut first, Some(&list));
            first
                .update_if_active(|| {
                    let mut list = Node::new(Role::ListBox);
                    list.set_children([OPTION]);
                    list.set_active_descendant(OPTION);
                    TreeUpdate {
                        nodes: vec![(LIST, list)],
                        tree: None,
                        tree_id: TreeId::ROOT,
                        focus: LIST,
                    }
                })
                .unwrap()
                .raise();
            assert!(!focused(&list));
            assert!(focused(&option));
            assert_focus(&mut first, Some(&option));
            set_focus(&mut first, SECOND);
        }

        set_focus(&mut first, ROOT);
        assert!(
            !focused(&root),
            "the synthetic window group must not acquire focus"
        );
        assert!(!focused(&field2));
        assert_focus(&mut first, None);
        println!(
            "focus: inactive/active views, independent hosts, active descendants, and window root passed"
        );
    });
}
