// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_schema::{NodeId, TreeId, TreeUpdate};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::{Node, NodeData, TreeData};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParentAndIndex(pub(crate) NodeId, pub(crate) usize);

pub(crate) struct NodeState {
    pub(crate) parent_and_index: Option<ParentAndIndex>,
    pub(crate) data: Box<NodeData>,
}

pub(crate) struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) root: NodeId,
    pub(crate) data: TreeData,
}

enum InternalChange {
    NodeAdded(NodeId),
    NodeUpdated {
        old_data: Box<NodeData>,
    },
    FocusMoved {
        old_id: Option<NodeId>,
        new_id: Option<NodeId>,
    },
    NodeRemoved(Box<NodeData>),
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

    fn update(&mut self, update: TreeUpdate, mut changes: Option<&mut Vec<InternalChange>>) {
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

        fn add_node(
            nodes: &mut HashMap<NodeId, NodeState>,
            changes: &mut Option<&mut Vec<InternalChange>>,
            parent_and_index: Option<ParentAndIndex>,
            data: NodeData,
        ) {
            let id = data.id;
            let state = NodeState {
                parent_and_index,
                data: Box::new(data),
            };
            nodes.insert(id, state);
            if let Some(changes) = changes {
                changes.push(InternalChange::NodeAdded(id));
            }
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
                    if child_state.parent_and_index != Some(parent_and_index) {
                        child_state.parent_and_index = Some(parent_and_index);
                    }
                } else if let Some(child_data) = pending_nodes.remove(child_id) {
                    add_node(
                        &mut self.nodes,
                        &mut changes,
                        Some(parent_and_index),
                        child_data,
                    );
                } else {
                    pending_children.insert(*child_id, parent_and_index);
                }
                seen_child_ids.insert(child_id);
            }

            if let Some(node_state) = self.nodes.get_mut(&node_id) {
                if node_id == root {
                    node_state.parent_and_index = None
                }
                for child_id in node_state.data.children.iter() {
                    if !seen_child_ids.contains(child_id) {
                        orphans.insert(*child_id);
                    }
                }
                if *node_state.data != node_data {
                    let old_data = std::mem::replace(&mut node_state.data, Box::new(node_data));
                    if let Some(changes) = &mut changes {
                        changes.push(InternalChange::NodeUpdated { old_data });
                    }
                }
            } else if let Some(parent_and_index) = pending_children.remove(&node_id) {
                add_node(
                    &mut self.nodes,
                    &mut changes,
                    Some(parent_and_index),
                    node_data,
                );
            } else if node_id == root {
                add_node(&mut self.nodes, &mut changes, None, node_data);
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

        if let Some(tree) = update.tree {
            assert_eq!(tree.id, self.data.id);
            if tree.focus != self.data.focus {
                if let Some(changes) = &mut changes {
                    changes.push(InternalChange::FocusMoved {
                        old_id: self.data.focus,
                        new_id: tree.focus,
                    });
                }
            }
            self.data = tree;
        }

        if !orphans.is_empty() {
            let mut to_remove = HashSet::new();

            fn traverse_orphan(
                nodes: &HashMap<NodeId, NodeState>,
                to_remove: &mut HashSet<NodeId>,
                id: NodeId,
            ) {
                to_remove.insert(id);
                let node = nodes.get(&id).unwrap();
                for child_id in node.data.children.iter() {
                    traverse_orphan(nodes, to_remove, *child_id);
                }
            }

            for id in orphans {
                traverse_orphan(&self.nodes, &mut to_remove, id);
            }

            for id in to_remove {
                if let Some(old_state) = self.nodes.remove(&id) {
                    if let Some(changes) = &mut changes {
                        changes.push(InternalChange::NodeRemoved(old_state.data));
                    }
                }
            }
        }

        self.validate_global();
    }

    fn serialize(&self) -> TreeUpdate {
        let mut nodes = Vec::new();

        fn traverse(state: &State, nodes: &mut Vec<NodeData>, id: NodeId) {
            let node = state.nodes.get(&id).unwrap();
            nodes.push((*node.data).clone());

            for child_id in node.data.children.iter() {
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
    pub fn node_by_id(&self, id: NodeId) -> Option<Node<'_>> {
        self.state.nodes.get(&id).map(|node_state| Node {
            tree_reader: self,
            state: node_state,
        })
    }

    pub fn root(&self) -> Node<'_> {
        self.node_by_id(self.state.root).unwrap()
    }

    pub fn id(&self) -> &TreeId {
        &self.state.data.id
    }
}

pub enum Change<'a> {
    NodeAdded(Node<'a>),
    NodeUpdated {
        old_data: Box<NodeData>,
        new_node: Node<'a>,
    },
    FocusMoved {
        old_id: Option<NodeId>,
        new_node: Option<Node<'a>>,
    },
    NodeRemoved(Box<NodeData>),
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
        state.update(initial_state, None);
        Arc::new(Self {
            state: RwLock::new(state),
        })
    }

    pub fn update(&self, update: TreeUpdate) {
        let mut state = self.state.write();
        state.update(update, None);
    }

    pub fn update_and_process_changes<F>(self: &Arc<Tree>, update: TreeUpdate, mut f: F)
    where
        for<'a> F: FnMut(Change<'a>),
    {
        let mut changes = Vec::<InternalChange>::new();
        let mut state = self.state.write();
        state.update(update, Some(&mut changes));
        let state = RwLockWriteGuard::downgrade(state);
        let reader = Reader { tree: self, state };
        for change in changes {
            match change {
                InternalChange::NodeAdded(id) => {
                    let node = reader.node_by_id(id).unwrap();
                    f(Change::NodeAdded(node));
                }
                InternalChange::NodeUpdated { old_data } => {
                    let id = old_data.id;
                    let new_node = reader.node_by_id(id).unwrap();
                    f(Change::NodeUpdated { old_data, new_node });
                }
                InternalChange::FocusMoved { old_id, new_id } => {
                    let new_node = new_id.map(|id| reader.node_by_id(id)).flatten();
                    f(Change::FocusMoved { old_id, new_node });
                }
                InternalChange::NodeRemoved(old_data) => {
                    f(Change::NodeRemoved(old_data));
                }
            };
        }
    }

    // Intended for debugging.
    pub fn serialize(&self) -> TreeUpdate {
        let state = self.state.read();
        state.serialize()
    }

    // https://github.com/rust-lang/rust-clippy/issues/7296
    #[allow(clippy::needless_lifetimes)]
    pub fn read<'a>(self: &'a Arc<Tree>) -> Reader<'a> {
        Reader {
            tree: self,
            state: self.state.read(),
        }
    }
}

#[cfg(test)]
mod tests {
    use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
    use std::num::NonZeroU64;

    const TREE_ID: &str = "test_tree";
    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });

    #[test]
    fn init_tree_with_root_node() {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![Node::new(NODE_ID_1, Role::Window)],
            tree: Some(Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8)),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(update);
        assert_eq!(&TreeId(TREE_ID.into()), tree.read().id());
        assert_eq!(NODE_ID_1, tree.read().root().id());
        assert_eq!(Role::Window, tree.read().root().role());
        assert!(tree.read().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: Box::new([NODE_ID_2, NODE_ID_3]),
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node::new(NODE_ID_2, Role::Button),
                Node::new(NODE_ID_3, Role::Button),
            ],
            tree: Some(Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8)),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(update);
        let reader = tree.read();
        assert_eq!(
            NODE_ID_1,
            reader.node_by_id(NODE_ID_2).unwrap().parent().unwrap().id()
        );
        assert_eq!(
            NODE_ID_1,
            reader.node_by_id(NODE_ID_3).unwrap().parent().unwrap().id()
        );
        assert_eq!(2, reader.root().children().count());
    }

    #[test]
    fn add_child_to_root_node() {
        let root_node = Node::new(NODE_ID_1, Role::Window);
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![root_node.clone()],
            tree: Some(Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8)),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(first_update);
        assert_eq!(0, tree.read().root().children().count());
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: Box::new([NODE_ID_2]),
                    ..root_node
                },
                Node::new(NODE_ID_2, Role::RootWebArea),
            ],
            tree: None,
            root: None,
        };
        let mut got_updated_root_node = false;
        let mut got_new_child_node = false;
        tree.update_and_process_changes(second_update, |change| {
            if let super::Change::NodeUpdated { old_data, new_node } = &change {
                if new_node.id() == NODE_ID_1
                    && old_data.children == Box::new([])
                    && new_node.data().children == Box::new([NODE_ID_2])
                {
                    got_updated_root_node = true;
                    return;
                }
            }
            if let super::Change::NodeAdded(node) = &change {
                if node.id() == NODE_ID_2 {
                    got_new_child_node = true;
                    return;
                }
            }
            panic!("expected only new child node and updated root node");
        });
        assert!(got_updated_root_node);
        assert!(got_new_child_node);
        let reader = tree.read();
        assert_eq!(1, reader.root().children().count());
        assert_eq!(NODE_ID_2, reader.root().children().next().unwrap().id());
        assert_eq!(
            NODE_ID_1,
            reader.node_by_id(NODE_ID_2).unwrap().parent().unwrap().id()
        );
    }

    #[test]
    fn remove_child_from_root_node() {
        let root_node = Node::new(NODE_ID_1, Role::Window);
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: Box::new([NODE_ID_2]),
                    ..root_node.clone()
                },
                Node::new(NODE_ID_2, Role::RootWebArea),
            ],
            tree: Some(Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8)),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(first_update);
        assert_eq!(1, tree.read().root().children().count());
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![root_node],
            tree: None,
            root: None,
        };
        let mut got_updated_root_node = false;
        let mut got_removed_child_node = false;
        tree.update_and_process_changes(second_update, |change| {
            if let super::Change::NodeUpdated { old_data, new_node } = &change {
                if new_node.id() == NODE_ID_1
                    && old_data.children == Box::new([NODE_ID_2])
                    && new_node.data().children == Box::new([])
                {
                    got_updated_root_node = true;
                    return;
                }
            }
            if let super::Change::NodeRemoved(old_data) = &change {
                if old_data.id == NODE_ID_2 {
                    got_removed_child_node = true;
                    return;
                }
            }
            panic!("expected only removed child node and updated root node");
        });
        assert!(got_updated_root_node);
        assert!(got_removed_child_node);
        assert_eq!(0, tree.read().root().children().count());
        assert!(tree.read().node_by_id(NODE_ID_2).is_none());
    }

    #[test]
    fn move_focus_between_siblings() {
        let tree_data = Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8);
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: Box::new([NODE_ID_2, NODE_ID_3]),
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node::new(NODE_ID_2, Role::Button),
                Node::new(NODE_ID_3, Role::Button),
            ],
            tree: Some(Tree {
                focus: Some(NODE_ID_2),
                ..tree_data.clone()
            }),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(first_update);
        assert!(tree.read().node_by_id(NODE_ID_2).unwrap().is_focused());
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![],
            tree: Some(Tree {
                focus: Some(NODE_ID_3),
                ..tree_data
            }),
            root: None,
        };
        let mut got_focus_change = false;
        tree.update_and_process_changes(second_update, |change| {
            if let super::Change::FocusMoved {
                old_id,
                new_node: Some(new_node),
            } = &change
            {
                if *old_id == Some(NODE_ID_2) && new_node.id() == NODE_ID_3 {
                    got_focus_change = true;
                    return;
                }
            }
            panic!("expected only focus change");
        });
        assert!(got_focus_change);
        assert!(tree.read().node_by_id(NODE_ID_3).unwrap().is_focused());
        assert!(!tree.read().node_by_id(NODE_ID_2).unwrap().is_focused());
    }

    #[test]
    fn update_node() {
        let tree_data = Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8);
        let child_node = Node::new(NODE_ID_2, Role::Button);
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: Box::new([NODE_ID_2]),
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node {
                    name: Some("foo".into()),
                    ..child_node.clone()
                },
            ],
            tree: Some(tree_data),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(first_update);
        assert_eq!(
            Some("foo"),
            tree.read().node_by_id(NODE_ID_2).unwrap().name()
        );
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![Node {
                name: Some("bar".into()),
                ..child_node
            }],
            tree: None,
            root: None,
        };
        let mut got_updated_child_node = false;
        tree.update_and_process_changes(second_update, |change| {
            if let super::Change::NodeUpdated { old_data, new_node } = &change {
                if new_node.id() == NODE_ID_2
                    && old_data.name == Some("foo".into())
                    && new_node.name() == Some("bar")
                {
                    got_updated_child_node = true;
                    return;
                }
            }
            panic!("expected only updated child node");
        });
        assert!(got_updated_child_node);
        assert_eq!(
            Some("bar"),
            tree.read().node_by_id(NODE_ID_2).unwrap().name()
        );
    }

    // Verify that if an update consists entirely of node data and tree data
    // that's the same as before, no changes are reported. This would be useful
    // for a provider that constructs a fresh tree every time, such as
    // an immediate-mode GUI.
    #[test]
    fn no_change_update() {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: Box::new([NODE_ID_2, NODE_ID_3]),
                    ..Node::new(NODE_ID_1, Role::Window)
                },
                Node::new(NODE_ID_2, Role::Button),
                Node::new(NODE_ID_3, Role::Button),
            ],
            tree: Some(Tree::new(TreeId(TREE_ID.into()), StringEncoding::Utf8)),
            root: Some(NODE_ID_1),
        };
        let tree = super::Tree::new(update.clone());
        tree.update_and_process_changes(update, |_| {
            panic!("expected no changes");
        });
    }
}
