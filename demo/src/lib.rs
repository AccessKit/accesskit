mod live;
mod tabs;

use accesskit::{ActionRequest, Node, NodeId, Role, Tree, TreeUpdate};
use live::LiveRegionTab;
use tabs::{Tab, TabView};

#[macro_export]
macro_rules! node_id {
    ($($name:expr $(, $index:expr)?)?) => {
        {
            use std::hash::{DefaultHasher, Hasher};

            let mut hasher = DefaultHasher::new();
            hasher.write(file!().as_bytes());
            hasher.write_u32(line!());
            hasher.write_u32(column!());
            $(
                hasher.write($name.as_bytes());
                $(
                    hasher.write_usize($index);
                )*
            )*
            NodeId(hasher.finish())
        }
    }
}

trait Widget {
    fn key_pressed(&mut self, key: Key) -> bool;

    fn has_focus(&self) -> bool;

    fn set_focused(&mut self, has_focus: bool);

    fn render(&self, update: &mut TreeUpdate) -> NodeId;

    fn do_action(&mut self, request: &ActionRequest) -> bool;
}

pub enum Key {
    Left,
    Right,
    Space,
    Tab,
}

const WINDOW_TITLE: &str = "AccessKit Demo";
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
        let tabs: Vec<Box<dyn Tab + Send>> = vec![Box::new(LiveRegionTab::new())];
        let mut root_view = Box::new(TabView::new("examples", tabs));
        root_view.set_focused(true);
        Self {
            root_view,
            id: node_id!(),
        }
    }
}

impl WindowState {
    pub fn key_pressed(&mut self, key: Key) {
        if !self.root_view.key_pressed(key) && !self.root_view.has_focus() {
            self.root_view.set_focused(true);
        }
    }

    fn build_root(&self, child: NodeId) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children(vec![child]);
        node.set_label(self.title());
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

    pub fn do_action(&mut self, request: &ActionRequest) {
        self.root_view.do_action(request);
    }

    pub fn deactivate_accessibility(&mut self) {}

    pub fn title(&self) -> &str {
        WINDOW_TITLE
    }
}
