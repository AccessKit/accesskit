use accesskit::{Node, NodeId, Rect, Role};

use super::Widget;

/// A non-interactive image.
pub struct Image {
    id: NodeId,
    label: String,
    bounds: Rect,
}

impl Image {
    /// Creates an image, described by `label` for screen reader users.
    pub fn new(id: NodeId, label: impl Into<String>, bounds: Rect) -> Self {
        Self {
            id,
            label: label.into(),
            bounds,
        }
    }
}

impl<M> Widget<M> for Image {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::Image);
        node.set_bounds(self.bounds);
        node.set_label(self.label.as_str());
        node
    }
}
