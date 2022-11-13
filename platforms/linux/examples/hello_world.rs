// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    ActionHandler, ActionRequest, DefaultActionVerb, Node, NodeId, Role, Tree, TreeUpdate,
};
use accesskit_linux::Adapter;
use std::{num::NonZeroU128, sync::Arc};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });

struct NullActionHandler;

impl ActionHandler for NullActionHandler {
    fn do_action(&self, _request: ActionRequest) {}
}

fn get_tree() -> Tree {
    Tree {
        ..Tree::new(WINDOW_ID)
    }
}

fn make_button(name: &str) -> Arc<Node> {
    Arc::new(Node {
        role: Role::Button,
        name: Some(name.into()),
        focusable: true,
        default_action_verb: Some(DefaultActionVerb::Click),
        ..Default::default()
    })
}

fn get_initial_state() -> TreeUpdate {
    let root = Arc::new(Node {
        role: Role::Window,
        children: vec![BUTTON_1_ID, BUTTON_2_ID],
        name: Some(WINDOW_TITLE.into()),
        ..Default::default()
    });
    let button_1 = make_button("Button 1");
    let button_2 = make_button("Button 2");
    TreeUpdate {
        nodes: vec![
            (WINDOW_ID, root),
            (BUTTON_1_ID, button_1),
            (BUTTON_2_ID, button_2),
        ],
        tree: Some(get_tree()),
        focus: None,
    }
}

static mut FOCUS: NodeId = BUTTON_1_ID;

fn main() {
    let adapter = Adapter::new(
        String::from("hello_world"),
        String::from("ExampleUI"),
        String::from("0.1.0"),
        get_initial_state(),
        Box::new(NullActionHandler {}),
    )
    .unwrap();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Focused(window_has_focus) => {
                    adapter
                        .update(TreeUpdate {
                            nodes: vec![],
                            focus: window_has_focus.then(|| unsafe { FOCUS }),
                            tree: None,
                        })
                        .raise();
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Tab),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => unsafe {
                    FOCUS = if FOCUS == BUTTON_1_ID {
                        BUTTON_2_ID
                    } else {
                        BUTTON_1_ID
                    };
                    adapter
                        .update(TreeUpdate {
                            nodes: vec![],
                            focus: Some(FOCUS),
                            tree: None,
                        })
                        .raise();
                },
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => unsafe {
                    let updated_node = if FOCUS == BUTTON_1_ID {
                        (BUTTON_1_ID, make_button("You pressed button 1"))
                    } else {
                        (BUTTON_2_ID, make_button("You pressed button 2"))
                    };
                    adapter
                        .update(TreeUpdate {
                            nodes: vec![updated_node],
                            focus: Some(FOCUS),
                            tree: None,
                        })
                        .raise();
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
