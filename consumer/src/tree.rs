// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use accesskit_schema::{NodeId, TreeId, TreeUpdate};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::{Node, NodeData, TreeData};

pub(crate) struct ParentAndIndex(pub(crate) NodeId, pub(crate) usize);

pub(crate) struct NodeState {
    pub(crate) parent_and_index: Option<ParentAndIndex>,
    pub(crate) data: NodeData,
}

pub(crate) struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) root: NodeId,
    pub(crate) data: TreeData,
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

    fn update(&mut self, update: TreeUpdate) {
        // TODO: handle TreeUpdate::clear
        assert!(update.clear.is_none());

        let root = update.root.unwrap_or(self.root);
        let mut pending_nodes: HashMap<NodeId, _> = HashMap::new();
        let mut pending_children = HashMap::new();
        let mut orphans = HashSet::new();

        if root != self.root {
            orphans.insert(self.root);
            self.root = root;
        }

        for node_data in update.nodes {
            let node_id = node_data.id;
            orphans.remove(&node_id);

            let mut seen_child_ids = HashSet::new();
            for (child_index, child_id) in node_data.children.iter().enumerate() {
                assert!(!seen_child_ids.contains(child_id));
                orphans.remove(child_id);
                let parent_and_index = ParentAndIndex(node_id, child_index);
                if let Some(child_state) = self.nodes.get_mut(child_id) {
                    child_state.parent_and_index = Some(parent_and_index);
                } else if let Some(child_data) = pending_nodes.remove(child_id) {
                    let node_state = NodeState {
                        parent_and_index: Some(parent_and_index),
                        data: child_data,
                    };
                    self.nodes.insert(*child_id, node_state);
                } else {
                    pending_children.insert(*child_id, parent_and_index);
                }
                seen_child_ids.insert(child_id);
            }

            if let Some(node_state) = self.nodes.get_mut(&node_id) {
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
                self.nodes.insert(node_id, node_state);
            } else if node_id == root {
                let node_state = NodeState {
                    parent_and_index: None,
                    data: node_data,
                };
                self.nodes.insert(node_id, node_state);
            } else {
                pending_nodes.insert(node_id, node_data);
            }
        }

        if !pending_nodes.is_empty() {
            for (node_id, data) in &pending_nodes {
                println!("unattached: {:?} {:?}", node_id, data.role);
            }
            panic!("unattached nodes");
        }

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
                traverse_orphan(&self.nodes, &mut to_remove, id);
            }

            for id in to_remove {
                self.nodes.remove(&id);
            }
        }

        if let Some(tree) = update.tree {
            assert_eq!(tree.id, self.data.id);
            self.data = tree;
        }

        self.validate_global();
    }

    fn serialize(&self) -> TreeUpdate {
        let mut nodes = Vec::new();

        fn traverse(state: &State, nodes: &mut Vec<NodeData>, id: NodeId) {
            let node = state.nodes.get(&id).unwrap();
            nodes.push(node.data.clone());

            for child_id in &node.data.children {
                traverse(state, nodes, *child_id);
            }
        }

        traverse(self, &mut nodes, self.root);
        assert_eq!(nodes.len(), self.nodes.len());

        TreeUpdate {
            clear: None,
            nodes,
            tree: Some(self.data.clone()),
            root: Some(self.root),
        }
    }
}

pub struct Reader<'a> {
    pub(crate) tree: &'a Arc<Tree>,
    pub(crate) state: RwLockReadGuard<'a, State>,
}

impl Reader<'_> {
    pub fn node_by_id<'a>(&'a self, id: NodeId) -> Option<Node<'a>> {
        self.state.nodes.get(&id).map(|node_state| Node {
            tree_reader: &self,
            state: node_state,
        })
    }

    pub fn root<'a>(&'a self) -> Node<'a> {
        self.node_by_id(self.state.root).unwrap()
    }

    pub fn id<'a>(&'a self) -> &'a TreeId {
        &self.state.data.id
    }
}

pub struct Tree {
    state: RwLock<State>,
}

impl Tree {
    pub fn new(mut initial_state: TreeUpdate) -> Arc<Self> {
        assert!(initial_state.clear.is_none());

        let mut state = State {
            nodes: HashMap::new(),
            root: initial_state.root.take().unwrap(),
            data: initial_state.tree.take().unwrap(),
        };
        state.update(initial_state);
        Arc::new(Self {
            state: RwLock::new(state),
        })
    }

    pub fn update(&self, update: TreeUpdate) {
        let mut state = self.state.write().unwrap();
        state.update(update)
    }

    // Intended for debugging.
    pub fn serialize(&self) -> TreeUpdate {
        let state = self.state.read().unwrap();
        state.serialize()
    }

    pub fn read<'a>(self: &'a Arc<Tree>) -> Reader<'a> {
        Reader {
            tree: self,
            state: self.state.read().unwrap(),
        }
    }
}
