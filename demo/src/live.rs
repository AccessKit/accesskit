use accesskit::{Action, ActionRequest, Live, Node, NodeId, Rect, Role, TreeUpdate};

use crate::{node_id, Key, Widget, CHARACTER_HEIGHT, CHARACTER_WIDTH, MARGIN, PADDING};

const BUTTON_HEIGHT: f64 = CHARACTER_HEIGHT + PADDING * 2.0;

fn build_button(label: &str, y: f64) -> Node {
    let mut node = Node::new(Role::Button);
    node.set_bounds(Rect {
        x0: MARGIN,
        y0: y,
        x1: MARGIN + label.len() as f64 * CHARACTER_WIDTH + PADDING * 2.0,
        y1: MARGIN + BUTTON_HEIGHT,
    });
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

pub(crate) struct LiveView {
    button_1_id: NodeId,
    button_2_id: NodeId,
    has_focus: bool,
    focus: NodeId,
    announcement: Option<String>,
}

impl LiveView {
    pub(crate) fn new() -> Self {
        let button_1_id = node_id!();
        Self {
            button_1_id,
            button_2_id: node_id!(),
            has_focus: false,
            focus: button_1_id,
            announcement: None,
        }
    }

    fn set_focus(&mut self, focus: NodeId) {
        self.focus = focus;
    }

    fn press_button(&mut self, id: NodeId) {
        let text = if id == self.button_1_id {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        self.announcement = Some(text.into());
    }
}

impl Widget for LiveView {
    fn key_pressed(&mut self, key: Key) -> bool {
        match key {
            Key::Tab => {
                if self.focus == self.button_1_id {
                    self.set_focus(self.button_2_id);
                    true
                } else {
                    self.set_focus(self.button_1_id);
                    self.set_focused(false);
                    false
                }
            }
            Key::Space => {
                self.press_button(self.focus);
                true
            }
        }
    }

    fn has_focus(&self) -> bool {
        self.has_focus
    }

    fn set_focused(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
    }

    fn render(&self, update: &mut TreeUpdate) -> NodeId {
        let mut container = Node::new(Role::GenericContainer);
        container.set_children(vec![self.button_1_id, self.button_2_id]);
        let button_1 = build_button("Button 1", MARGIN);
        let button_2 = build_button("Button 2", MARGIN + BUTTON_HEIGHT);
        update.nodes.push((self.button_1_id, button_1));
        update.nodes.push((self.button_2_id, button_2));
        if let Some(announcement) = &self.announcement {
            let announcement_id = node_id!();
            update
                .nodes
                .push((announcement_id, build_announcement(announcement)));
            container.push_child(announcement_id);
        }
        let container_id = node_id!();
        update.nodes.push((container_id, container));
        if self.has_focus() {
            update.focus = self.focus;
        }
        container_id
    }

    fn do_action(&mut self, request: ActionRequest) -> bool {
        if request.target == self.button_1_id || request.target == self.button_2_id {
            match request.action {
                Action::Focus => {
                    self.set_focus(request.target);
                    self.set_focused(true);
                }
                Action::Click => {
                    self.press_button(request.target);
                }
                _ => (),
            }
            true
        } else {
            false
        }
    }
}
