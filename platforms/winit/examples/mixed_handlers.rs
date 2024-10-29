use accesskit::{
    Action, ActionRequest, ActivationHandler, Live, Node, NodeId, Rect, Role, Tree, TreeUpdate,
};
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::Key,
    window::{Window, WindowId},
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

fn build_button(id: NodeId, label: &str) -> Node {
    let rect = match id {
        BUTTON_1_ID => BUTTON_1_RECT,
        BUTTON_2_ID => BUTTON_2_RECT,
        _ => unreachable!(),
    };

    let mut node = Node::new(Role::Button);
    node.set_bounds(rect);
    node.set_label(label);
    node.add_action(Action::Focus);
    node.add_action(Action::Click);
    node
}

fn build_announcement(text: &str) -> Node {
    let mut node = Node::new(Role::Label);
    node.set_value(text);
    node.set_live(Live::Polite);
    node
}

struct UiState {
    focus: NodeId,
    announcement: Option<String>,
}

impl UiState {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            announcement: None,
        }))
    }

    fn build_root(&mut self) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        if self.announcement.is_some() {
            node.push_child(ANNOUNCEMENT_ID);
        }
        node.set_label(WINDOW_TITLE);
        node
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let button_1 = build_button(BUTTON_1_ID, "Button 1");
        let button_2 = build_button(BUTTON_2_ID, "Button 2");
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
            result
                .nodes
                .push((ANNOUNCEMENT_ID, build_announcement(announcement)));
        }
        result
    }

    fn set_focus(&mut self, adapter: &mut Adapter, focus: NodeId) {
        self.focus = focus;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
        });
    }

    fn press_button(&mut self, adapter: &mut Adapter, id: NodeId) {
        let text = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        self.announcement = Some(text.into());
        adapter.update_if_active(|| {
            let announcement = build_announcement(text);
            let root = self.build_root();
            TreeUpdate {
                nodes: vec![(ANNOUNCEMENT_ID, announcement), (WINDOW_ID, root)],
                tree: None,
                focus: self.focus,
            }
        });
    }
}

struct TearoffActivationHandler {
    state: Arc<Mutex<UiState>>,
}

impl ActivationHandler for TearoffActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.state.lock().unwrap().build_initial_tree())
    }
}

struct WindowState {
    window: Window,
    adapter: Adapter,
    ui: Arc<Mutex<UiState>>,
}

impl WindowState {
    fn new(window: Window, adapter: Adapter, ui: Arc<Mutex<UiState>>) -> Self {
        Self {
            window,
            adapter,
            ui,
        }
    }
}

struct Application {
    event_loop_proxy: EventLoopProxy<AccessKitEvent>,
    window: Option<WindowState>,
}

impl Application {
    fn new(event_loop_proxy: EventLoopProxy<AccessKitEvent>) -> Self {
        Self {
            event_loop_proxy,
            window: None,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_visible(false);

        let window = event_loop.create_window(window_attributes)?;
        let ui = UiState::new();
        let activation_handler = TearoffActivationHandler {
            state: Arc::clone(&ui),
        };
        let adapter = Adapter::with_mixed_handlers(
            &window,
            activation_handler,
            self.event_loop_proxy.clone(),
        );
        window.set_visible(true);

        self.window = Some(WindowState::new(window, adapter, ui));
        Ok(())
    }
}

impl ApplicationHandler<AccessKitEvent> for Application {
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };
        let adapter = &mut window.adapter;
        let state = &mut window.ui;

        adapter.process_event(&window.window, &event);
        match event {
            WindowEvent::CloseRequested => {
                self.window = None;
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
                    state.set_focus(adapter, new_focus);
                }
                Key::Named(winit::keyboard::NamedKey::Space) => {
                    let mut state = state.lock().unwrap();
                    let id = state.focus;
                    state.press_button(adapter, id);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, user_event: AccessKitEvent) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };
        let adapter = &mut window.adapter;
        let state = &mut window.ui;

        match user_event.window_event {
            AccessKitWindowEvent::InitialTreeRequested => unreachable!(),
            AccessKitWindowEvent::ActionRequested(ActionRequest { action, target, .. }) => {
                if target == BUTTON_1_ID || target == BUTTON_2_ID {
                    let mut state = state.lock().unwrap();
                    match action {
                        Action::Focus => {
                            state.set_focus(adapter, target);
                        }
                        Action::Click => {
                            state.press_button(adapter, target);
                        }
                        _ => (),
                    }
                }
            }
            AccessKitWindowEvent::AccessibilityDeactivated => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create initial window");
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            event_loop.exit();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

    let event_loop = EventLoop::with_user_event().build()?;
    let mut state = Application::new(event_loop.create_proxy());
    event_loop.run_app(&mut state).map_err(Into::into)
}
