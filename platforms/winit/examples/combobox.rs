use accesskit::{Action, ActionRequest, Node, NodeId, Orientation, Rect, Role, Tree, TreeUpdate};
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

const WINDOW_TITLE: &str = "ComboBox example";

const WINDOW_ID: NodeId = NodeId(0);
const COMBOBOX_ID: NodeId = NodeId(1);
const LISTBOX_ID: NodeId = NodeId(2);
const INITIAL_FOCUS: NodeId = COMBOBOX_ID;

const LANGUAGES: &[(&str, NodeId)] = &[
    ("English", NodeId(3)),
    ("Esperanto", NodeId(4)),
    ("French", NodeId(5)),
    ("Spanish", NodeId(6)),
];

const ITEM_HEIGHT: f64 = 40.0;

const COMBOBOX_RECT: Rect = Rect {
    x0: 100.0,
    y0: 20.0,
    x1: 300.0,
    y1: 20.0 + ITEM_HEIGHT,
};

const LISTBOX_RECT: Rect = Rect {
    x0: COMBOBOX_RECT.x0,
    y0: COMBOBOX_RECT.y0,
    x1: COMBOBOX_RECT.x1,
    y1: COMBOBOX_RECT.y0 + ITEM_HEIGHT * (LANGUAGES.len() as f64),
};

fn build_combobox(expanded: bool, selected_index: usize) -> Node {
    let mut node = Node::new(Role::ComboBox);
    node.set_bounds(COMBOBOX_RECT);
    node.set_controls([LISTBOX_ID]);
    node.set_expanded(expanded);
    node.set_label("Select your language");
    node.set_value(LANGUAGES[selected_index].0);
    node.add_action(Action::Click);
    node.add_action(Action::Focus);
    node.add_action(if expanded {
        Action::Collapse
    } else {
        Action::Expand
    });
    node
}

fn build_listbox(hidden: bool) -> Node {
    let mut node = Node::new(Role::ListBox);
    node.set_bounds(LISTBOX_RECT);
    node.set_children(
        LANGUAGES
            .iter()
            .map(|(_, id)| id)
            .copied()
            .collect::<Vec<NodeId>>(),
    );
    if hidden {
        node.set_hidden();
    }
    node.set_orientation(Orientation::Vertical);
    node.set_size_of_set(LANGUAGES.len());
    node
}

fn build_option(index: usize, selected: bool) -> Node {
    let mut node = Node::new(Role::ListBoxOption);
    node.set_bounds(Rect {
        x0: LISTBOX_RECT.x0,
        y0: LISTBOX_RECT.y0 + (index as f64) * ITEM_HEIGHT,
        x1: LISTBOX_RECT.x1,
        y1: LISTBOX_RECT.y0 + (index as f64) * ITEM_HEIGHT + ITEM_HEIGHT,
    });
    node.set_label(LANGUAGES[index].0);
    node.set_position_in_set(index);
    node.set_selected(selected);
    node.add_action(Action::Click);
    node.add_action(Action::Focus);
    node
}

struct UiState {
    focus: NodeId,
    is_expanded: bool,
    combobox_index: usize,
    listbox_index: usize,
}

impl UiState {
    fn new() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            is_expanded: false,
            combobox_index: 0,
            listbox_index: 0,
        }
    }

    fn build_root(&mut self) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children([COMBOBOX_ID, LISTBOX_ID]);
        node.set_label(WINDOW_TITLE);
        node
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let combobox = build_combobox(self.is_expanded, self.combobox_index);
        let listbox = build_listbox(!self.is_expanded);
        let tree = Tree::new(WINDOW_ID);
        let mut update = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (COMBOBOX_ID, combobox),
                (LISTBOX_ID, listbox),
            ],
            tree: Some(tree),
            focus: self.focus,
        };
        for (i, (_, id)) in LANGUAGES.iter().enumerate() {
            update
                .nodes
                .push((*id, build_option(i, i == self.listbox_index)));
        }
        update
    }

    fn set_focus(&mut self, adapter: &mut Adapter, focus: NodeId) {
        self.focus = focus;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
        });
    }

    fn expand(&mut self, adapter: &mut Adapter) {
        self.is_expanded = true;
        self.focus = LANGUAGES[self.combobox_index].1;
        self.listbox_index = self.combobox_index;
        adapter.update_if_active(|| {
            let root = self.build_root();
            let combobox = build_combobox(self.is_expanded, self.combobox_index);
            let listbox = build_listbox(!self.is_expanded);
            let mut update = TreeUpdate {
                nodes: vec![
                    (WINDOW_ID, root),
                    (COMBOBOX_ID, combobox),
                    (LISTBOX_ID, listbox),
                ],
                tree: None,
                focus: self.focus,
            };
            for (i, (_, id)) in LANGUAGES.iter().enumerate() {
                update
                    .nodes
                    .push((*id, build_option(i, i == self.listbox_index)));
            }
            update
        });
    }

    fn collapse(&mut self, adapter: &mut Adapter) {
        self.is_expanded = false;
        self.focus = COMBOBOX_ID;
        adapter.update_if_active(|| {
            let root = self.build_root();
            let combobox = build_combobox(self.is_expanded, self.combobox_index);
            let listbox = build_listbox(!self.is_expanded);
            TreeUpdate {
                nodes: vec![
                    (WINDOW_ID, root),
                    (COMBOBOX_ID, combobox),
                    (LISTBOX_ID, listbox),
                ],
                tree: None,
                focus: self.focus,
            }
        });
    }

    fn update_combobox_value(&mut self, adapter: &mut Adapter, index: usize) {
        self.combobox_index = index;
        adapter.update_if_active(|| {
            let combobox = build_combobox(self.is_expanded, self.combobox_index);
            TreeUpdate {
                nodes: vec![(COMBOBOX_ID, combobox)],
                tree: None,
                focus: self.focus,
            }
        })
    }

    fn update_listbox_selection(&mut self, adapter: &mut Adapter, index: usize) {
        self.focus = LANGUAGES[index].1;
        let previous_index = self.listbox_index;
        self.listbox_index = index;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![
                (
                    LANGUAGES[previous_index].1,
                    build_option(previous_index, false),
                ),
                (LANGUAGES[index].1, build_option(index, true)),
            ],
            tree: None,
            focus: self.focus,
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
                Key::Named(winit::keyboard::NamedKey::ArrowDown) if state.is_expanded => {
                    if state.listbox_index < LANGUAGES.len() - 1 {
                        state.update_listbox_selection(adapter, state.listbox_index + 1);
                    }
                }
                Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                    if state.combobox_index < LANGUAGES.len() - 1 {
                        state.update_combobox_value(adapter, state.combobox_index + 1);
                    }
                }
                Key::Named(winit::keyboard::NamedKey::ArrowUp) if state.is_expanded => {
                    if state.listbox_index > 0 {
                        state.update_listbox_selection(adapter, state.listbox_index - 1);
                    }
                }
                Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                    if state.combobox_index > 0 {
                        state.update_combobox_value(adapter, state.combobox_index - 1);
                    }
                }
                Key::Named(winit::keyboard::NamedKey::Space) if state.is_expanded => {
                    state.combobox_index = state.listbox_index;
                    state.collapse(adapter);
                }
                Key::Named(winit::keyboard::NamedKey::Space) => {
                    state.expand(adapter);
                }
                Key::Named(winit::keyboard::NamedKey::Escape) if state.is_expanded => {
                    state.collapse(adapter);
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
                if target == COMBOBOX_ID {
                    if (action == Action::Click || action == Action::Expand) && !state.is_expanded {
                        state.expand(adapter);
                    } else if (action == Action::Click || action == Action::Collapse)
                        && state.is_expanded
                    {
                        state.collapse(adapter);
                    } else if action == Action::Focus {
                        state.set_focus(adapter, COMBOBOX_ID);
                    }
                } else if action == Action::Click && state.is_expanded {
                    if let Some(selection) = LANGUAGES.iter().position(|(_, id)| *id == target) {
                        state.combobox_index = selection;
                        state.collapse(adapter);
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
    println!("- [Space] expands the combobox.");
    println!("- [Up] and [Down] arrow keys update the selection.");
    println!("- [Escape] collapses the combobox, discarding the selection.");
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
