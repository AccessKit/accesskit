pub use accesskit_schema::{Node as NodeData, Tree as TreeData};

pub(crate) mod tree;
pub use tree::{Reader as TreeReader, Tree};

pub(crate) mod node;
pub use node::{Node, WeakNode};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
