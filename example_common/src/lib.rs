mod key;
mod login;
mod main_screen;
mod render;
pub mod widgets;

pub use key::{Key, KeyEvent, KeyState, Modifiers};
pub use render::Renderer;

use accesskit::{
    ActionRequest, Affine, Node, NodeId, Rect, Size, TreeId, TreeInfo, TreeUpdate, Vec2,
};
use login::LoginScreen;
use main_screen::MainScreen;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use widgets::{
    ImeEdit, Response, ScrollPosition, TextInputInfo, TextInputState, Widget, Window, bubble, visit,
};

/// The title of the example's window.
pub const WINDOW_TITLE: &str = "AccessKit Example application";

const WINDOW_ID: NodeId = NodeId(0);

const fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
    Rect { x0, y0, x1, y1 }
}

/// The size of the user interface until the platform reports it.
const DEFAULT_SIZE: Size = Size::new(393.0, 759.0);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Viewport {
    scale_factor: f64,
    safe_area_inset: Vec2,
    size: Size,
}

#[derive(Clone, Debug)]
enum Message {
    ViewportChanged(Viewport),
    Focus(NodeId),
    // Login screen
    EmailChanged(TextInputState),
    PasswordChanged(TextInputState),
    PasswordVisibilityChanged(bool),
    LogIn,
    LoginFinished,
    // Main screen
    ContactSelected(usize),
    ChatMessageSelected(usize),
    MessagesScrolled(ScrollPosition),
    DraftChanged(TextInputState),
    SendDraft,
    ReplyReceived(usize),
}

/// What [`Model::update`] asks to happen next.
enum Command {
    None,
    /// Dispatches the message after the duration.
    PostAfter(Duration, Message),
}

enum Screen {
    Login(LoginScreen),
    Main(MainScreen),
}

struct Model {
    focus: NodeId,
    screen: Screen,
    viewport: Option<Viewport>,
}

struct View {
    root: Window<Message>,
}

impl Model {
    fn new() -> Self {
        Self {
            focus: login::INITIAL_FOCUS,
            screen: Screen::Login(LoginScreen::default()),
            viewport: None,
        }
    }

    fn update(&mut self, message: Message) -> Command {
        match message {
            Message::ViewportChanged(viewport) => self.viewport = Some(viewport),
            Message::Focus(id) => self.focus = id,
            Message::LoginFinished => {
                self.screen = Screen::Main(MainScreen::new());
                self.focus = main_screen::INITIAL_FOCUS;
            }
            message => {
                return match &mut self.screen {
                    Screen::Login(screen) => screen.update(message),
                    Screen::Main(screen) => screen.update(message),
                };
            }
        }
        Command::None
    }

    fn view(&self) -> View {
        let mut root = Window::new(WINDOW_ID, WINDOW_TITLE).with_language("en");
        if let Some(viewport) = &self.viewport {
            root = root
                .with_bounds(Rect::from_origin_size((0.0, 0.0), viewport.size))
                .with_transform(
                    Affine::translate(viewport.safe_area_inset)
                        * Affine::scale(viewport.scale_factor),
                );
        }
        let size = self.viewport.map_or(DEFAULT_SIZE, |viewport| viewport.size);
        let root = match &self.screen {
            Screen::Login(screen) => screen.view(root, size),
            Screen::Main(screen) => screen.view(root, size),
        };
        View { root }
    }
}

impl View {
    fn nodes(&self) -> BTreeMap<NodeId, Node> {
        let mut nodes = BTreeMap::new();
        visit(&self.root, &mut |widget| {
            nodes.insert(widget.id(), widget.build());
        });
        nodes
    }
}

/// The example's user interface, and the AccessKit tree describing it.
pub struct UiState {
    model: Model,
    delayed_messages: Vec<(Instant, Message)>,
    /// The nodes last sent to the AccessKit adapter, if it has a tree.
    sent_nodes: Option<BTreeMap<NodeId, Node>>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    /// Creates the user interface, starting on the login screen.
    pub fn new() -> Self {
        Self {
            model: Model::new(),
            delayed_messages: Vec::new(),
            sent_nodes: None,
        }
    }

    fn dispatch(&mut self, message: Message) {
        match self.model.update(message) {
            Command::None => (),
            Command::PostAfter(delay, message) => {
                let due = Instant::now() + delay;
                let index = self
                    .delayed_messages
                    .partition_point(|(other_due, _)| *other_due <= due);
                self.delayed_messages.insert(index, (due, message));
            }
        }
    }

    /// Sets the area of the window the user interface occupies: its logical
    /// size, and its offset in physical pixels. Returns whether it changed.
    pub fn set_viewport(&mut self, scale_factor: f64, safe_area_inset: Vec2, size: Size) -> bool {
        let viewport = Viewport {
            scale_factor,
            safe_area_inset,
            size,
        };
        if self.model.viewport == Some(viewport) {
            return false;
        }
        self.dispatch(Message::ViewportChanged(viewport));
        true
    }

    /// Builds an update for the AccessKit adapter: the whole tree when the
    /// adapter has none, and only the nodes that changed otherwise.
    pub fn build_tree_update(&mut self) -> TreeUpdate {
        let view = self.model.view();
        let nodes = view.nodes();
        let (changed, tree) = match &self.sent_nodes {
            Some(sent) => (
                nodes
                    .iter()
                    .filter(|(id, node)| sent.get(id) != Some(node))
                    .map(|(id, node)| (*id, node.clone()))
                    .collect(),
                None,
            ),
            None => (
                nodes.iter().map(|(id, node)| (*id, node.clone())).collect(),
                Some(TreeInfo::new(view.root.id())),
            ),
        };
        self.sent_nodes = Some(nodes);
        TreeUpdate {
            nodes: changed,
            tree,
            tree_id: TreeId::ROOT,
            focus: self.model.focus,
        }
    }

    /// Handles the deactivation of the AccessKit adapter, which drops its
    /// tree.
    pub fn deactivated(&mut self) {
        self.sent_nodes = None;
    }

    fn handle_response(&mut self, response: Response<Message>) -> bool {
        match response {
            Response::Ignored => return false,
            Response::Handled => (),
            Response::Message(message) => self.dispatch(message),
            Response::Messages(messages) => {
                for message in messages {
                    self.dispatch(message);
                }
            }
            Response::Focus(id) => self.dispatch(Message::Focus(id)),
            Response::FocusAndMessage(id, message) => {
                self.dispatch(Message::Focus(id));
                self.dispatch(message);
            }
        }
        true
    }

    /// Handles a key event. Returns whether it was handled.
    pub fn handle_key(&mut self, event: KeyEvent) -> bool {
        let focus = self.model.focus;
        let view = self.model.view();
        let response = match bubble(&view.root, focus, &mut |widget| {
            widget.on_key(&event, focus)
        }) {
            Response::Ignored => default_key_action(&view.root, &event, focus),
            response => response,
        };
        self.handle_response(response)
    }

    /// Handles an action request from the AccessKit adapter.
    pub fn do_action(&mut self, request: &ActionRequest) {
        let response = bubble(
            &self.model.view().root,
            request.target_node,
            &mut |widget| widget.on_action(request),
        );
        self.handle_response(response);
    }

    /// The state of the focused text input, if any.
    pub fn focused_text_input(&self) -> Option<TextInputInfo> {
        let focus = self.model.focus;
        let view = self.model.view();
        let mut result = None;
        visit(&view.root, &mut |widget| {
            if widget.id() == focus {
                result = widget.text_input();
            }
        });
        result
    }

    /// Handles an edit from the platform's input method. Returns whether it
    /// was handled.
    pub fn edit_focused_text(&mut self, edit: &ImeEdit) -> bool {
        let focus = self.model.focus;
        let response = bubble(&self.model.view().root, focus, &mut |widget| {
            widget.on_ime_edit(edit)
        });
        self.handle_response(response)
    }

    /// How long until [`UiState::process_due_messages`] should be called, if
    /// it should.
    pub fn time_until_next_message(&self) -> Option<Duration> {
        self.delayed_messages
            .first()
            .map(|(due, _)| due.saturating_duration_since(Instant::now()))
    }

    /// Handles the delayed messages that are due. Returns whether there were
    /// any.
    pub fn process_due_messages(&mut self) -> bool {
        let now = Instant::now();
        let count = self
            .delayed_messages
            .partition_point(|(due, _)| *due <= now);
        let due: Vec<_> = self.delayed_messages.drain(..count).collect();
        for (_, message) in due {
            self.dispatch(message);
        }
        count > 0
    }
}

/// Moves focus with Tab and Shift+Tab.
fn default_key_action<M>(root: &dyn Widget<M>, event: &KeyEvent, focus: NodeId) -> Response<M> {
    if event.key != Key::Tab || event.state != KeyState::Pressed {
        return Response::Ignored;
    }
    let mut widgets = Vec::new();
    visit(root, &mut |widget| {
        widgets.push((widget.id(), widget.is_focusable()));
    });
    let count = widgets.len();
    let current = widgets.iter().position(|(id, _)| *id == focus).unwrap_or(0);
    (1..=count)
        .map(|step| {
            if event.modifiers.shift {
                (current + count - step) % count
            } else {
                (current + step) % count
            }
        })
        .map(|index| widgets[index])
        .find(|(_, is_focusable)| *is_focusable)
        .map_or(Response::Ignored, |(id, _)| Response::Focus(id))
}

/// Prints how to use the example.
pub fn print_instructions() {
    println!(
        "This example is a fake messaging app. It has no visible GUI, and a keyboard interface:"
    );
    println!("- [Tab] and [Shift]+[Tab] move focus between the controls.");
    println!(
        "- In text fields, type to edit the text, and use the arrow keys, [Home] and [End] to move the caret. Hold [Shift] to select."
    );
    println!("- [Enter] or [Space] presses buttons.");
    println!("On the login screen, any email and password are accepted.");
    println!("On the main screen:");
    println!("- In the lists of contacts and messages, [Up] and [Down] select an item.");
    println!(
        "- In the message field, [Enter] sends the message, and [Ctrl]+[Enter] starts a new line."
    );
    #[cfg(target_os = "windows")]
    println!(
        "Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows)."
    );
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    println!("Enable Orca with [Super]+[Alt]+[S].");
}
