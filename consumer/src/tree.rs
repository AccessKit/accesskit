// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Live, Node as NodeData, NodeId, Tree as TreeData, TreeUpdate};
use std::collections::{HashMap, HashSet};

use crate::node::{DetachedNode, Node, NodeState, ParentAndIndex};

#[derive(Clone)]
pub struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) data: TreeData,
    focus: NodeId,
    is_host_focused: bool,
}

struct InternalFocusChange {
    old_focus: Option<DetachedNode>,
    new_focus_old_node: Option<DetachedNode>,
}

#[derive(Default)]
struct InternalChanges {
    added_node_ids: HashSet<NodeId>,
    updated_nodes: HashMap<NodeId, DetachedNode>,
    focus_change: Option<InternalFocusChange>,
    removed_nodes: HashMap<NodeId, DetachedNode>,
}

impl State {
    fn validate_global(&self) {
        assert!(self.nodes.contains_key(&self.data.root));
        assert!(self.nodes.contains_key(&self.focus));
    }

    fn update(
        &mut self,
        update: TreeUpdate,
        is_host_focused: bool,
        mut changes: Option<&mut InternalChanges>,
    ) {
        // First, if we're collecting changes, get the accurate state
        // of any updated nodes.
        if let Some(changes) = &mut changes {
            for (node_id, _) in &update.nodes {
                if let Some(old_node) = self.node_by_id(*node_id) {
                    let old_node = old_node.detached();
                    changes.updated_nodes.insert(*node_id, old_node);
                }
            }
        }

        let mut orphans = HashSet::new();
        let old_focus_id = self.is_host_focused.then_some(self.focus);
        let old_root_id = self.data.root;

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
            nodes: &mut HashMap<NodeId, NodeState>,
            changes: &mut Option<&mut InternalChanges>,
            parent_and_index: Option<ParentAndIndex>,
            id: NodeId,
            data: NodeData,
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
            for (child_index, child_id) in node_data.children().iter().enumerate() {
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
                    node_state.parent_and_index = None;
                }
                for child_id in node_state.data.children().iter() {
                    if !seen_child_ids.contains(child_id) {
                        orphans.insert(*child_id);
                    }
                }
                node_state.data = node_data;
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

        if update.focus != self.focus || is_host_focused != self.is_host_focused {
            let old_focus = old_focus_id.map(|id| self.node_by_id(id).unwrap().detached());
            let new_focus = is_host_focused.then_some(update.focus);
            if let Some(changes) = &mut changes {
                changes.focus_change = Some(InternalFocusChange {
                    old_focus,
                    new_focus_old_node: new_focus
                        .and_then(|id| {
                            (!changes.updated_nodes.contains_key(&id))
                                .then(|| self.node_by_id(id).map(|node| node.detached()))
                        })
                        .flatten(),
                });
            }
            self.focus = update.focus;
            self.is_host_focused = is_host_focused;
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
                for child_id in node.data.children().iter() {
                    traverse_orphan(nodes, to_remove, *child_id);
                }
            }

            for id in orphans {
                traverse_orphan(&self.nodes, &mut to_remove, id);
            }

            for id in to_remove {
                if let Some(old_node_state) = self.nodes.remove(&id) {
                    if let Some(changes) = &mut changes {
                        let old_node = DetachedNode {
                            state: old_node_state,
                            is_focused: old_focus_id == Some(id),
                            is_root: old_root_id == id,
                            name: None,
                            value: None,
                            live: Live::Off,
                            supports_text_ranges: false,
                        };
                        changes.removed_nodes.insert(id, old_node);
                    }
                }
            }
        }

        self.validate_global();
    }

    fn update_host_focus_state(
        &mut self,
        is_host_focused: bool,
        changes: Option<&mut InternalChanges>,
    ) {
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: self.focus,
        };
        self.update(update, is_host_focused, changes);
    }

    pub fn serialize(&self) -> TreeUpdate {
        let mut nodes = Vec::new();

        fn traverse(state: &State, nodes: &mut Vec<(NodeId, NodeData)>, id: NodeId) {
            let node = state.nodes.get(&id).unwrap();
            nodes.push((id, node.data.clone()));

            for child_id in node.data.children().iter() {
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
        self.is_host_focused.then_some(self.focus)
    }

    pub fn focus(&self) -> Option<Node<'_>> {
        self.focus_id().map(|id| self.node_by_id(id).unwrap())
    }

    pub fn app_name(&self) -> Option<String> {
        self.data.app_name.clone()
    }

    pub fn toolkit_name(&self) -> Option<String> {
        self.data.toolkit_name.clone()
    }

    pub fn toolkit_version(&self) -> Option<String> {
        self.data.toolkit_version.clone()
    }
}

pub trait ChangeHandler {
    fn node_added(&mut self, node: &Node);
    fn node_updated(&mut self, old_node: &DetachedNode, new_node: &Node);
    fn focus_moved(
        &mut self,
        old_node: Option<&DetachedNode>,
        new_node: Option<&Node>,
        current_state: &State,
    );
    /// The tree update process doesn't currently collect all possible information
    /// about removed nodes. The following methods don't accurately reflect
    /// the full state of the old node:
    ///
    /// * [`DetachedNode::name`]
    /// * [`DetachedNode::live`]
    /// * [`DetachedNode::supports_text_ranges`]
    fn node_removed(&mut self, node: &DetachedNode, current_state: &State);
}

pub struct Tree {
    state: State,
}

impl Tree {
    pub fn new(mut initial_state: TreeUpdate, is_host_focused: bool) -> Self {
        let mut state = State {
            nodes: HashMap::new(),
            data: initial_state.tree.take().unwrap(),
            focus: initial_state.focus,
            is_host_focused,
        };
        state.update(initial_state, is_host_focused, None);
        Self { state }
    }

    pub fn update(&mut self, update: TreeUpdate) {
        self.state.update(update, self.state.is_host_focused, None);
    }

    pub fn update_and_process_changes(
        &mut self,
        update: TreeUpdate,
        handler: &mut impl ChangeHandler,
    ) {
        let mut changes = InternalChanges::default();
        self.state
            .update(update, self.state.is_host_focused, Some(&mut changes));
        self.process_changes(changes, handler);
    }

    pub fn update_host_focus_state(&mut self, is_host_focused: bool) {
        self.state.update_host_focus_state(is_host_focused, None);
    }

    pub fn update_host_focus_state_and_process_changes(
        &mut self,
        is_host_focused: bool,
        handler: &mut impl ChangeHandler,
    ) {
        let mut changes = InternalChanges::default();
        self.state
            .update_host_focus_state(is_host_focused, Some(&mut changes));
        self.process_changes(changes, handler);
    }

    fn process_changes(&self, changes: InternalChanges, handler: &mut impl ChangeHandler) {
        for id in &changes.added_node_ids {
            let node = self.state.node_by_id(*id).unwrap();
            handler.node_added(&node);
        }
        for (id, old_node) in &changes.updated_nodes {
            let new_node = self.state.node_by_id(*id).unwrap();
            handler.node_updated(old_node, &new_node);
        }
        if let Some(focus_change) = changes.focus_change {
            if let Some(old_node) = &focus_change.old_focus {
                let id = old_node.id();
                if !changes.updated_nodes.contains_key(&id)
                    && !changes.removed_nodes.contains_key(&id)
                {
                    if let Some(old_node_new_version) = self.state.node_by_id(id) {
                        handler.node_updated(old_node, &old_node_new_version);
                    }
                }
            }
            let new_node = self.state.focus();
            if let Some(new_node) = new_node {
                let id = new_node.id();
                if !changes.added_node_ids.contains(&id) && !changes.updated_nodes.contains_key(&id)
                {
                    if let Some(new_node_old_version) = focus_change.new_focus_old_node {
                        handler.node_updated(&new_node_old_version, &new_node);
                    }
                }
            }
            handler.focus_moved(
                focus_change.old_focus.as_ref(),
                new_node.as_ref(),
                &self.state,
            );
        }
        for node in changes.removed_nodes.values() {
            handler.node_removed(node, &self.state);
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use accesskit::{NodeBuilder, NodeClassSet, NodeId, Role, Tree, TreeUpdate};

    #[test]
    fn init_tree_with_root_node() {
        let mut classes = NodeClassSet::new();
        let update = TreeUpdate {
            nodes: vec![(
                NodeId(0),
                NodeBuilder::new(Role::Window).build(&mut classes),
            )],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);
        assert_eq!(NodeId(0), tree.state().root().id());
        assert_eq!(Role::Window, tree.state().root().role());
        assert!(tree.state().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let mut classes = NodeClassSet::new();
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut builder = NodeBuilder::new(Role::Window);
                    builder.set_children(vec![NodeId(1), NodeId(2)]);
                    builder.build(&mut classes)
                }),
                (
                    NodeId(1),
                    NodeBuilder::new(Role::Button).build(&mut classes),
                ),
                (
                    NodeId(2),
                    NodeBuilder::new(Role::Button).build(&mut classes),
                ),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);
        let state = tree.state();
        assert_eq!(
            NodeId(0),
            state.node_by_id(NodeId(1)).unwrap().parent().unwrap().id()
        );
        assert_eq!(
            NodeId(0),
            state.node_by_id(NodeId(2)).unwrap().parent().unwrap().id()
        );
        assert_eq!(2, state.root().children().count());
    }

    #[test]
    fn add_child_to_root_node() {
        let mut classes = NodeClassSet::new();
        let root_builder = NodeBuilder::new(Role::Window);
        let first_update = TreeUpdate {
            nodes: vec![(NodeId(0), root_builder.clone().build(&mut classes))],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(0, tree.state().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut builder = root_builder;
                    builder.push_child(NodeId(1));
                    builder.build(&mut classes)
                }),
                (
                    NodeId(1),
                    NodeBuilder::new(Role::RootWebArea).build(&mut classes),
                ),
            ],
            tree: None,
            focus: NodeId(0),
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
                if node.id() == NodeId(1) {
                    self.got_new_child_node = true;
                    return;
                }
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::DetachedNode, new_node: &crate::Node) {
                if new_node.id() == NodeId(0)
                    && old_node.data().children().is_empty()
                    && new_node.data().children() == [NodeId(1)]
                {
                    self.got_updated_root_node = true;
                    return;
                }
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&crate::DetachedNode>,
                _new_node: Option<&crate::Node>,
                _current_state: &crate::TreeState,
            ) {
                unexpected_change();
            }
            fn node_removed(
                &mut self,
                _node: &crate::DetachedNode,
                _current_state: &crate::TreeState,
            ) {
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
        let state = tree.state();
        assert_eq!(1, state.root().children().count());
        assert_eq!(NodeId(1), state.root().children().next().unwrap().id());
        assert_eq!(
            NodeId(0),
            state.node_by_id(NodeId(1)).unwrap().parent().unwrap().id()
        );
    }

    #[test]
    fn remove_child_from_root_node() {
        let mut classes = NodeClassSet::new();
        let root_builder = NodeBuilder::new(Role::Window);
        let first_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut builder = root_builder.clone();
                    builder.push_child(NodeId(1));
                    builder.build(&mut classes)
                }),
                (
                    NodeId(1),
                    NodeBuilder::new(Role::RootWebArea).build(&mut classes),
                ),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(1, tree.state().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![(NodeId(0), root_builder.build(&mut classes))],
            tree: None,
            focus: NodeId(0),
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
            fn node_updated(&mut self, old_node: &crate::DetachedNode, new_node: &crate::Node) {
                if new_node.id() == NodeId(0)
                    && old_node.data().children() == [NodeId(1)]
                    && new_node.data().children().is_empty()
                {
                    self.got_updated_root_node = true;
                    return;
                }
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&crate::DetachedNode>,
                _new_node: Option<&crate::Node>,
                _current_state: &crate::TreeState,
            ) {
                unexpected_change();
            }
            fn node_removed(
                &mut self,
                node: &crate::DetachedNode,
                _current_state: &crate::TreeState,
            ) {
                if node.id() == NodeId(1) {
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
        assert_eq!(0, tree.state().root().children().count());
        assert!(tree.state().node_by_id(NodeId(1)).is_none());
    }

    #[test]
    fn move_focus_between_siblings() {
        let mut classes = NodeClassSet::new();
        let first_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut builder = NodeBuilder::new(Role::Window);
                    builder.set_children(vec![NodeId(1), NodeId(2)]);
                    builder.build(&mut classes)
                }),
                (
                    NodeId(1),
                    NodeBuilder::new(Role::Button).build(&mut classes),
                ),
                (
                    NodeId(2),
                    NodeBuilder::new(Role::Button).build(&mut classes),
                ),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(1),
        };
        let mut tree = super::Tree::new(first_update, true);
        assert!(tree.state().node_by_id(NodeId(1)).unwrap().is_focused());
        let second_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: NodeId(2),
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
            fn node_updated(&mut self, old_node: &crate::DetachedNode, new_node: &crate::Node) {
                if old_node.id() == NodeId(1)
                    && new_node.id() == NodeId(1)
                    && old_node.is_focused()
                    && !new_node.is_focused()
                {
                    self.got_old_focus_node_update = true;
                    return;
                }
                if old_node.id() == NodeId(2)
                    && new_node.id() == NodeId(2)
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
                old_node: Option<&crate::DetachedNode>,
                new_node: Option<&crate::Node>,
                _current_state: &crate::TreeState,
            ) {
                if let (Some(old_node), Some(new_node)) = (old_node, new_node) {
                    if old_node.id() == NodeId(1) && new_node.id() == NodeId(2) {
                        self.got_focus_change = true;
                        return;
                    }
                }
                unexpected_change();
            }
            fn node_removed(
                &mut self,
                _node: &crate::DetachedNode,
                _current_state: &crate::TreeState,
            ) {
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
        assert!(tree.state().node_by_id(NodeId(2)).unwrap().is_focused());
        assert!(!tree.state().node_by_id(NodeId(1)).unwrap().is_focused());
    }

    #[test]
    fn update_node() {
        let mut classes = NodeClassSet::new();
        let child_builder = NodeBuilder::new(Role::Button);
        let first_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut builder = NodeBuilder::new(Role::Window);
                    builder.set_children(vec![NodeId(1)]);
                    builder.build(&mut classes)
                }),
                (NodeId(1), {
                    let mut builder = child_builder.clone();
                    builder.set_name("foo");
                    builder.build(&mut classes)
                }),
            ],
            tree: Some(Tree::new(NodeId(0))),
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(
            Some("foo".into()),
            tree.state().node_by_id(NodeId(1)).unwrap().name()
        );
        let second_update = TreeUpdate {
            nodes: vec![(NodeId(1), {
                let mut builder = child_builder;
                builder.set_name("bar");
                builder.build(&mut classes)
            })],
            tree: None,
            focus: NodeId(0),
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
            fn node_updated(&mut self, old_node: &crate::DetachedNode, new_node: &crate::Node) {
                if new_node.id() == NodeId(1)
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
                _old_node: Option<&crate::DetachedNode>,
                _new_node: Option<&crate::Node>,
                _current_state: &crate::TreeState,
            ) {
                unexpected_change();
            }
            fn node_removed(
                &mut self,
                _node: &crate::DetachedNode,
                _current_state: &crate::TreeState,
            ) {
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
            tree.state().node_by_id(NodeId(1)).unwrap().name()
        );
    }
}
