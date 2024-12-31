mod live;

use accesskit::{ActionRequest, Node, NodeId, Role, Tree, TreeUpdate};
use live::LiveView;

#[macro_export]
macro_rules! node_id {
    ($($index:expr)?) => {
        {
            use std::hash::{DefaultHasher, Hasher};

            let mut hasher = DefaultHasher::new();
            hasher.write(file!().as_bytes());
            hasher.write_u32(line!());
            hasher.write_u32(column!());
            $(
                hasher.write_usize($index);
            )*
            NodeId(hasher.finish())
        }
    }
}

trait Widget {
    fn key_pressed(&mut self, key: Key);

    fn render(&self, update: &mut TreeUpdate) -> NodeId;

    fn do_action(&mut self, request: ActionRequest);
}

pub enum Key {
    Space,
    Tab,
}

const WINDOW_TITLE: &str = "Hello world";
const MARGIN: f64 = 20.0;
const PADDING: f64 = 5.0;
const CHARACTER_WIDTH: f64 = 12.0;
const CHARACTER_HEIGHT: f64 = 20.0;

pub struct WindowState {
    root_view: Box<dyn Widget + Send>,
    id: NodeId,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            root_view: Box::new(LiveView::new()),
            id: node_id!(),
        }
    }
}

impl WindowState {
    pub fn key_pressed(&mut self, key: Key) {
        self.root_view.key_pressed(key);
    }

    fn build_root(&self, child: NodeId) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children(vec![child]);
        node.set_label(WINDOW_TITLE);
        node
    }

    pub fn build_initial_tree(&mut self) -> TreeUpdate {
        let mut update = self.build_tree();
        update.tree = Some(Tree::new(self.id));
        update
    }

    pub fn build_tree(&self) -> TreeUpdate {
        let mut update = TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            focus: self.id,
        };
        let root_child = self.root_view.render(&mut update);
        let root = self.build_root(root_child);
        update.nodes.push((self.id, root));
        update
    }

    pub fn do_action(&mut self, request: ActionRequest) {
        self.root_view.do_action(request);
    }

    pub fn deactivate_accessibility(&mut self) {}

    pub fn title(&self) -> &str {
        WINDOW_TITLE
    }
}
