use accesskit::{Action, ActionRequest, Live, Node, NodeId, Rect, Role, Tree, TreeUpdate};

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

impl Default for UiState {
    fn default() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            announcement: None,
        }
    }
}

impl UiState {
    fn build_root(&self) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        if self.announcement.is_some() {
            node.push_child(ANNOUNCEMENT_ID);
        }
        node.set_label(WINDOW_TITLE);
        node
    }

    fn build_initial_tree(&self) -> TreeUpdate {
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

    fn build_tree(&self) -> TreeUpdate {
        let root = self.build_root();
        let button_1 = build_button(BUTTON_1_ID, "Button 1");
        let button_2 = build_button(BUTTON_2_ID, "Button 2");
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (BUTTON_1_ID, button_1),
                (BUTTON_2_ID, button_2),
            ],
            tree: None,
            focus: self.focus,
        };
        if let Some(announcement) = &self.announcement {
            result
                .nodes
                .push((ANNOUNCEMENT_ID, build_announcement(announcement)));
        }
        result
    }

    fn set_focus(&mut self, focus: NodeId) {
        self.focus = focus;
    }

    fn press_button(&mut self, id: NodeId) {
        let text = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        self.announcement = Some(text.into());
    }
}

#[derive(Default)]
pub struct WindowState {
    ui: UiState,
}

impl WindowState {
    pub fn tab_pressed(&mut self) {
        let new_focus = if self.ui.focus == BUTTON_1_ID {
            BUTTON_2_ID
        } else {
            BUTTON_1_ID
        };
        self.ui.set_focus(new_focus);
    }

    pub fn space_pressed(&mut self) {
        let id = self.ui.focus;
        self.ui.press_button(id);
    }

    pub fn build_initial_tree(&mut self) -> TreeUpdate {
        self.ui.build_initial_tree()
    }

    pub fn build_tree(&self) -> TreeUpdate {
        self.ui.build_tree()
    }

    pub fn do_action(&mut self, request: ActionRequest) {
        if request.target == BUTTON_1_ID || request.target == BUTTON_2_ID {
            match request.action {
                Action::Focus => {
                    self.ui.set_focus(request.target);
                }
                Action::Click => {
                    self.ui.press_button(request.target);
                }
                _ => (),
            }
        }
    }

    pub fn deactivate_accessibility(&mut self) {}

    pub fn title(&self) -> &str {
        WINDOW_TITLE
    }
}
