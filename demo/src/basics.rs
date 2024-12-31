use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Toggled, TreeUpdate};

use crate::{group, node_id, tabs::Tab, Key, Widget, CHARACTER_HEIGHT, CHARACTER_WIDTH, MARGIN, PADDING};

//const WIDGET_HEIGHT: f64 = CHARACTER_HEIGHT + PADDING * 2.0;

pub(crate) struct BasicsTab {
    has_focus: bool,
    focus: NodeId,

    regular_button_id: NodeId,
    toggle_button_id: NodeId,

    toggle_button_state: Toggled,
}

impl BasicsTab {
    pub(crate) fn new() -> Self {
        let regular_button_id = node_id!();
        Self {
            has_focus: false,
            focus: regular_button_id,
            regular_button_id,
            toggle_button_id: node_id!(),
            toggle_button_state: Toggled::False,
        }
    }

    fn focusable_widgets(&self) -> Vec<NodeId> {
        vec![self.regular_button_id, self.toggle_button_id]
    }

    fn toggle_toggle_button(&mut self) {
        self.toggle_button_state = if self.toggle_button_state == Toggled::False {
            Toggled::True
        } else {
            Toggled::False
        };
    }
}

impl Widget for BasicsTab {
    fn key_pressed(&mut self, key: Key) -> bool {
        match key {
            Key::Tab => {
                let focusable = self.focusable_widgets();
                if let Some(index) = focusable.iter().position(|x| *x == self.focus) {
                    let next_index = if index == focusable.len() - 1 {
                        0
                    } else {
                        index + 1
                    };
                    println!("{next_index}");
                    self.focus = focusable[next_index];
                    if index == focusable.len() - 1 {
                        self.set_focused(false);
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Key::Space => {
                if self.focus == self.toggle_button_id {
                    self.toggle_toggle_button();
                }
                true
            }
            _ => false,
        }
    }

    fn has_focus(&self) -> bool {
        self.has_focus
    }

    fn set_focused(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
    }

    fn render(&self, update: &mut TreeUpdate) -> NodeId {
        let container_id = node_id!();
        let mut container = Node::new(Role::GenericContainer);

        let buttons_group_id = group("Buttons", update, |update, children| {
            let mut regular_button = Node::new(Role::Button);
            regular_button.set_label("Regular button");
            regular_button.add_action(Action::Focus);
            regular_button.add_action(Action::Click);
            update.nodes.push((self.regular_button_id, regular_button));

            let mut toggle_button = Node::new(Role::Button);
            toggle_button.set_toggled(self.toggle_button_state);
            toggle_button.set_label("Toggle button");
            toggle_button.add_action(Action::Focus);
            toggle_button.add_action(Action::Click);
            update.nodes.push((self.toggle_button_id, toggle_button));
            children.extend_from_slice(&[self.regular_button_id, self.toggle_button_id]);
        });

        container.set_children(vec![buttons_group_id]);
        update.nodes.push((container_id, container));

        if self.has_focus() {
            update.focus = self.focus;
        }
        container_id
    }

    fn do_action(&mut self, request: &ActionRequest) -> bool {
        if self.focusable_widgets().contains(&request.target) {
            match request.action {
                Action::Focus => {
                    self.focus = request.target;
                    self.set_focused(true);
                }
                Action::Click => {
                    if request.target == self.toggle_button_id {
                        self.toggle_toggle_button();
                    }
                }
                _ => (),
            }
            true
        } else {
            false
        }
    }
}

impl Tab for BasicsTab {
    fn title(&self) -> &str {
        "Basics"
    }
}
