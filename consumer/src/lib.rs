pub type NodeData = accesskit_schema::Node;
pub type TreeData = accesskit_schema::Tree;

mod tree;
pub use tree::Tree;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
