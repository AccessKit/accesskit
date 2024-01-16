use accesskit::{
    Action, ActionRequest, DefaultActionVerb, Live, Node, NodeBuilder, NodeClassSet, NodeId, Rect,
    Role, Tree, TreeUpdate,
};
use accesskit_winit::{ActionRequestEvent, Adapter};
use std::sync::{Arc, Mutex};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    keyboard::Key,
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(0);
const BUTTON_1_ID: NodeId = NodeId(1);
const BUTTON_2_ID: NodeId = NodeId(2);
const ANNOUNCEMENT_ID: NodeId = NodeId(3);
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
    announcement: Option<String>,
    node_classes: NodeClassSet,
}

impl State {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            announcement: None,
            node_classes: NodeClassSet::new(),
        }))
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
        let mut tree = Tree::new(WINDOW_ID);
        tree.app_name = Some("simple".to_string());
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (BUTTON_1_ID, button_1),
                (BUTTON_2_ID, button_2),
            ],
            tree: Some(tree),
            focus: self.focus,
        };
        if let Some(announcement) = &self.announcement {
            result.nodes.push((
                ANNOUNCEMENT_ID,
                build_announcement(announcement, &mut self.node_classes),
            ));
        }
        result
    }

    fn set_focus(&mut self, adapter: &Adapter, focus: NodeId) {
        self.focus = focus;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
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
                focus: self.focus,
            }
        });
    }
}

fn main() -> Result<(), impl std::error::Error> {
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

    let event_loop = EventLoopBuilder::with_user_event().build()?;

    let state = State::new();

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_visible(false)
        .build(&event_loop)?;

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

    event_loop.run(move |event, event_loop| {
        event_loop.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, .. } => {
                adapter.process_event(&window, &event);
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: virtual_code,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match virtual_code {
                        Key::Named(winit::keyboard::NamedKey::Tab) => {
                            let mut state = state.lock().unwrap();
                            let new_focus = if state.focus == BUTTON_1_ID {
                                BUTTON_2_ID
                            } else {
                                BUTTON_1_ID
                            };
                            state.set_focus(&adapter, new_focus);
                        }
                        Key::Named(winit::keyboard::NamedKey::Space) => {
                            let mut state = state.lock().unwrap();
                            let id = state.focus;
                            state.press_button(&adapter, id);
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
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
                        state.set_focus(&adapter, target);
                    }
                    Action::Default => {
                        state.press_button(&adapter, target);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    })
}
