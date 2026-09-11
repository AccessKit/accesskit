mod key;
mod render;

pub use key::{Key, KeyEvent, KeyState, Modifiers};
pub use render::Renderer;

use accesskit::{
    Action, ActionRequest, Affine, Live, Node, NodeId, Rect, Role, TreeId, TreeInfo, TreeUpdate,
    Vec2,
};
use std::{
    collections::BTreeSet,
    mem::take,
    time::{Duration, Instant},
};

pub const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(0);
const BUTTON_1_ID: NodeId = NodeId(1);
const BUTTON_2_ID: NodeId = NodeId(2);
const ANNOUNCEMENT_ID: NodeId = NodeId(3);
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;
const FOCUS_ORDER: [NodeId; 2] = [BUTTON_1_ID, BUTTON_2_ID];

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

struct Viewport {
    scale_factor: f64,
    safe_area_inset: Vec2,
}

pub struct UiState {
    focus: NodeId,
    announcement: Option<&'static str>,
    pending_announcement: Option<(&'static str, Instant)>,
    viewport: Option<Viewport>,
    adapter_has_tree: bool,
    dirty_nodes: BTreeSet<NodeId>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    pub fn new() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            announcement: None,
            pending_announcement: None,
            viewport: None,
            adapter_has_tree: false,
            dirty_nodes: BTreeSet::new(),
        }
    }

    pub fn set_viewport(&mut self, scale_factor: f64, safe_area_inset: Vec2) {
        self.viewport = Some(Viewport {
            scale_factor,
            safe_area_inset,
        });
        self.dirty_nodes.insert(WINDOW_ID);
    }

    fn build_root(&self) -> Node {
        let mut node = Node::new(Role::Window);
        if let Some(viewport) = &self.viewport {
            node.set_bounds(WINDOW_RECT);
            node.set_transform(
                Affine::translate(viewport.safe_area_inset) * Affine::scale(viewport.scale_factor),
            );
        }
        node.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        if self.announcement.is_some() {
            node.push_child(ANNOUNCEMENT_ID);
        }
        node.set_label(WINDOW_TITLE);
        node.set_language("en");
        node
    }

    fn build_node(&self, id: NodeId) -> Node {
        match id {
            WINDOW_ID => self.build_root(),
            BUTTON_1_ID => build_button(BUTTON_1_ID, "Button 1"),
            BUTTON_2_ID => build_button(BUTTON_2_ID, "Button 2"),
            ANNOUNCEMENT_ID => build_announcement(self.announcement.unwrap()),
            _ => unreachable!(),
        }
    }

    pub fn build_tree_update(&mut self) -> TreeUpdate {
        if self.adapter_has_tree {
            self.build_update_for_dirty_nodes()
        } else {
            self.build_full_tree()
        }
    }

    fn build_full_tree(&mut self) -> TreeUpdate {
        self.adapter_has_tree = true;
        self.dirty_nodes.clear();
        let mut ids = vec![WINDOW_ID, BUTTON_1_ID, BUTTON_2_ID];
        if self.announcement.is_some() {
            ids.push(ANNOUNCEMENT_ID);
        }
        TreeUpdate {
            nodes: ids
                .into_iter()
                .map(|id| (id, self.build_node(id)))
                .collect(),
            tree: Some(TreeInfo::new(WINDOW_ID)),
            tree_id: TreeId::ROOT,
            focus: self.focus,
        }
    }

    fn build_update_for_dirty_nodes(&mut self) -> TreeUpdate {
        let ids = take(&mut self.dirty_nodes);
        TreeUpdate {
            nodes: ids
                .into_iter()
                .map(|id| (id, self.build_node(id)))
                .collect(),
            tree: None,
            tree_id: TreeId::ROOT,
            focus: self.focus,
        }
    }

    pub fn deactivated(&mut self) {
        self.adapter_has_tree = false;
        self.dirty_nodes.clear();
    }

    fn set_focus(&mut self, focus: NodeId) {
        self.focus = focus;
    }

    fn move_focus(&mut self, forward: bool) {
        let count = FOCUS_ORDER.len();
        let current = FOCUS_ORDER
            .iter()
            .position(|id| *id == self.focus)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        self.focus = FOCUS_ORDER[next];
    }

    fn press_button(&mut self, id: NodeId) {
        let text = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        // On iOS, VoiceOver announces the label of the activated button.
        // Postpone the live region update so the messages don't overlap.
        self.pending_announcement = Some((text, Instant::now()));
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            Key::Enter | Key::Space => self.press_button(self.focus),
            Key::Tab => self.move_focus(!event.modifiers.shift),
        }
    }

    pub fn do_action(&mut self, request: &ActionRequest) {
        if request.target_node != BUTTON_1_ID && request.target_node != BUTTON_2_ID {
            return;
        }
        match request.action {
            Action::Focus => self.set_focus(request.target_node),
            Action::Click => self.press_button(request.target_node),
            _ => (),
        }
    }

    pub fn time_until_announcement(&self) -> Option<Duration> {
        self.pending_announcement
            .map(|(_, queued_at)| ANNOUNCEMENT_DELAY.saturating_sub(queued_at.elapsed()))
    }

    pub fn flush_announcement(&mut self) -> bool {
        let Some((text, _)) = self.pending_announcement.take() else {
            return false;
        };
        self.announcement = Some(text);
        self.dirty_nodes.insert(WINDOW_ID);
        self.dirty_nodes.insert(ANNOUNCEMENT_ID);
        true
    }
}

pub fn print_instructions() {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!(
        "- [Space] 'presses' the button, adding static text in a live region announcing that it was pressed."
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
