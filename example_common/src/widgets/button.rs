use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role};

use super::{Response, Widget};
use crate::{Key, KeyEvent, KeyState};

/// A push button.
pub struct Button<M> {
    id: NodeId,
    label: String,
    bounds: Rect,
    disabled: bool,
    on_press: M,
}

impl<M> Button<M> {
    /// Creates a button producing `on_press` when pressed.
    pub fn new(id: NodeId, label: impl Into<String>, bounds: Rect, on_press: M) -> Self {
        Self {
            id,
            label: label.into(),
            bounds,
            disabled: false,
            on_press,
        }
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<M: Clone> Widget<M> for Button<M> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::Button);
        node.set_bounds(self.bounds);
        node.set_label(self.label.as_str());
        if self.disabled {
            node.set_disabled();
        } else {
            node.add_action(Action::Focus);
            node.add_action(Action::Click);
        }
        node
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn on_key(&self, event: &KeyEvent, _focus: NodeId) -> Response<M> {
        match (event.key, event.state) {
            (Key::Enter | Key::Space, KeyState::Pressed) if !self.disabled => {
                Response::Message(self.on_press.clone())
            }
            _ => Response::Ignored,
        }
    }

    fn on_action(&self, request: &ActionRequest) -> Response<M> {
        if self.disabled {
            return Response::Ignored;
        }
        match request.action {
            Action::Click => Response::Message(self.on_press.clone()),
            Action::Focus => Response::Focus(self.id),
            _ => Response::Ignored,
        }
    }
}
