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

const WINDOW_TITLE: &str = "ComboBox example";

const WINDOW_ID: NodeId = NodeId(0);
const COMBOBOX_ID: NodeId = NodeId(1);
const POPUP_ID: NodeId = NodeId(2);
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

const POPUP_RECT: Rect = Rect {
    x0: COMBOBOX_RECT.x0,
    y0: COMBOBOX_RECT.y0,
    x1: COMBOBOX_RECT.x1,
    y1: COMBOBOX_RECT.y0 + ITEM_HEIGHT * (LANGUAGES.len() as f64),
};

fn build_combobox(expanded: bool) -> Node {
    let mut node = Node::new(Role::ComboBox);
    node.set_bounds(COMBOBOX_RECT);
    node.set_children([POPUP_ID]);
    node.set_expanded(expanded);
    node.set_label("Select your language");
    node.add_action(Action::Click);
    node.add_action(Action::Focus);
    node.add_action(if expanded {
        Action::Collapse
    } else {
        Action::Expand
    });
    node
}

fn build_popup() -> Node {
    let mut node = Node::new(Role::MenuListPopup);
    node.set_bounds(POPUP_RECT);
    node.set_children(
        LANGUAGES
            .iter()
            .map(|(_, id)| id)
            .copied()
            .collect::<Vec<NodeId>>(),
    );
    node.set_size_of_set(LANGUAGES.len());
    node
}

fn build_option(index: usize, selected: bool) -> Node {
    let mut node = Node::new(Role::MenuListOption);
    node.set_bounds(Rect {
        x0: POPUP_RECT.x0,
        y0: POPUP_RECT.y0 + (index as f64) * ITEM_HEIGHT,
        x1: POPUP_RECT.x1,
        y1: POPUP_RECT.y0 + (index as f64) * ITEM_HEIGHT + ITEM_HEIGHT,
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
    selected_index: usize,
    previous_selected_index: Option<usize>,
}

impl UiState {
    fn new() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            selected_index: 0,
            previous_selected_index: None,
        }
    }

    fn build_root(&mut self) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children([COMBOBOX_ID]);
        node.set_label(WINDOW_TITLE);
        node
    }

    fn is_expanded(&self) -> bool {
        self.previous_selected_index.is_some()
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let combobox = build_combobox(self.is_expanded());
        let popup = build_popup();
        let tree = Tree::new(WINDOW_ID);
        let mut update = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (COMBOBOX_ID, combobox),
                (POPUP_ID, popup),
            ],
            tree: Some(tree),
            focus: self.focus,
        };
        for (i, (_, id)) in LANGUAGES.iter().enumerate() {
            let is_selected = i == self.selected_index;
            update.nodes.push((*id, build_option(i, is_selected)));
        }
        update
    }

    fn set_focus(&mut self, adapter: &mut Adapter, id: NodeId) {
        self.focus = id;
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: self.focus,
        });
    }

    fn expand(&mut self, adapter: &mut Adapter) {
        self.previous_selected_index = Some(self.selected_index);
        self.focus = LANGUAGES[self.selected_index].1;
        adapter.update_if_active(|| {
            let combobox = build_combobox(self.is_expanded());
            TreeUpdate {
                nodes: vec![(COMBOBOX_ID, combobox)],
                tree: None,
                focus: self.focus,
            }
        });
    }

    fn confirm_selection_and_collapse(&mut self, adapter: &mut Adapter) {
        self.previous_selected_index = None;
        self.focus = COMBOBOX_ID;
        adapter.update_if_active(|| {
            let combobox = build_combobox(self.is_expanded());
            TreeUpdate {
                nodes: vec![(COMBOBOX_ID, combobox)],
                tree: None,
                focus: self.focus,
            }
        });
    }

    fn discard_selection_and_collapse(&mut self, adapter: &mut Adapter) {
        let previous_selection = self.previous_selected_index.take();
        if let Some(previous_selection) = previous_selection {
            self.selected_index = previous_selection;
        }
        self.focus = COMBOBOX_ID;
        adapter.update_if_active(|| {
            let combobox = build_combobox(self.is_expanded());
            let selected_option = build_option(self.selected_index, true);
            let mut update = TreeUpdate {
                nodes: vec![
                    (COMBOBOX_ID, combobox),
                    (LANGUAGES[self.selected_index].1, selected_option),
                ],
                tree: None,
                focus: self.focus,
            };
            if let Some(previous_selection) = previous_selection
                .filter(|previous_selection| *previous_selection != self.selected_index)
            {
                update.nodes.push((
                    LANGUAGES[previous_selection].1,
                    build_option(previous_selection, false),
                ));
            }
            update
        });
    }

    fn update_selection(&mut self, adapter: &mut Adapter, index: usize) {
        let previous_selection = self.selected_index;
        self.selected_index = index;
        if self.is_expanded() {
            self.focus = LANGUAGES[index].1;
        }
        adapter.update_if_active(|| TreeUpdate {
            nodes: vec![
                (LANGUAGES[index].1, build_option(index, true)),
                (
                    LANGUAGES[previous_selection].1,
                    build_option(previous_selection, false),
                ),
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
                Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                    if state.selected_index < LANGUAGES.len() - 1 {
                        state.update_selection(adapter, state.selected_index + 1);
                    }
                }
                Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                    if state.selected_index > 0 {
                        state.update_selection(adapter, state.selected_index - 1);
                    }
                }
                Key::Named(winit::keyboard::NamedKey::Space) if state.is_expanded() => {
                    state.confirm_selection_and_collapse(adapter);
                }
                Key::Named(winit::keyboard::NamedKey::Space) => {
                    state.expand(adapter);
                }
                Key::Named(winit::keyboard::NamedKey::Escape) if state.is_expanded() => {
                    state.discard_selection_and_collapse(adapter);
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
                    if (action == Action::Click || action == Action::Expand) && !state.is_expanded()
                    {
                        state.expand(adapter);
                    } else if (action == Action::Click || action == Action::Collapse)
                        && state.is_expanded()
                    {
                        state.discard_selection_and_collapse(adapter);
                    } else if action == Action::Focus {
                        state.set_focus(adapter, COMBOBOX_ID);
                    }
                } else if action == Action::Click && state.is_expanded() {
                    if let Some(selection) = LANGUAGES.iter().position(|(_, id)| *id == target) {
                        state.selected_index = selection;
                        state.confirm_selection_and_collapse(adapter);
                    }
                } else if action == Action::Focus && state.is_expanded() {
                    if LANGUAGES.iter().any(|(_, id)| *id == target) {
                        state.set_focus(adapter, target);
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
