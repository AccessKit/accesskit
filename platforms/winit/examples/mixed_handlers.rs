#[path = "util/fill.rs"]
mod fill;

use accesskit::{
    Action, ActionRequest, ActivationHandler, Live, Node, NodeId, Rect, Role, Tree, TreeId,
    TreeUpdate,
};
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
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

const WINDOW_RECT: Rect = Rect {
    x0: 0.0,
    y0: 0.0,
    x1: 393.0,
    y1: 759.0,
};

const BUTTON_1_RECT: Rect = Rect {
    x0: 20.0,
    y0: 20.0,
    x1: 200.0,
    y1: 64.0,
};

const BUTTON_2_RECT: Rect = Rect {
    x0: 20.0,
    y0: 84.0,
    x1: 200.0,
    y1: 128.0,
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

const ANNOUNCEMENT_DELAY: Duration = Duration::from_millis(150);

struct UiState {
    focus: NodeId,
    announcement: Option<String>,
    pending_announcement: Option<(String, Instant)>,
}

impl UiState {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            announcement: None,
            pending_announcement: None,
        }))
    }

    fn build_root(&mut self) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_bounds(WINDOW_RECT);
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
        let tree = Tree::new(WINDOW_ID);
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (BUTTON_1_ID, button_1),
                (BUTTON_2_ID, button_2),
            ],
            tree: Some(tree),
            tree_id: TreeId::ROOT,
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
            tree_id: TreeId::ROOT,
            focus,
        });
    }

    fn press_button(&mut self, id: NodeId) {
        let text = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        // On iOS, VoiceOver announces the label of the activated button.
        // Postpone the live region update so the messages don't overlap.
        self.pending_announcement = Some((text.into(), Instant::now()));
    }

    fn flush_announcement(&mut self, adapter: &mut Adapter) -> bool {
        let Some((_, queued_at)) = &self.pending_announcement else {
            return false;
        };
        if queued_at.elapsed() < ANNOUNCEMENT_DELAY {
            return true;
        }
        if let Some((text, _)) = self.pending_announcement.take() {
            self.announcement = Some(text.clone());
            adapter.update_if_active(|| {
                let announcement = build_announcement(&text);
                let root = self.build_root();
                TreeUpdate {
                    nodes: vec![(ANNOUNCEMENT_ID, announcement), (WINDOW_ID, root)],
                    tree: None,
                    tree_id: TreeId::ROOT,
                    focus: self.focus,
                }
            });
        }
        false
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
            event_loop,
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
                fill::cleanup_window(&window.window);
                self.window = None;
            }
            WindowEvent::Resized(_) => {
                window.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                fill::fill_window(&window.window);
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
                    window.window.request_redraw();
                }
                Key::Named(winit::keyboard::NamedKey::Space) => {
                    let mut state = state.lock().unwrap();
                    let id = state.focus;
                    state.press_button(id);
                    window.window.request_redraw();
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
            AccessKitWindowEvent::ActionRequested(ActionRequest {
                action,
                target_node,
                ..
            }) => {
                if target_node == BUTTON_1_ID || target_node == BUTTON_2_ID {
                    let mut state = state.lock().unwrap();
                    match action {
                        Action::Focus => {
                            state.set_focus(adapter, target_node);
                        }
                        Action::Click => {
                            state.press_button(target_node);
                        }
                        _ => (),
                    }
                }
                window.window.request_redraw();
            }
            AccessKitWindowEvent::AccessibilityDeactivated => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.create_window(event_loop)
                .expect("failed to create initial window");
        }
        if let Some(window) = self.window.as_ref() {
            window.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &mut self.window {
            if window
                .ui
                .lock()
                .unwrap()
                .flush_announcement(&mut window.adapter)
            {
                event_loop.set_control_flow(ControlFlow::wait_duration(ANNOUNCEMENT_DELAY));
            }
        } else {
            event_loop.exit();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!(
        "- [Space] 'presses' the button, adding static text in a live region announcing that it was pressed."
    );
    #[cfg(target_os = "windows")]
    println!(
        "Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows)."
    );
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
