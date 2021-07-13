use accesskit_schema::{NodeId, TreeUpdate};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use crate::{NodeData, TreeData};

struct ParentAndIndex(NodeId, usize);

struct NodeState {
    parent_and_index: Option<ParentAndIndex>,
    data: NodeData,
}

struct State {
    nodes: HashMap<NodeId, NodeState>,
    root: NodeId,
    data: TreeData,
}

impl State {
    fn validate_global(&self) {
        assert!(self.nodes.contains_key(&self.root));
        if let Some(id) = self.data.focus {
            assert!(self.nodes.contains_key(&id));
        }
        if let Some(id) = self.data.root_scroller {
            assert!(self.nodes.contains_key(&id));
        }
    }
}

pub struct Tree {
    state: RwLock<Option<State>>,
}

impl Tree {
    pub fn new(initial_state: TreeUpdate) -> Self {
        assert!(initial_state.clear.is_none());

        let root = initial_state.root.unwrap();
        let mut nodes = HashMap::new();
        let mut pending_nodes = HashMap::new();
        let mut pending_children = HashMap::new();

        for node_data in initial_state.nodes {
            let node_id = node_data.id;
            assert!(!nodes.contains_key(&node_id));

            let mut seen_child_ids = HashSet::new();
            for (child_index, child_id) in node_data.children.iter().enumerate() {
                assert!(!seen_child_ids.contains(child_id));
                let parent_and_index = ParentAndIndex(node_id, child_index);
                if let Some(child_data) = pending_nodes.remove(child_id) {
                    let node_state = NodeState {
                        parent_and_index: Some(parent_and_index),
                        data: child_data,
                    };
                    nodes.insert(*child_id, node_state);
                } else {
                    pending_children.insert(*child_id, parent_and_index);
                }
                seen_child_ids.insert(child_id);
            }

            if let Some(parent_and_index) = pending_children.remove(&node_id) {
                let node_state = NodeState {
                    parent_and_index: Some(parent_and_index),
                    data: node_data,
                };
                nodes.insert(node_id, node_state);
            } else if node_id == root {
                let node_state = NodeState {
                    parent_and_index: None,
                    data: node_data,
                };
                nodes.insert(node_id, node_state);
            } else {
                pending_nodes.insert(node_id, node_data);
            }
        }

        assert_eq!(pending_nodes.len(), 0);
        assert_eq!(pending_children.len(), 0);

        let state = State {
            nodes,
            root,
            data: initial_state.tree.unwrap(),
        };
        state.validate_global();
        Self {
            state: RwLock::new(Some(state)),
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        // TODO: handle TreeUpdate::clear
        assert!(update.clear.is_none());

        let mut state_guard = self.state.write().unwrap();
        let mut state = state_guard.as_mut().unwrap();

        let root = update.root.unwrap_or(state.root);
        let mut pending_nodes: HashMap<NodeId, _> = HashMap::new();
        let mut pending_children = HashMap::new();
        let mut orphans = HashSet::new();

        if root != state.root {
            orphans.insert(root);
        }

        for node_data in update.nodes {
            let node_id = node_data.id;
            orphans.remove(&node_id);

            let mut seen_child_ids = HashSet::new();
            for (child_index, child_id) in node_data.children.iter().enumerate() {
                assert!(!seen_child_ids.contains(child_id));
                orphans.remove(child_id);
                let parent_and_index = ParentAndIndex(node_id, child_index);
                if let Some(child_state) = state.nodes.get_mut(child_id) {
                    child_state.parent_and_index = Some(parent_and_index);
                } else if let Some(child_data) = pending_nodes.remove(child_id) {
                    let node_state = NodeState {
                        parent_and_index: Some(parent_and_index),
                        data: child_data,
                    };
                    state.nodes.insert(*child_id, node_state);
                } else {
                    pending_children.insert(*child_id, parent_and_index);
                }
                seen_child_ids.insert(child_id);
            }

            if let Some(node_state) = state.nodes.get_mut(&node_id) {
                if node_id == root {
                    node_state.parent_and_index = None
                }
                for child_id in &node_state.data.children {
                    if !seen_child_ids.contains(child_id) {
                        orphans.insert(*child_id);
                    }
                }
                node_state.data = node_data;
            } else if let Some(parent_and_index) = pending_children.remove(&node_id) {
                let node_state = NodeState {
                    parent_and_index: Some(parent_and_index),
                    data: node_data,
                };
                state.nodes.insert(node_id, node_state);
            } else if node_id == root {
                let node_state = NodeState {
                    parent_and_index: None,
                    data: node_data,
                };
                state.nodes.insert(node_id, node_state);
            } else {
                pending_nodes.insert(node_id, node_data);
            }
        }

        assert_eq!(pending_nodes.len(), 0);
        assert_eq!(pending_children.len(), 0);

        if !orphans.is_empty() {
            let mut to_remove = HashSet::new();

            fn traverse_orphan(
                nodes: &HashMap<NodeId, NodeState>,
                to_remove: &mut HashSet<NodeId>,
                id: NodeId,
            ) {
                to_remove.insert(id);
                let node = nodes.get(&id).unwrap();
                for child_id in &node.data.children {
                    traverse_orphan(nodes, to_remove, *child_id);
                }
            }

            for id in orphans {
                traverse_orphan(&state.nodes, &mut to_remove, id);
            }

            for id in to_remove {
                state.nodes.remove(&id);
            }
        }

        if let Some(tree) = update.tree {
            assert_eq!(tree.id, state.data.id);
            state.data = tree;
        }

        if let Some(root) = update.root {
            state.root = root;
        }

        state.validate_global();
    }

    pub fn is_alive(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_some()
    }

    // Intended for debugging.
    pub fn serialize(&self) -> TreeUpdate {
        let state_guard = self.state.read().unwrap();
        let state = state_guard.as_ref().unwrap();
        let mut nodes = Vec::new();

        fn traverse(state: &State, nodes: &mut Vec<NodeData>, id: NodeId) {
            let node = state.nodes.get(&id).unwrap();
            nodes.push(node.data.clone());

            for child_id in &node.data.children {
                traverse(state, nodes, *child_id);
            }
        }

        traverse(state, &mut nodes, state.root);
        assert_eq!(nodes.len(), state.nodes.len());

        TreeUpdate {
            clear: None,
            nodes,
            tree: Some(state.data.clone()),
            root: Some(state.root),
        }
    }
}
