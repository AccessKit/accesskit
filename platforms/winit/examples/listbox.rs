use accesskit::{
    Action, ActionRequest, ActivationHandler, DefaultActionVerb, Node, NodeBuilder, NodeId, Rect,
    Role, Tree, TreeUpdate,
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

const ITEMS: &[(NodeId, &str)] = &[
    (NodeId(2), "bevy"),
    (NodeId(3), "egui"),
    (NodeId(4), "Slint"),
];

const WINDOW_ID: NodeId = NodeId(0);
const LISTBOX_ID: NodeId = NodeId(1);
const INITIAL_FOCUS: NodeId = LISTBOX_ID;

const LISTBOX_X: f64 = 20.0;
const LISTBOX_Y: f64 = 20.0;
const ITEM_WIDTH: f64 = 500.0;
const ITEM_HEIGHT: f64 = 40.0;
const LISTBOX_RECT: Rect = Rect {
    x0: LISTBOX_X,
    y0: LISTBOX_Y,
    x1: LISTBOX_X + ITEM_WIDTH,
    y1: LISTBOX_Y + (ITEMS.len() as f64 * ITEM_HEIGHT),
};

fn build_option(index: usize, is_selected: bool) -> Node {
    let rect = Rect {
        x0: LISTBOX_RECT.x0,
        y0: LISTBOX_RECT.y0 + (index as f64 * ITEM_HEIGHT),
        x1: LISTBOX_RECT.x1,
        y1: LISTBOX_RECT.y0 + ((index + 1) as f64 * ITEM_HEIGHT),
    };

    let mut builder = NodeBuilder::new(Role::ListBoxOption);
    builder.set_bounds(rect);
    builder.set_name(ITEMS[index].1);
    builder.set_selected(is_selected);
    builder.set_position_in_set(index + 1);
    builder.set_size_of_set(ITEMS.len());
    builder.add_action(Action::Focus);
    builder.set_default_action_verb(DefaultActionVerb::Click);
    builder.build()
}

fn build_listbox(name: &str) -> Node {
    let mut builder = NodeBuilder::new(Role::ListBox);
    builder.set_bounds(LISTBOX_RECT);
    builder.set_children(ITEMS.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    builder.set_name(name);
    builder.build()
}

struct UiState {
    focus: NodeId,
    selected_item_index: Option<usize>,
}

impl UiState {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            selected_item_index: None,
        }))
    }

    fn build_root(&mut self) -> Node {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.push_child(LISTBOX_ID);

        builder.set_name(WINDOW_TITLE);
        builder.build()
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let listbox = build_listbox("Friends of AccessKit");
        let mut tree = Tree::new(WINDOW_ID);
        tree.app_name = Some("listbox".to_string());
        let mut result = TreeUpdate {
            nodes: vec![(WINDOW_ID, root), (LISTBOX_ID, listbox)],
            tree: Some(tree),
            focus: self.focus,
        };
        for i in 0..ITEMS.len() {
            let id = ITEMS[i].0;
            result.nodes.push((id, build_option(i, false)));
        }
        result
    }

    fn focus_option(&mut self, adapter: &mut Adapter, index: usize) {
        self.focus = ITEMS[index].0;
        let previously_selected_item_index = self.selected_item_index;
        self.selected_item_index = Some(index);
        adapter.update_if_active(|| {
            let mut nodes = vec![(ITEMS[index].0, build_option(index, true))];
            if let Some(previous_index) = previously_selected_item_index {
                nodes.push((ITEMS[previous_index].0, build_option(previous_index, false)));
            }
            TreeUpdate {
                nodes,
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
                Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                    let mut state = state.lock().unwrap();
                    let new_selected_item_index = match state.selected_item_index {
                        None => 0,
                        Some(index) if index + 1 < ITEMS.len() => index + 1,
                        Some(index) => index,
                    };
                    state.focus_option(adapter, new_selected_item_index);
                }
                Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                    let mut state = state.lock().unwrap();
                    let new_selected_item_index = match state.selected_item_index {
                        None => 0,
                        Some(index) if index > 0 => index - 1,
                        Some(index) => index,
                    };
                    state.focus_option(adapter, new_selected_item_index);
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
                if let Some(index) = ITEMS.iter().position(|(id, _)| *id == target) {
                    let mut state = state.lock().unwrap();
                    if let Action::Focus = action {
                        state.focus_option(adapter, index);
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
    let event_loop = EventLoop::with_user_event().build()?;
    let mut state = Application::new(event_loop.create_proxy());
    event_loop.run_app(&mut state).map_err(Into::into)
}
