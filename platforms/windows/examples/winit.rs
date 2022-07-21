use accesskit::{
    Action, ActionHandler, ActionRequest, Node, NodeId, Role, StringEncoding, Tree, TreeUpdate,
};
use accesskit_windows::{Adapter, WindowSubclass};
use std::{
    num::NonZeroU128,
    sync::{Arc, Mutex},
};
use windows::Win32::Foundation::HWND;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

fn make_button(id: NodeId, name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(id, Role::Button)
    }
}

#[derive(Debug)]
struct State {
    focus: NodeId,
    is_window_focused: bool,
}

impl State {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            is_window_focused: false,
        }))
    }

    fn update_focus(&mut self, adapter: &Adapter) {
        adapter
            .update_if_active(|| TreeUpdate {
                nodes: vec![],
                tree: None,
                focus: self.is_window_focused.then_some(self.focus),
            })
            .raise();
    }
}

#[derive(Debug)]
struct MyAccessKitFactory(Arc<Mutex<State>>);

fn initial_tree_update(state: &State) -> TreeUpdate {
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
        focus: state.is_window_focused.then_some(state.focus),
    }
}

pub struct WinitActionHandler(Mutex<EventLoopProxy<ActionRequest>>);

impl ActionHandler for WinitActionHandler {
    fn do_action(&self, request: ActionRequest) {
        let proxy = self.0.lock().unwrap();
        proxy.send_event(request).unwrap();
    }
}

fn main() {
    let event_loop = EventLoop::with_user_event();

    let state = State::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let adapter = {
        let state = Arc::clone(&state);
        let proxy = Mutex::new(event_loop.create_proxy());
        Arc::new(Adapter::new(
            HWND(window.hwnd() as _),
            Box::new(move || {
                let state = state.lock().unwrap();
                initial_tree_update(&state)
            }),
            Box::new(WinitActionHandler(proxy)),
        ))
    };
    let _subclass = WindowSubclass::new(&*adapter);

    window.set_visible(true);

    let adapter = Arc::clone(&adapter); // to move into the event handler
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Focused(is_window_focused) => {
                        let mut state = state.lock().unwrap();
                        state.is_window_focused = is_window_focused;
                        state.update_focus(&*adapter);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        match virtual_code {
                            VirtualKeyCode::Tab => {
                                let mut state = state.lock().unwrap();
                                state.focus = if state.focus == BUTTON_1_ID {
                                    BUTTON_2_ID
                                } else {
                                    BUTTON_1_ID
                                };
                                state.update_focus(&*adapter);
                            }
                            VirtualKeyCode::Space => {
                                // This is a pretty hacky way of updating a node.
                                // A real GUI framework would have a consistent
                                // way of building a node from underlying data.
                                let focus = state.lock().unwrap().focus;
                                let node = if focus == BUTTON_1_ID {
                                    make_button(BUTTON_1_ID, "You pressed button 1")
                                } else {
                                    make_button(BUTTON_2_ID, "You pressed button 2")
                                };
                                let update = TreeUpdate {
                                    nodes: vec![node],
                                    tree: None,
                                    focus: Some(focus),
                                };
                                adapter.update(update).raise();
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            Event::UserEvent(ActionRequest {
                action: Action::Focus,
                target,
                data: None,
            }) if target == BUTTON_1_ID || target == BUTTON_2_ID => {
                let mut state = state.lock().unwrap();
                state.focus = target;
                state.update_focus(&*adapter);
            }
            _ => (),
        }
    });
}
