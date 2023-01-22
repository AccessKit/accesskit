// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::num::NonZeroU128;

use accesskit::{
    Action, ActionHandler, ActionRequest, Node, NodeBuilder, NodeClassSet, NodeId, Role, Tree,
    TreeUpdate,
};
use windows::Win32::{Foundation::*, UI::Accessibility::*};
use winit::{
    event_loop::EventLoopBuilder,
    platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows},
    window::WindowBuilder,
};

use crate::SubclassingAdapter;

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });

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
        focus: None,
    }
}

pub struct NullActionHandler;

impl ActionHandler for NullActionHandler {
    fn do_action(&self, _request: ActionRequest) {}
}

// This module uses winit for the purpose of testing with a real third-party
// window implementation that we don't control.

#[test]
fn has_native_uia() {
    // This test is simple enough that we know it's fine to run entirely
    // on one thread, so we don't need a full multithreaded test harness.
    let event_loop = EventLoopBuilder::<()>::new().with_any_thread(true).build();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .unwrap();
    let hwnd = HWND(window.hwnd());
    assert!(!unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    let adapter = SubclassingAdapter::new(hwnd, get_initial_state, Box::new(NullActionHandler {}));
    assert!(unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    drop(adapter);
    assert!(!unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    let adapter = SubclassingAdapter::new(hwnd, get_initial_state, Box::new(NullActionHandler {}));
    assert!(unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    drop(window);
    drop(adapter);
}
