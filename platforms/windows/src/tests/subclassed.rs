// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::num::NonZeroU128;

use accesskit::{
    ActionHandler, ActionRequest, Node, NodeId, Role, StringEncoding, Tree, TreeUpdate,
};
use windows::Win32::{Foundation::*, UI::Accessibility::*};
use winit::{
    event_loop::EventLoop,
    platform::windows::{EventLoopExtWindows, WindowExtWindows},
    window::WindowBuilder,
};

use crate::{Adapter, WindowSubclass};

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });

fn make_button(id: NodeId, name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(id, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: vec![BUTTON_1_ID, BUTTON_2_ID],
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = make_button(BUTTON_1_ID, "Button 1");
    let button_2 = make_button(BUTTON_2_ID, "Button 2");
    TreeUpdate {
        nodes: vec![root, button_1, button_2],
        tree: Some(Tree::new(WINDOW_ID, StringEncoding::Utf8)),
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
    let event_loop = EventLoop::<()>::new_any_thread();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .unwrap();
    let hwnd = HWND(window.hwnd() as _);
    assert!(!unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    let adapter = Adapter::new(hwnd, get_initial_state(), Box::new(NullActionHandler {}));
    let subclass = WindowSubclass::new(&adapter);
    assert!(unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    drop(subclass);
    assert!(!unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    let subclass = WindowSubclass::new(&adapter);
    assert!(unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
    drop(window);
    drop(subclass);
}
