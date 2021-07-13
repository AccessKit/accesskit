pub use accesskit_schema::{Node as NodeData, Tree as TreeData};

mod tree;
pub use tree::Tree;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
