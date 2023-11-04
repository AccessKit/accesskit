// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionHandler, ActionRequest, Node, NodeBuilder, NodeClassSet, NodeId, Role, Tree,
    TreeUpdate,
};
use windows::Win32::{Foundation::*, UI::Accessibility::*};
use winit::{
    event_loop::EventLoopBuilder,
    platform::windows::EventLoopBuilderExtWindows,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::WindowBuilder,
};

use super::MUTEX;
use crate::SubclassingAdapter;

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(0);
const BUTTON_1_ID: NodeId = NodeId(1);
const BUTTON_2_ID: NodeId = NodeId(2);

fn make_button(name: &str, classes: &mut NodeClassSet) -> Node {
    let mut builder = NodeBuilder::new(Role::Button);
    builder.set_name(name);
    builder.add_action(Action::Focus);
    builder.build(classes)
}

fn get_initial_state() -> TreeUpdate {
    let mut classes = NodeClassSet::new();
    let root = {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        builder.set_name(WINDOW_TITLE);
        builder.build(&mut classes)
    };
    let button_1 = make_button("Button 1", &mut classes);
    let button_2 = make_button("Button 2", &mut classes);
    TreeUpdate {
        nodes: vec![
            (WINDOW_ID, root),
            (BUTTON_1_ID, button_1),
            (BUTTON_2_ID, button_2),
        ],
        tree: Some(Tree::new(WINDOW_ID)),
        focus: BUTTON_1_ID,
    }
}

pub struct NullActionHandler;

impl ActionHandler for NullActionHandler {
    fn do_action(&mut self, _request: ActionRequest) {}
}

// This module uses winit for the purpose of testing with a real third-party
// window implementation that we don't control.

#[test]
fn has_native_uia() {
    // This test is simple enough that we know it's fine to run entirely
    // on one thread, so we don't need a full multithreaded test harness.
    // Still, we must prevent this test from running concurrently with other
    // tests, especially the focus test.
    let _lock_guard = MUTEX.lock().unwrap();
    let event_loop = EventLoopBuilder::<()>::new()
        .with_any_thread(true)
        .build()
        .unwrap();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();
    let hwnd = match window.window_handle().unwrap().as_raw() {
        RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get()),
        RawWindowHandle::WinRt(_) => unimplemented!(),
        _ => unreachable!(),
    };
    let adapter = SubclassingAdapter::new(hwnd, get_initial_state, Box::new(NullActionHandler {}));
    assert!(unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    drop(window);
    drop(adapter);
}
