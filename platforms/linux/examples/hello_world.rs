// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use accesskit_linux::Adapter;
use std::num::NonZeroU64;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });

fn get_tree() -> Tree {
    Tree {
        ..Tree::new(TreeId("test".into()), WINDOW_ID, StringEncoding::Utf8)
    }
}

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
        clear: None,
        nodes: vec![root, button_1, button_2],
        tree: Some(get_tree()),
        focus: None,
    }
}

static mut FOCUS: NodeId = BUTTON_1_ID;

fn main() {
    let adapter = Adapter::new(String::from("hello_world"), String::from("ExampleUI"), String::from("0.1.0"), get_initial_state()).unwrap();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Focused(true) => {
                        adapter.window_activated(WINDOW_ID);
                        adapter.update(TreeUpdate {
                            clear: None,
                            nodes: vec![],
                            focus: Some(unsafe { FOCUS }),
                            tree: None
                        });
                    },
                    WindowEvent::Focused(false) => {
                        adapter.window_deactivated(WINDOW_ID);
                    },
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Tab),
                            state: ElementState::Pressed,
                            .. },
                        .. } => {
                            unsafe {
                                FOCUS = if FOCUS == BUTTON_1_ID {
                                    BUTTON_2_ID
                                } else {
                                    BUTTON_1_ID
                                };
                                adapter.update(TreeUpdate {
                                    clear: None,
                                    nodes: vec![],
                                    focus: Some(FOCUS),
                                    tree: None
                                });
                            }
                    },
                    _ => (),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
