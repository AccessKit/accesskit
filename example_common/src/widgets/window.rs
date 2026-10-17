use accesskit::{Affine, Node, NodeId, Rect, Role};

use super::Widget;

/// The root of the user interface.
pub struct Window<M> {
    id: NodeId,
    title: String,
    language: Option<String>,
    bounds: Option<Rect>,
    transform: Option<Affine>,
    children: Vec<Box<dyn Widget<M>>>,
}

impl<M> Window<M> {
    /// Creates a window with the given title.
    pub fn new(id: NodeId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            language: None,
            bounds: None,
            transform: None,
            children: Vec::new(),
        }
    }

    /// Sets the language of the window's content, which its descendants
    /// inherit.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Sets the area the user interface occupies.
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the transform from the coordinate space of the user interface
    /// to that of the platform window.
    pub fn with_transform(mut self, transform: Affine) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn with_child(mut self, child: impl Widget<M> + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl<M> Widget<M> for Window<M> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn build(&self) -> Node {
        let mut node = Node::new(Role::Window);
        if let Some(bounds) = self.bounds {
            node.set_bounds(bounds);
        }
        if let Some(transform) = self.transform {
            node.set_transform(transform);
        }
        node.set_children(
            self.children
                .iter()
                .map(|child| child.id())
                .collect::<Vec<_>>(),
        );
        node.set_label(self.title.as_str());
        if let Some(language) = &self.language {
            node.set_language(language.as_str());
        }
        node
    }

    fn children(&self) -> Vec<&dyn Widget<M>> {
        self.children.iter().map(|child| child.as_ref()).collect()
    }
}
