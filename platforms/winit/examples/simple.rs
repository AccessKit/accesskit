use accesskit::kurbo::Rect;
use accesskit::{Action, ActionRequest, DefaultActionVerb, Node, NodeId, Role, Tree, TreeUpdate};
use accesskit_winit::{ActionRequestEvent, Adapter};
use std::{
    num::NonZeroU128,
    sync::{Arc, Mutex},
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

const BUTTON_1_RECT: Rect = Rect {
    x0: 20.0,
    y0: 20.0,
    x1: 100.0,
    y1: 60.0,
};

const BUTTON_2_RECT: Rect = Rect {
    x0: 20.0,
    y0: 60.0,
    x1: 100.0,
    y1: 100.0,
};

fn make_button(id: NodeId, name: &str) -> Node {
    let rect = match id {
        BUTTON_1_ID => BUTTON_1_RECT,
        BUTTON_2_ID => BUTTON_2_RECT,
        _ => unreachable!(),
    };

    Node {
        bounds: Some(rect),
        name: Some(name.into()),
        focusable: true,
        default_action_verb: Some(DefaultActionVerb::Click),
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
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: self.is_window_focused.then_some(self.focus),
        });
    }

    fn press_button(&self, adapter: &Adapter, id: NodeId) {
        // This is a pretty hacky way of updating a node.
        // A real GUI framework would have a consistent way
        // of building a node from underlying data.
        // Also, this update isn't as lazy as it could be;
        // we force the AccessKit tree to be initialized.
        // This is expedient in this case, because that tree
        // is the only place where the state of the buttons
        // is stored. It's not a problem because we're really
        // only concerned with testing lazy updates in the context
        // of focus changes.
        let name = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        let node = make_button(id, name);
        let update = TreeUpdate {
            nodes: vec![node],
            tree: None,
            focus: self.is_window_focused.then_some(self.focus),
        };
        adapter.update(update);
    }
}

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
        tree: Some(Tree::new(WINDOW_ID)),
        focus: state.is_window_focused.then_some(state.focus),
    }
}

fn main() {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!("- [Space] 'presses' the button, permanently renaming it.");
    #[cfg(target_os = "windows")]
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");

    let event_loop = EventLoop::with_user_event();

    let state = State::new();

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let adapter = {
        let state = Arc::clone(&state);
        Adapter::new(
            &window,
            Box::new(move || {
                let state = state.lock().unwrap();
                initial_tree_update(&state)
            }),
            event_loop.create_proxy(),
        )
    };

    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Focused(is_window_focused) => {
                    let mut state = state.lock().unwrap();
                    state.is_window_focused = is_window_focused;
                    state.update_focus(&adapter);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match virtual_code {
                    VirtualKeyCode::Tab => {
                        let mut state = state.lock().unwrap();
                        state.focus = if state.focus == BUTTON_1_ID {
                            BUTTON_2_ID
                        } else {
                            BUTTON_1_ID
                        };
                        state.update_focus(&adapter);
                    }
                    VirtualKeyCode::Space => {
                        let state = state.lock().unwrap();
                        state.press_button(&adapter, state.focus);
                    }
                    _ => (),
                },
                _ => (),
            },
            Event::UserEvent(ActionRequestEvent {
                request:
                    ActionRequest {
                        action,
                        target,
                        data: None,
                    },
                ..
            }) if target == BUTTON_1_ID || target == BUTTON_2_ID => {
                let mut state = state.lock().unwrap();
                match action {
                    Action::Focus => {
                        state.focus = target;
                        state.update_focus(&adapter);
                    }
                    Action::Default => {
                        state.press_button(&adapter, target);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    });
}
