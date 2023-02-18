use accesskit::{
    Action, ActionRequest, DefaultActionVerb, Live, Node, NodeBuilder, NodeClassSet, NodeId, Rect,
    Role, Tree, TreeUpdate,
};
use accesskit_winit::{ActionRequestEvent, Adapter};
use std::{
    num::NonZeroU128,
    sync::{Arc, Mutex},
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
const ANNOUNCEMENT_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
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

fn build_button(id: NodeId, name: &str, classes: &mut NodeClassSet) -> Node {
    let rect = match id {
        BUTTON_1_ID => BUTTON_1_RECT,
        BUTTON_2_ID => BUTTON_2_RECT,
        _ => unreachable!(),
    };

    let mut builder = NodeBuilder::new(Role::Button);
    builder.set_bounds(rect);
    builder.set_name(name);
    builder.add_action(Action::Focus);
    builder.set_default_action_verb(DefaultActionVerb::Click);
    builder.build(classes)
}

fn build_announcement(text: &str, classes: &mut NodeClassSet) -> Node {
    let mut builder = NodeBuilder::new(Role::StaticText);
    builder.set_name(text);
    builder.set_live(Live::Polite);
    builder.build(classes)
}

struct State {
    focus: NodeId,
    is_window_focused: bool,
    announcement: Option<String>,
    node_classes: NodeClassSet,
}

impl State {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            is_window_focused: false,
            announcement: None,
            node_classes: NodeClassSet::new(),
        }))
    }

    fn focus(&self) -> Option<NodeId> {
        self.is_window_focused.then_some(self.focus)
    }

    fn build_root(&mut self) -> Node {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        if self.announcement.is_some() {
            builder.push_child(ANNOUNCEMENT_ID);
        }
        builder.set_name(WINDOW_TITLE);
        builder.build(&mut self.node_classes)
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let button_1 = build_button(BUTTON_1_ID, "Button 1", &mut self.node_classes);
        let button_2 = build_button(BUTTON_2_ID, "Button 2", &mut self.node_classes);
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (BUTTON_1_ID, button_1),
                (BUTTON_2_ID, button_2),
            ],
            tree: Some(Tree::new(WINDOW_ID)),
            focus: self.focus(),
        };
        if let Some(announcement) = &self.announcement {
            result.nodes.push((
                ANNOUNCEMENT_ID,
                build_announcement(announcement, &mut self.node_classes),
            ));
        }
        result
    }

    fn update_focus(&mut self, adapter: &Adapter) {
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: self.focus(),
        });
    }

    fn press_button(&mut self, adapter: &Adapter, id: NodeId) {
        let text = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        self.announcement = Some(text.into());
        adapter.update_if_active(|| {
            let announcement = build_announcement(text, &mut self.node_classes);
            let root = self.build_root();
            TreeUpdate {
                nodes: vec![(ANNOUNCEMENT_ID, announcement), (WINDOW_ID, root)],
                tree: None,
                focus: self.focus(),
            }
        });
    }
}

fn main() {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!("- [Space] 'presses' the button, adding static text in a live region announcing that it was pressed.");
    #[cfg(target_os = "windows")]
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");
    #[cfg(all(
        feature = "accesskit_unix",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    println!("Enable Orca with [Super]+[Alt]+[S].");

    let event_loop = EventLoopBuilder::with_user_event().build();

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
            move || {
                let mut state = state.lock().unwrap();
                state.build_initial_tree()
            },
            event_loop.create_proxy(),
        )
    };

    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } if adapter.on_event(&window, &event) => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::ExitWithCode(0);
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
                        let mut state = state.lock().unwrap();
                        let id = state.focus;
                        state.press_button(&adapter, id);
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
