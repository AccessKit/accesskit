use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Toggled};

use super::{Response, Widget};
use crate::{Key, KeyEvent, KeyState};

/// A button that is either pressed or not.
///
/// In AccessKit, this is a button with a toggled state.
pub struct ToggleButton<M> {
    id: NodeId,
    label: String,
    bounds: Rect,
    is_pressed: bool,
    controls: Option<NodeId>,
    disabled: bool,
    on_press: fn(bool) -> M,
}

impl<M> ToggleButton<M> {
    /// Creates a toggle button. `on_press` receives the new state.
    pub fn new(
        id: NodeId,
        label: impl Into<String>,
        bounds: Rect,
        is_pressed: bool,
        on_press: fn(bool) -> M,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            bounds,
            is_pressed,
            controls: None,
            disabled: false,
            on_press,
        }
    }

    /// Sets the widget that the button affects.
    pub fn with_controls(mut self, controls: NodeId) -> Self {
        self.controls = Some(controls);
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn toggle(&self) -> Response<M> {
        Response::Message((self.on_press)(!self.is_pressed))
    }
}

impl<M> Widget<M> for ToggleButton<M> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::Button);
        node.set_bounds(self.bounds);
        node.set_label(self.label.as_str());
        node.set_toggled(Toggled::from(self.is_pressed));
        if let Some(controls) = self.controls {
            node.set_controls(vec![controls]);
        }
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
            (Key::Enter | Key::Space, KeyState::Pressed) if !self.disabled => self.toggle(),
            _ => Response::Ignored,
        }
    }

    fn on_action(&self, request: &ActionRequest) -> Response<M> {
        if self.disabled {
            return Response::Ignored;
        }
        match request.action {
            Action::Click => self.toggle(),
            Action::Focus => Response::Focus(self.id),
            _ => Response::Ignored,
        }
    }
}
