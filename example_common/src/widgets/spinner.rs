use accesskit::{Node, NodeId, Rect, Role};

use super::Widget;

/// An indeterminate progress indicator.
///
/// It has no numeric value, which is how AccessKit tells that the progress
/// is indeterminate.
pub struct Spinner {
    id: NodeId,
    label: String,
    bounds: Rect,
}

impl Spinner {
    /// Creates a spinner described by `label`.
    pub fn new(id: NodeId, label: impl Into<String>, bounds: Rect) -> Self {
        Self {
            id,
            label: label.into(),
            bounds,
        }
    }
}

impl<M> Widget<M> for Spinner {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::ProgressIndicator);
        node.set_bounds(self.bounds);
        node.set_label(self.label.as_str());
        node
    }
}
