use accesskit::{Node, NodeId, Rect, Role};

use super::Widget;

/// Static text.
pub struct Label {
    id: NodeId,
    text: String,
    bounds: Rect,
}

impl Label {
    pub fn new(id: NodeId, text: impl Into<String>, bounds: Rect) -> Self {
        Self {
            id,
            text: text.into(),
            bounds,
        }
    }
}

impl<M> Widget<M> for Label {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::Label);
        node.set_bounds(self.bounds);
        node.set_value(self.text.as_str());
        node
    }
}
