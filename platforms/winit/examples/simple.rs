#[path = "util/fill.rs"]
mod fill;

use accesskit::{Action, ActionRequest, Live, Node, NodeId, Rect, Role, Tree, TreeUpdate};
use accesskit_winit::{WindowEvent as AccessKitWindowEvent, WinitAdapter};
use std::{
    error::Error,
    sync::mpsc::{channel, Receiver, Sender},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::Key,
    window::{Window, WindowAttributes, WindowId},
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
    fn new() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            announcement: None,
        }
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
        let tree = Tree::new(WINDOW_ID);
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

    fn set_focus(&mut self, adapter: &mut WinitAdapter, focus: NodeId) {
        self.focus = focus;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
        });
    }

    fn press_button(&mut self, adapter: &mut WinitAdapter, id: NodeId) {
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

struct WindowState {
    window: Box<dyn Window>,
    adapter: WinitAdapter,
    ui: UiState,
}

impl WindowState {
    fn new(window: Box<dyn Window>, adapter: WinitAdapter, ui: UiState) -> Self {
        Self {
            window,
            adapter,
            ui,
        }
    }
}

struct Application {
    window: Option<WindowState>,
    event_recv: Receiver<AccessKitWindowEvent>,
    event_sender: Sender<AccessKitWindowEvent>,
}

impl Application {
    fn new() -> Self {
        let (event_sender, event_recv) = channel();
        Self {
            window: None,
            event_sender,
            event_recv,
        }
    }

    fn create_window(&mut self, event_loop: &dyn ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let window_attributes = WindowAttributes::default()
            .with_title(WINDOW_TITLE)
            .with_visible(false);

        let window = event_loop.create_window(window_attributes)?;
        let proxy = event_loop.create_proxy();
        let sender = self.event_sender.clone();
        let adapter = WinitAdapter::new(event_loop, &*window, move |_window_id, evt| {
            sender.send(evt).unwrap();
            proxy.wake_up();
        });
        window.set_visible(true);

        self.window = Some(WindowState::new(window, adapter, UiState::new()));
        Ok(())
    }
}

impl ApplicationHandler for Application {
    fn window_event(&mut self, _: &dyn ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };
        let adapter = &mut window.adapter;
        let state = &mut window.ui;

        adapter.process_event(&*window.window, &event);
        match event {
            WindowEvent::CloseRequested => {
                fill::cleanup_window(&*window.window);
                self.window = None;
            }
            WindowEvent::SurfaceResized(_) => {
                window.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                fill::fill_window(&*window.window);
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
                    let new_focus = if state.focus == BUTTON_1_ID {
                        BUTTON_2_ID
                    } else {
                        BUTTON_1_ID
                    };
                    state.set_focus(adapter, new_focus);
                    window.window.request_redraw();
                }
                Key::Character(c) if c == " " => {
                    let id = state.focus;
                    state.press_button(adapter, id);
                    window.window.request_redraw();
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn proxy_wake_up(&mut self, _: &dyn ActiveEventLoop) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };
        let adapter = &mut window.adapter;
        let state = &mut window.ui;

        for accesskit_event in self.event_recv.try_iter() {
            match accesskit_event {
                AccessKitWindowEvent::InitialTreeRequested => {
                    adapter.update_if_active(|| state.build_initial_tree());
                }
                AccessKitWindowEvent::ActionRequested(ActionRequest { action, target, .. }) => {
                    if target == BUTTON_1_ID || target == BUTTON_2_ID {
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
                    window.window.request_redraw();
                }
                AccessKitWindowEvent::AccessibilityDeactivated => (),
            }
        }
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create initial window");
        if let Some(window) = self.window.as_ref() {
            window.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
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

    let event_loop = EventLoop::new()?;
    let app = Application::new();
    event_loop.run_app(app).map_err(Into::into)
}
