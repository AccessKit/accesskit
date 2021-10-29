use accesskit_consumer::{Node, WeakNode};
use crate::atspi::interfaces::AccessibleInterface;

pub struct PlatformNode(WeakNode);

impl PlatformNode {
    pub(crate) fn new(node: &Node) -> Self {
        Self(node.downgrade())
    }
}

impl AccessibleInterface for PlatformNode {
    fn name(&self) -> String {
        self.0.map(|node| {
            match node.name() {
                None => String::new(),
                Some(name) => name.to_string()
            }
        }).unwrap()
    }

    fn description(&self) -> String {
        todo!()
    }

    fn parent(&self) -> Option<crate::atspi::OwnedObjectAddress> {
        todo!()
    }

    fn child_count(&self) -> usize {
        todo!()
    }

    fn locale(&self) -> &str {
        todo!()
    }

    fn accessible_id(&self) -> String {
        todo!()
    }

    fn child_at_index(&self, index: usize) -> Option<crate::atspi::OwnedObjectAddress> {
        todo!()
    }

    fn children(&self) -> Vec<crate::atspi::OwnedObjectAddress> {
        todo!()
    }

    fn index_in_parent(&self) -> Option<usize> {
        todo!()
    }

    fn role(&self) -> crate::atspi::Role {
        todo!()
    }
}