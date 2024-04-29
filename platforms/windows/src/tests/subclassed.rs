// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Node, NodeBuilder, NodeId, Role, Tree,
    TreeUpdate,
};
use windows::Win32::{Foundation::*, UI::Accessibility::*};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::windows::EventLoopBuilderExtWindows,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowId},
};

use super::MUTEX;
use crate::SubclassingAdapter;

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(0);
const BUTTON_1_ID: NodeId = NodeId(1);
const BUTTON_2_ID: NodeId = NodeId(2);

fn make_button(name: &str) -> Node {
    let mut builder = NodeBuilder::new(Role::Button);
    builder.set_name(name);
    builder.add_action(Action::Focus);
    builder.build()
}

fn get_initial_state() -> TreeUpdate {
    let root = {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        builder.set_name(WINDOW_TITLE);
        builder.build()
    };
    let button_1 = make_button("Button 1");
    let button_2 = make_button("Button 2");
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

struct SimpleActivationHandler;

impl ActivationHandler for SimpleActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(get_initial_state())
    }
}

// This module uses winit for the purpose of testing with a real third-party
// window implementation that we don't control.

struct TestApplication;

impl ApplicationHandler<()> for TestApplication {
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_visible(false);

        let window = event_loop.create_window(window_attributes).unwrap();
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get()),
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        };
        let adapter =
            SubclassingAdapter::new(hwnd, SimpleActivationHandler {}, NullActionHandler {});
        assert!(unsafe { UiaHasServerSideProvider(hwnd) }.as_bool());
        drop(window);
        drop(adapter);
        event_loop.exit();
    }
}

#[test]
fn has_native_uia() {
    // This test is simple enough that we know it's fine to run entirely
    // on one thread, so we don't need a full multithreaded test harness.
    // Still, we must prevent this test from running concurrently with other
    // tests, especially the focus test.
    let _lock_guard = MUTEX.lock().unwrap();
    let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();
    let mut state = TestApplication {};
    event_loop.run_app(&mut state).unwrap();
}
