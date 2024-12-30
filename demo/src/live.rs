use accesskit::{Action, ActionRequest, Live, Node, NodeId, Rect, Role, TreeUpdate};

use crate::{node_id, Widget};

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

fn build_button(label: &str, rect: Rect) -> Node {
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

pub(crate) struct LiveView {
    button_1_id: NodeId,
    button_2_id: NodeId,
    focus: NodeId,
    announcement: Option<String>,
}

impl LiveView {
    pub(crate) fn new() -> Self {
        let button_1_id = node_id!();
        Self {
            button_1_id,
            button_2_id: node_id!(),
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
    fn tab_pressed(&mut self) {
        let new_focus = if self.focus == self.button_1_id {
            self.button_2_id
        } else {
            self.button_1_id
        };
        self.set_focus(new_focus);
    }

    fn space_pressed(&mut self) {
        self.press_button(self.focus);
    }

    fn render(&self, update: &mut TreeUpdate) -> NodeId {
        let mut container = Node::new(Role::GenericContainer);
        container.set_children(vec![self.button_1_id, self.button_2_id]);
        let button_1 = build_button("Button 1", BUTTON_1_RECT);
        let button_2 = build_button("Button 2", BUTTON_2_RECT);
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
        update.focus = self.focus;
        container_id
    }

    fn do_action(&mut self, request: ActionRequest) {
        if request.target == self.button_1_id || request.target == self.button_2_id {
            match request.action {
                Action::Focus => {
                    self.set_focus(request.target);
                }
                Action::Click => {
                    self.press_button(request.target);
                }
                _ => (),
            }
        }
    }
}
