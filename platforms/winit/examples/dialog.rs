use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Tree, TreeUpdate};
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use std::error::Error;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::Key,
    window::{Window, WindowId},
};

const WINDOW_TITLE: &str = "* New document";

const WINDOW_ID: NodeId = NodeId(0);
const DIALOG_ID: NodeId = NodeId(1);
const DIALOG_TITLE_ID: NodeId = NodeId(2);
const BUTTON_YES_ID: NodeId = NodeId(3);
const BUTTON_NO_ID: NodeId = NodeId(4);
const INITIAL_FOCUS: NodeId = WINDOW_ID;

const DIALOG_RECT: Rect = Rect {
    x0: 100.0,
    y0: 100.0,
    x1: 300.0,
    y1: 200.0,
};

const DIALOG_TITLE_RECT: Rect = Rect {
    x0: 110.0,
    y0: 110.0,
    x1: 290.0,
    y1: 140.0,
};

const BUTTON_YES_RECT: Rect = Rect {
    x0: 120.0,
    y0: 150.0,
    x1: 200.0,
    y1: 190.0,
};

const BUTTON_NO_RECT: Rect = Rect {
    x0: 210.0,
    y0: 150.0,
    x1: 290.0,
    y1: 190.0,
};

fn build_dialog(title: NodeId) -> Node {
    let mut node = Node::new(Role::AlertDialog);
    node.set_bounds(DIALOG_RECT);
    node.push_labelled_by(title);
    node.set_children([DIALOG_TITLE_ID, BUTTON_YES_ID, BUTTON_NO_ID]);
    node
}

fn build_dialog_title(text: &str) -> Node {
    let mut node = Node::new(Role::Label);
    node.set_bounds(DIALOG_TITLE_RECT);
    node.set_value(text);
    node
}

fn build_button(id: NodeId, label: &str) -> Node {
    let rect = match id {
        BUTTON_YES_ID => BUTTON_YES_RECT,
        BUTTON_NO_ID => BUTTON_NO_RECT,
        _ => unreachable!(),
    };

    let mut node = Node::new(Role::Button);
    node.set_bounds(rect);
    node.set_label(label);
    node.add_action(Action::Focus);
    node.add_action(Action::Click);
    node
}

struct UiState {
    focus: NodeId,
    is_dialog_open: bool,
}

impl UiState {
    fn new() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            is_dialog_open: false,
        }
    }

    fn build_root(&mut self) -> Node {
        let mut node = Node::new(Role::Window);
        if self.is_dialog_open {
            node.push_child(DIALOG_ID);
        }
        node.set_label(WINDOW_TITLE);
        node
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let tree = Tree::new(WINDOW_ID);
        TreeUpdate {
            nodes: vec![(WINDOW_ID, root)],
            tree: Some(tree),
            focus: self.focus,
        }
    }

    fn open_dialog(&mut self, adapter: &mut Adapter) {
        self.is_dialog_open = true;
        self.focus = BUTTON_NO_ID;
        adapter.update_if_active(|| {
            let root = self.build_root();
            let dialog = build_dialog(DIALOG_TITLE_ID);
            let title =
                build_dialog_title("You have unsaved changes. Are you sure you want to quit?");
            let yes = build_button(BUTTON_YES_ID, "Yes");
            let no = build_button(BUTTON_NO_ID, "No");
            TreeUpdate {
                nodes: vec![
                    (WINDOW_ID, root),
                    (DIALOG_ID, dialog),
                    (DIALOG_TITLE_ID, title),
                    (BUTTON_YES_ID, yes),
                    (BUTTON_NO_ID, no),
                ],
                tree: None,
                focus: self.focus,
            }
        });
    }

    fn close_dialog(&mut self, adapter: &mut Adapter) {
        self.is_dialog_open = false;
        self.focus = INITIAL_FOCUS;
        adapter.update_if_active(|| {
            let root = self.build_root();
            TreeUpdate {
                nodes: vec![(WINDOW_ID, root)],
                tree: None,
                focus: self.focus,
            }
        });
    }

    fn set_focus(&mut self, adapter: &mut Adapter, focus: NodeId) {
        self.focus = focus;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
        });
    }
}

struct WindowState {
    window: Window,
    adapter: Adapter,
    ui: UiState,
}

impl WindowState {
    fn new(window: Window, adapter: Adapter, ui: UiState) -> Self {
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
            .with_visible(false)
            .with_inner_size(LogicalSize::new(400, 300));

        let window = event_loop.create_window(window_attributes)?;
        let adapter =
            Adapter::with_event_loop_proxy(event_loop, &window, self.event_loop_proxy.clone());
        window.set_visible(true);

        self.window = Some(WindowState::new(window, adapter, UiState::new()));
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
                state.open_dialog(adapter);
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
                Key::Named(winit::keyboard::NamedKey::Tab) if state.is_dialog_open => {
                    let new_focus = if state.focus == BUTTON_YES_ID {
                        BUTTON_NO_ID
                    } else {
                        BUTTON_YES_ID
                    };
                    state.set_focus(adapter, new_focus);
                }
                Key::Named(winit::keyboard::NamedKey::Space) if state.is_dialog_open => {
                    if state.focus == BUTTON_YES_ID {
                        self.window = None;
                    } else if state.focus == BUTTON_NO_ID {
                        state.close_dialog(adapter);
                    }
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
            AccessKitWindowEvent::InitialTreeRequested => {
                adapter.update_if_active(|| state.build_initial_tree());
            }
            AccessKitWindowEvent::ActionRequested(ActionRequest { action, target, .. }) => {
                if !state.is_dialog_open {
                    return;
                }
                match action {
                    Action::Focus => {
                        if target == BUTTON_YES_ID || target == BUTTON_NO_ID {
                            state.set_focus(adapter, target);
                        }
                    }
                    Action::Click => {
                        if state.focus == BUTTON_YES_ID {
                            self.window = None;
                        } else if state.focus == BUTTON_NO_ID {
                            state.close_dialog(adapter);
                        }
                    }
                    _ => (),
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
    println!("- Closing the window opens a dialog.");
    println!("- [Tab] switches focus between Yes and No buttons.");
    println!("- [Space] 'presses' the button, either closing the app or the dialog.");
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
