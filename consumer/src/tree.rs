// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, Node as NodeData, NodeId, Tree as TreeData,
    TreeUpdate,
};
use parking_lot::{RwLock, RwLockWriteGuard};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use crate::{text::Position as TextPosition, Node};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParentAndIndex(pub(crate) NodeId, pub(crate) usize);

#[derive(Clone)]
pub(crate) struct NodeState {
    pub(crate) id: NodeId,
    pub(crate) parent_and_index: Option<ParentAndIndex>,
    pub(crate) data: Arc<NodeData>,
}

#[derive(Clone)]
pub struct State {
    pub(crate) nodes: im::HashMap<NodeId, NodeState>,
    pub(crate) data: TreeData,
    pub(crate) focus: Option<NodeId>,
}

#[derive(Default)]
struct InternalChanges {
    added_node_ids: HashSet<NodeId>,
    updated_node_ids: HashSet<NodeId>,
    focus_moved: bool,
    removed_node_ids: HashSet<NodeId>,
}

impl State {
    fn validate_global(&self) {
        assert!(self.nodes.contains_key(&self.data.root));
        if let Some(id) = self.focus {
            assert!(self.nodes.contains_key(&id));
        }
        if let Some(id) = self.data.root_scroller {
            assert!(self.nodes.contains_key(&id));
        }
    }

    fn update(&mut self, update: TreeUpdate, mut changes: Option<&mut InternalChanges>) {
        let mut orphans = HashSet::new();

        if let Some(tree) = update.tree {
            if tree.root != self.data.root {
                orphans.insert(self.data.root);
            }
            self.data = tree;
        }

        let root = self.data.root;
        let mut pending_nodes: HashMap<NodeId, _> = HashMap::new();
        let mut pending_children = HashMap::new();

        fn add_node(
            nodes: &mut im::HashMap<NodeId, NodeState>,
            changes: &mut Option<&mut InternalChanges>,
            parent_and_index: Option<ParentAndIndex>,
            id: NodeId,
            data: Arc<NodeData>,
        ) {
            let state = NodeState {
                id,
                parent_and_index,
                data,
            };
            nodes.insert(id, state);
            if let Some(changes) = changes {
                changes.added_node_ids.insert(id);
            }
        }

        for (node_id, node_data) in update.nodes {
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
                        *child_id,
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
                node_state.data = node_data;
                if let Some(changes) = &mut changes {
                    changes.updated_node_ids.insert(node_id);
                }
            } else if let Some(parent_and_index) = pending_children.remove(&node_id) {
                add_node(
                    &mut self.nodes,
                    &mut changes,
                    Some(parent_and_index),
                    node_id,
                    node_data,
                );
            } else if node_id == root {
                add_node(&mut self.nodes, &mut changes, None, node_id, node_data);
            } else {
                pending_nodes.insert(node_id, node_data);
            }
        }

        assert_eq!(pending_nodes.len(), 0);
        assert_eq!(pending_children.len(), 0);

        if update.focus != self.focus {
            if let Some(changes) = &mut changes {
                changes.focus_moved = true;
            }
            self.focus = update.focus;
        }

        if !orphans.is_empty() {
            let mut to_remove = HashSet::new();

            fn traverse_orphan(
                nodes: &im::HashMap<NodeId, NodeState>,
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
                if self.nodes.remove(&id).is_some() {
                    if let Some(changes) = &mut changes {
                        changes.removed_node_ids.insert(id);
                    }
                }
            }
        }

        self.validate_global();
    }

    pub fn serialize(&self) -> TreeUpdate {
        let mut nodes = Vec::new();

        fn traverse(state: &State, nodes: &mut Vec<(NodeId, Arc<NodeData>)>, id: NodeId) {
            let node = state.nodes.get(&id).unwrap();
            nodes.push((id, Arc::clone(&node.data)));

            for child_id in node.data.children.iter() {
                traverse(state, nodes, *child_id);
            }
        }

        traverse(self, &mut nodes, self.data.root);
        assert_eq!(nodes.len(), self.nodes.len());

        TreeUpdate {
            nodes,
            tree: Some(self.data.clone()),
            focus: self.focus,
        }
    }

    pub fn has_node(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn node_by_id(&self, id: NodeId) -> Option<Node<'_>> {
        self.nodes.get(&id).map(|node_state| Node {
            tree_state: self,
            state: node_state,
        })
    }

    pub fn root_id(&self) -> NodeId {
        self.data.root
    }

    pub fn root(&self) -> Node<'_> {
        self.node_by_id(self.root_id()).unwrap()
    }

    pub fn focus_id(&self) -> Option<NodeId> {
        self.focus
    }

    pub fn focus(&self) -> Option<Node<'_>> {
        self.focus.map(|id| self.node_by_id(id).unwrap())
    }
}

pub trait ChangeHandler {
    fn node_added(&mut self, node: &Node);
    fn node_updated(&mut self, old_node: &Node, new_node: &Node);
    fn focus_moved(&mut self, old_node: Option<&Node>, new_node: Option<&Node>);
    fn node_removed(&mut self, node: &Node);
}

pub struct Tree {
    state: RwLock<State>,
    pub(crate) action_handler: Box<dyn ActionHandler>,
}

impl Tree {
    pub fn new(mut initial_state: TreeUpdate, action_handler: Box<dyn ActionHandler>) -> Self {
        let mut state = State {
            nodes: im::HashMap::new(),
            data: initial_state.tree.take().unwrap(),
            focus: None,
        };
        state.update(initial_state, None);
        Self {
            state: RwLock::new(state),
            action_handler,
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        let mut state = self.state.write();
        state.update(update, None);
    }

    pub fn update_and_process_changes(&self, update: TreeUpdate, handler: &mut impl ChangeHandler) {
        let mut changes = InternalChanges::default();
        let mut state = self.state.write();
        let old_state = state.clone();
        state.update(update, Some(&mut changes));
        let state = RwLockWriteGuard::downgrade(state);
        for id in &changes.added_node_ids {
            let node = state.node_by_id(*id).unwrap();
            handler.node_added(&node);
        }
        for id in &changes.updated_node_ids {
            let old_node = old_state.node_by_id(*id).unwrap();
            let new_node = state.node_by_id(*id).unwrap();
            handler.node_updated(&old_node, &new_node);
        }
        if changes.focus_moved {
            let old_node = old_state.focus();
            if let Some(old_node) = old_node {
                let id = old_node.id();
                if !changes.updated_node_ids.contains(&id)
                    && !changes.removed_node_ids.contains(&id)
                {
                    if let Some(old_node_new_version) = state.node_by_id(id) {
                        handler.node_updated(&old_node, &old_node_new_version);
                    }
                }
            }
            let new_node = state.focus();
            if let Some(new_node) = new_node {
                let id = new_node.id();
                if !changes.added_node_ids.contains(&id) && !changes.updated_node_ids.contains(&id)
                {
                    if let Some(new_node_old_version) = old_state.node_by_id(id) {
                        handler.node_updated(&new_node_old_version, &new_node);
                    }
                }
            }
            handler.focus_moved(old_node.as_ref(), new_node.as_ref());
        }
        for id in &changes.removed_node_ids {
            let node = old_state.node_by_id(*id).unwrap();
            handler.node_removed(&node);
        }
    }

    pub fn read(&self) -> impl Deref<Target = State> + '_ {
        self.state.read()
    }

    pub fn set_focus(&self, target: NodeId) {
        self.action_handler.do_action(ActionRequest {
            action: Action::Focus,
            target,
            data: None,
        })
    }

    pub fn do_default_action(&self, target: NodeId) {
        self.action_handler.do_action(ActionRequest {
            action: Action::Default,
            target,
            data: None,
        })
    }

    pub fn set_value(&self, target: NodeId, value: impl Into<Box<str>>) {
        self.action_handler.do_action(ActionRequest {
            action: Action::SetValue,
            target,
            data: Some(ActionData::Value(value.into())),
        })
    }

    pub fn set_numeric_value(&self, target: NodeId, value: f64) {
        self.action_handler.do_action(ActionRequest {
            action: Action::SetValue,
            target,
            data: Some(ActionData::NumericValue(value)),
        })
    }

    pub fn scroll_into_view(&self, target: NodeId) {
        self.action_handler.do_action(ActionRequest {
            action: Action::ScrollIntoView,
            target,
            data: None,
        })
    }

    pub fn scroll_text_position_into_view(&self, target: &TextPosition) {
        self.action_handler.do_action(ActionRequest {
            action: Action::ScrollIntoView,
            target: target.inner.node.id(),
            data: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
    use std::{num::NonZeroU128, sync::Arc};

    use crate::tests::NullActionHandler;

    const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
    const NODE_ID_2: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
    const NODE_ID_3: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });

    #[test]
    fn init_tree_with_root_node() {
        let update = TreeUpdate {
            nodes: vec![(
                NODE_ID_1,
                Arc::new(Node {
                    role: Role::Window,
                    ..Node::default()
                }),
            )],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = super::Tree::new(update, Box::new(NullActionHandler {}));
        assert_eq!(NODE_ID_1, tree.read().root().id());
        assert_eq!(Role::Window, tree.read().root().role());
        assert!(tree.read().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        role: Role::Window,
                        children: vec![NODE_ID_2, NODE_ID_3],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::Button,
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_3,
                    Arc::new(Node {
                        role: Role::Button,
                        ..Default::default()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = super::Tree::new(update, Box::new(NullActionHandler {}));
        let state = tree.read();
        assert_eq!(
            NODE_ID_1,
            state.node_by_id(NODE_ID_2).unwrap().parent().unwrap().id()
        );
        assert_eq!(
            NODE_ID_1,
            state.node_by_id(NODE_ID_3).unwrap().parent().unwrap().id()
        );
        assert_eq!(2, state.root().children().count());
    }

    #[test]
    fn add_child_to_root_node() {
        let root_node = Node {
            role: Role::Window,
            ..Default::default()
        };
        let first_update = TreeUpdate {
            nodes: vec![(NODE_ID_1, Arc::new(root_node.clone()))],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = super::Tree::new(first_update, Box::new(NullActionHandler {}));
        assert_eq!(0, tree.read().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        children: vec![NODE_ID_2],
                        ..root_node
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::RootWebArea,
                        ..Default::default()
                    }),
                ),
            ],
            tree: None,
            focus: None,
        };
        struct Handler {
            got_new_child_node: bool,
            got_updated_root_node: bool,
        }
        fn unexpected_change() {
            panic!("expected only new child node and updated root node");
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                if node.id() == NODE_ID_2 {
                    self.got_new_child_node = true;
                    return;
                }
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == NODE_ID_1
                    && old_node.data().children == vec![]
                    && new_node.data().children == vec![NODE_ID_2]
                {
                    self.got_updated_root_node = true;
                    return;
                }
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&crate::Node>,
                _new_node: Option<&crate::Node>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
        }
        let mut handler = Handler {
            got_new_child_node: false,
            got_updated_root_node: false,
        };
        tree.update_and_process_changes(second_update, &mut handler);
        assert!(handler.got_new_child_node);
        assert!(handler.got_updated_root_node);
        let state = tree.read();
        assert_eq!(1, state.root().children().count());
        assert_eq!(NODE_ID_2, state.root().children().next().unwrap().id());
        assert_eq!(
            NODE_ID_1,
            state.node_by_id(NODE_ID_2).unwrap().parent().unwrap().id()
        );
    }

    #[test]
    fn remove_child_from_root_node() {
        let root_node = Node {
            role: Role::Window,
            ..Default::default()
        };
        let first_update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        children: vec![NODE_ID_2],
                        ..root_node.clone()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::RootWebArea,
                        ..Default::default()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = super::Tree::new(first_update, Box::new(NullActionHandler {}));
        assert_eq!(1, tree.read().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![(NODE_ID_1, Arc::new(root_node))],
            tree: None,
            focus: None,
        };
        struct Handler {
            got_updated_root_node: bool,
            got_removed_child_node: bool,
        }
        fn unexpected_change() {
            panic!("expected only removed child node and updated root node");
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == NODE_ID_1
                    && old_node.data().children == vec![NODE_ID_2]
                    && new_node.data().children == vec![]
                {
                    self.got_updated_root_node = true;
                    return;
                }
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&crate::Node>,
                _new_node: Option<&crate::Node>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, node: &crate::Node) {
                if node.id() == NODE_ID_2 {
                    self.got_removed_child_node = true;
                    return;
                }
                unexpected_change();
            }
        }
        let mut handler = Handler {
            got_updated_root_node: false,
            got_removed_child_node: false,
        };
        tree.update_and_process_changes(second_update, &mut handler);
        assert!(handler.got_updated_root_node);
        assert!(handler.got_removed_child_node);
        assert_eq!(0, tree.read().root().children().count());
        assert!(tree.read().node_by_id(NODE_ID_2).is_none());
    }

    #[test]
    fn move_focus_between_siblings() {
        let first_update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        role: Role::Window,
                        children: vec![NODE_ID_2, NODE_ID_3],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        role: Role::Button,
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_3,
                    Arc::new(Node {
                        role: Role::Button,
                        ..Default::default()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: Some(NODE_ID_2),
        };
        let tree = super::Tree::new(first_update, Box::new(NullActionHandler {}));
        assert!(tree.read().node_by_id(NODE_ID_2).unwrap().is_focused());
        let second_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: Some(NODE_ID_3),
        };
        struct Handler {
            got_old_focus_node_update: bool,
            got_new_focus_node_update: bool,
            got_focus_change: bool,
        }
        fn unexpected_change() {
            panic!("expected only focus change");
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if old_node.id() == NODE_ID_2
                    && new_node.id() == NODE_ID_2
                    && old_node.is_focused()
                    && !new_node.is_focused()
                {
                    self.got_old_focus_node_update = true;
                    return;
                }
                if old_node.id() == NODE_ID_3
                    && new_node.id() == NODE_ID_3
                    && !old_node.is_focused()
                    && new_node.is_focused()
                {
                    self.got_new_focus_node_update = true;
                    return;
                }
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                old_node: Option<&crate::Node>,
                new_node: Option<&crate::Node>,
            ) {
                if let (Some(old_node), Some(new_node)) = (old_node, new_node) {
                    if old_node.id() == NODE_ID_2 && new_node.id() == NODE_ID_3 {
                        self.got_focus_change = true;
                        return;
                    }
                }
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
        }
        let mut handler = Handler {
            got_old_focus_node_update: false,
            got_new_focus_node_update: false,
            got_focus_change: false,
        };
        tree.update_and_process_changes(second_update, &mut handler);
        assert!(handler.got_old_focus_node_update);
        assert!(handler.got_new_focus_node_update);
        assert!(handler.got_focus_change);
        assert!(tree.read().node_by_id(NODE_ID_3).unwrap().is_focused());
        assert!(!tree.read().node_by_id(NODE_ID_2).unwrap().is_focused());
    }

    #[test]
    fn update_node() {
        let child_node = Node {
            role: Role::Button,
            ..Default::default()
        };
        let first_update = TreeUpdate {
            nodes: vec![
                (
                    NODE_ID_1,
                    Arc::new(Node {
                        role: Role::Window,
                        children: vec![NODE_ID_2],
                        ..Default::default()
                    }),
                ),
                (
                    NODE_ID_2,
                    Arc::new(Node {
                        name: Some("foo".into()),
                        ..child_node.clone()
                    }),
                ),
            ],
            tree: Some(Tree::new(NODE_ID_1)),
            focus: None,
        };
        let tree = super::Tree::new(first_update, Box::new(NullActionHandler {}));
        assert_eq!(
            Some("foo".into()),
            tree.read().node_by_id(NODE_ID_2).unwrap().name()
        );
        let second_update = TreeUpdate {
            nodes: vec![(
                NODE_ID_2,
                Arc::new(Node {
                    name: Some("bar".into()),
                    ..child_node
                }),
            )],
            tree: None,
            focus: None,
        };
        struct Handler {
            got_updated_child_node: bool,
        }
        fn unexpected_change() {
            panic!("expected only updated child node");
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == NODE_ID_2
                    && old_node.name() == Some("foo".into())
                    && new_node.name() == Some("bar".into())
                {
                    self.got_updated_child_node = true;
                    return;
                }
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&crate::Node>,
                _new_node: Option<&crate::Node>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
        }
        let mut handler = Handler {
            got_updated_child_node: false,
        };
        tree.update_and_process_changes(second_update, &mut handler);
        assert!(handler.got_updated_child_node);
        assert_eq!(
            Some("bar".into()),
            tree.read().node_by_id(NODE_ID_2).unwrap().name()
        );
    }
}
