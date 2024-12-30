use accesskit::{Action, ActionRequest, Live, Node, NodeId, Rect, Role, TreeUpdate};

use crate::Widget;

const CONTAINER_ID: NodeId = NodeId(1);
const BUTTON_1_ID: NodeId = NodeId(2);
const BUTTON_2_ID: NodeId = NodeId(3);
const ANNOUNCEMENT_ID: NodeId = NodeId(4);
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
    focus: NodeId,
    announcement: Option<String>,
}

impl LiveView {
    pub(crate) fn new() -> Self {
        Self {
            focus: INITIAL_FOCUS,
            announcement: None,
        }
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

impl Widget for LiveView {
    fn tab_pressed(&mut self) {
        let new_focus = if self.focus == BUTTON_1_ID {
            BUTTON_2_ID
        } else {
            BUTTON_1_ID
        };
        self.set_focus(new_focus);
    }

    fn space_pressed(&mut self) {
        self.press_button(self.focus);
    }

    fn render(&self, update: &mut TreeUpdate) -> NodeId {
        let mut container = Node::new(Role::GenericContainer);
        container.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        let button_1 = build_button("Button 1", BUTTON_1_RECT);
        let button_2 = build_button("Button 2", BUTTON_2_RECT);
        update.nodes.push((BUTTON_1_ID, button_1));
        update.nodes.push((BUTTON_2_ID, button_2));
        if let Some(announcement) = &self.announcement {
            update
                .nodes
                .push((ANNOUNCEMENT_ID, build_announcement(announcement)));
            container.push_child(ANNOUNCEMENT_ID);
        }
        update.nodes.push((CONTAINER_ID, container));
        update.focus = self.focus;
        CONTAINER_ID
    }

    fn do_action(&mut self, request: ActionRequest) {
        if request.target == BUTTON_1_ID || request.target == BUTTON_2_ID {
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
