// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Node as NodeData, NodeId, Tree as TreeData, TreeId, TreeUpdate};
use alloc::vec;
use core::fmt;
use hashbrown::{HashMap, HashSet};

use crate::node::{Node, NodeState, ParentAndIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct TreeIndex(pub(crate) u32);

#[derive(Debug, Default)]
struct TreeIndexMap {
    id_to_index: HashMap<TreeId, TreeIndex>,
    index_to_id: HashMap<TreeIndex, TreeId>,
    next: u32,
}

impl TreeIndexMap {
    fn get_index(&mut self, id: TreeId) -> TreeIndex {
        *self.id_to_index.entry(id).or_insert_with(|| {
            let tree_index = TreeIndex(self.next);
            self.next += 1;
            self.index_to_id.insert(tree_index, id);
            tree_index
        })
    }

    fn get_id(&self, index: TreeIndex) -> Option<TreeId> {
        self.index_to_id.get(&index).copied()
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) data: TreeData,
    pub(crate) focus: NodeId,
    is_host_focused: bool,
}

#[derive(Default)]
struct InternalChanges {
    added_node_ids: HashSet<NodeId>,
    updated_node_ids: HashSet<NodeId>,
    removed_node_ids: HashSet<NodeId>,
}

impl State {
    fn validate_global(&self) {
        if !self.nodes.contains_key(&self.data.root) {
            panic!("Root ID {:?} is not in the node list", self.data.root);
        }
        if !self.nodes.contains_key(&self.focus) {
            panic!("Focused ID {:?} is not in the node list", self.focus);
        }
    }

    fn update(
        &mut self,
        update: TreeUpdate,
        is_host_focused: bool,
        mut changes: Option<&mut InternalChanges>,
    ) {
        let mut unreachable = HashSet::new();
        let mut seen_child_ids = HashSet::new();

        if let Some(tree) = update.tree {
            if tree.root != self.data.root {
                unreachable.insert(self.data.root);
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
                parent_and_index,
                data,
            };
            nodes.insert(id, state);
            if let Some(changes) = changes {
                changes.added_node_ids.insert(id);
            }
        }

        for (node_id, node_data) in update.nodes {
            unreachable.remove(&node_id);

            for (child_index, child_id) in node_data.children().iter().enumerate() {
                if seen_child_ids.contains(child_id) {
                    panic!("TreeUpdate includes duplicate child {:?}", child_id);
                }
                seen_child_ids.insert(*child_id);
                unreachable.remove(child_id);
                let parent_and_index = ParentAndIndex(node_id, child_index);
                if let Some(child_state) = self.nodes.get_mut(child_id) {
                    if child_state.parent_and_index != Some(parent_and_index) {
                        child_state.parent_and_index = Some(parent_and_index);
                        if let Some(changes) = &mut changes {
                            changes.updated_node_ids.insert(*child_id);
                        }
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
            }

            if let Some(node_state) = self.nodes.get_mut(&node_id) {
                if node_id == root {
                    node_state.parent_and_index = None;
                }
                for child_id in node_state.data.children().iter() {
                    if !seen_child_ids.contains(child_id) {
                        unreachable.insert(*child_id);
                    }
                }
                if node_state.data != node_data {
                    node_state.data.clone_from(&node_data);
                    if let Some(changes) = &mut changes {
                        changes.updated_node_ids.insert(node_id);
                    }
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

        if !pending_nodes.is_empty() {
            panic!("TreeUpdate includes {} nodes which are neither in the current tree nor a child of another node from the update: {}", pending_nodes.len(), ShortNodeList(&pending_nodes));
        }
        if !pending_children.is_empty() {
            panic!("TreeUpdate's nodes include {} children ids which are neither in the current tree nor the ID of another node from the update: {}", pending_children.len(), ShortNodeList(&pending_children));
        }

        self.focus = update.focus;
        self.is_host_focused = is_host_focused;

        if !unreachable.is_empty() {
            fn traverse_unreachable(
                nodes: &mut HashMap<NodeId, NodeState>,
                changes: &mut Option<&mut InternalChanges>,
                seen_child_ids: &HashSet<NodeId>,
                id: NodeId,
            ) {
                if let Some(changes) = changes {
                    changes.removed_node_ids.insert(id);
                }
                let node = nodes.remove(&id).unwrap();
                for child_id in node.data.children().iter() {
                    if !seen_child_ids.contains(child_id) {
                        traverse_unreachable(nodes, changes, seen_child_ids, *child_id);
                    }
                }
            }

            for id in unreachable {
                traverse_unreachable(&mut self.nodes, &mut changes, &seen_child_ids, id);
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
            tree_id: TreeId::ROOT,
            focus: self.focus,
        };
        self.update(update, is_host_focused, changes);
    }

    pub fn has_node(&self, id: NodeId) -> bool {
        self.nodes.get(&id).is_some()
    }

    pub fn node_by_id(&self, id: NodeId) -> Option<Node<'_>> {
        self.nodes.get(&id).map(|node_state| Node {
            tree_state: self,
            id,
            state: node_state,
        })
    }

    pub fn root_id(&self) -> NodeId {
        self.data.root
    }

    pub fn root(&self) -> Node<'_> {
        self.node_by_id(self.root_id()).unwrap()
    }

    pub fn is_host_focused(&self) -> bool {
        self.is_host_focused
    }

    pub fn focus_id_in_tree(&self) -> NodeId {
        self.focus
    }

    pub fn focus_in_tree(&self) -> Node<'_> {
        self.node_by_id(self.focus_id_in_tree()).unwrap()
    }

    pub fn focus_id(&self) -> Option<NodeId> {
        self.is_host_focused.then_some(self.focus)
    }

    pub fn focus(&self) -> Option<Node<'_>> {
        self.focus_id().map(|id| self.node_by_id(id).unwrap())
    }

    pub fn active_dialog(&self) -> Option<Node<'_>> {
        let mut node = self.focus();
        while let Some(candidate) = node {
            if candidate.is_dialog() {
                return Some(candidate);
            }
            node = candidate.parent();
        }
        None
    }

    pub fn toolkit_name(&self) -> Option<&str> {
        self.data.toolkit_name.as_deref()
    }

    pub fn toolkit_version(&self) -> Option<&str> {
        self.data.toolkit_version.as_deref()
    }
}

pub trait ChangeHandler {
    fn node_added(&mut self, node: &Node);
    fn node_updated(&mut self, old_node: &Node, new_node: &Node);
    fn focus_moved(&mut self, old_node: Option<&Node>, new_node: Option<&Node>);
    fn node_removed(&mut self, node: &Node);
}

#[derive(Debug)]
pub struct Tree {
    state: State,
    next_state: State,
    tree_index_map: TreeIndexMap,
}

impl Tree {
    pub fn new(mut initial_state: TreeUpdate, is_host_focused: bool) -> Self {
        let Some(tree) = initial_state.tree.take() else {
            panic!("Tried to initialize the accessibility tree without a root tree. TreeUpdate::tree must be Some.");
        };
        if initial_state.tree_id != TreeId::ROOT {
            panic!("Cannot initialize with a subtree. TreeUpdate::tree_id must be TreeId::ROOT.");
        }
        let mut tree_index_map = TreeIndexMap::default();
        tree_index_map.get_index(initial_state.tree_id);
        let mut state = State {
            nodes: HashMap::new(),
            data: tree,
            focus: initial_state.focus,
            is_host_focused,
        };
        state.update(initial_state, is_host_focused, None);
        Self {
            next_state: state.clone(),
            state,
            tree_index_map,
        }
    }

    pub fn update_and_process_changes(
        &mut self,
        update: TreeUpdate,
        handler: &mut impl ChangeHandler,
    ) {
        let mut changes = InternalChanges::default();
        self.next_state
            .update(update, self.state.is_host_focused, Some(&mut changes));
        self.process_changes(changes, handler);
    }

    pub fn update_host_focus_state_and_process_changes(
        &mut self,
        is_host_focused: bool,
        handler: &mut impl ChangeHandler,
    ) {
        let mut changes = InternalChanges::default();
        self.next_state
            .update_host_focus_state(is_host_focused, Some(&mut changes));
        self.process_changes(changes, handler);
    }

    fn process_changes(&mut self, changes: InternalChanges, handler: &mut impl ChangeHandler) {
        for id in &changes.added_node_ids {
            let node = self.next_state.node_by_id(*id).unwrap();
            handler.node_added(&node);
        }
        for id in &changes.updated_node_ids {
            let old_node = self.state.node_by_id(*id).unwrap();
            let new_node = self.next_state.node_by_id(*id).unwrap();
            handler.node_updated(&old_node, &new_node);
        }
        if self.state.focus_id() != self.next_state.focus_id() {
            let old_node = self.state.focus();
            if let Some(old_node) = &old_node {
                let id = old_node.id();
                if !changes.updated_node_ids.contains(&id)
                    && !changes.removed_node_ids.contains(&id)
                {
                    if let Some(old_node_new_version) = self.next_state.node_by_id(id) {
                        handler.node_updated(old_node, &old_node_new_version);
                    }
                }
            }
            let new_node = self.next_state.focus();
            if let Some(new_node) = &new_node {
                let id = new_node.id();
                if !changes.added_node_ids.contains(&id) && !changes.updated_node_ids.contains(&id)
                {
                    if let Some(new_node_old_version) = self.state.node_by_id(id) {
                        handler.node_updated(&new_node_old_version, new_node);
                    }
                }
            }
            handler.focus_moved(old_node.as_ref(), new_node.as_ref());
        }
        for id in &changes.removed_node_ids {
            let node = self.state.node_by_id(*id).unwrap();
            handler.node_removed(&node);
        }
        for id in changes.added_node_ids {
            self.state
                .nodes
                .insert(id, self.next_state.nodes.get(&id).unwrap().clone());
        }
        for id in changes.updated_node_ids {
            self.state
                .nodes
                .get_mut(&id)
                .unwrap()
                .clone_from(self.next_state.nodes.get(&id).unwrap());
        }
        for id in changes.removed_node_ids {
            self.state.nodes.remove(&id);
        }
        if self.state.data != self.next_state.data {
            self.state.data.clone_from(&self.next_state.data);
        }
        self.state.focus = self.next_state.focus;
        self.state.is_host_focused = self.next_state.is_host_focused;
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

struct ShortNodeList<'a, T>(&'a HashMap<NodeId, T>);

impl<T> fmt::Display for ShortNodeList<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut iter = self.0.iter();
        for i in 0..10 {
            let Some((id, _)) = iter.next() else {
                break;
            };
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{id:?}")?;
        }
        if iter.next().is_some() {
            write!(f, " ...")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate, Uuid};
    use alloc::{vec, vec::Vec};

    use super::{TreeIndex, TreeIndexMap};

    #[test]
    fn tree_index_map_assigns_sequential_indices() {
        let mut map = TreeIndexMap::default();
        let id1 = TreeId::ROOT;
        let id2 = TreeId(Uuid::from_u128(1));
        let id3 = TreeId(Uuid::from_u128(2));

        let index1 = map.get_index(id1);
        let index2 = map.get_index(id2);
        let index3 = map.get_index(id3);

        assert_eq!(index1, TreeIndex(0));
        assert_eq!(index2, TreeIndex(1));
        assert_eq!(index3, TreeIndex(2));
    }

    #[test]
    fn tree_index_map_returns_same_index_for_same_id() {
        let mut map = TreeIndexMap::default();
        let id = TreeId::ROOT;

        let index1 = map.get_index(id);
        let index2 = map.get_index(id);

        assert_eq!(index1, index2);
    }

    #[test]
    fn tree_index_map_get_id_returns_correct_id() {
        let mut map = TreeIndexMap::default();
        let id1 = TreeId::ROOT;
        let id2 = TreeId(Uuid::from_u128(1));

        let index1 = map.get_index(id1);
        let index2 = map.get_index(id2);

        assert_eq!(map.get_id(index1), Some(id1));
        assert_eq!(map.get_id(index2), Some(id2));
    }

    #[test]
    fn tree_index_map_get_id_returns_none_for_unknown_index() {
        let map = TreeIndexMap::default();
        assert_eq!(map.get_id(TreeIndex(0)), None);
        assert_eq!(map.get_id(TreeIndex(999)), None);
    }

    #[test]
    fn init_tree_with_root_node() {
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);
        assert_eq!(NodeId(0), tree.state().root().id());
        assert_eq!(Role::Window, tree.state().root().role());
        assert!(tree.state().root().parent().is_none());
    }

    #[test]
    #[should_panic(
        expected = "Cannot initialize with a subtree. TreeUpdate::tree_id must be TreeId::ROOT."
    )]
    fn init_tree_with_non_root_tree_id_panics() {
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId(Uuid::from_u128(1)),
            focus: NodeId(0),
        };
        let _ = super::Tree::new(update, false);
    }

    #[test]
    fn root_node_has_children() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
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
        let root_node = Node::new(Role::Window);
        let first_update = TreeUpdate {
            nodes: vec![(NodeId(0), root_node.clone())],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(0, tree.state().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = root_node;
                    node.push_child(NodeId(1));
                    node
                }),
                (NodeId(1), Node::new(Role::RootWebArea)),
            ],
            tree: None,
            tree_id: TreeId::ROOT,
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
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
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
        let root_node = Node::new(Role::Window);
        let first_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = root_node.clone();
                    node.push_child(NodeId(1));
                    node
                }),
                (NodeId(1), Node::new(Role::RootWebArea)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(1, tree.state().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![(NodeId(0), root_node)],
            tree: None,
            tree_id: TreeId::ROOT,
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
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
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
                _old_node: Option<&crate::Node>,
                _new_node: Option<&crate::Node>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, node: &crate::Node) {
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
        let first_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let mut tree = super::Tree::new(first_update, true);
        assert!(tree.state().node_by_id(NodeId(1)).unwrap().is_focused());
        let second_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
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
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
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
                old_node: Option<&crate::Node>,
                new_node: Option<&crate::Node>,
            ) {
                if let (Some(old_node), Some(new_node)) = (old_node, new_node) {
                    if old_node.id() == NodeId(1) && new_node.id() == NodeId(2) {
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
        assert!(tree.state().node_by_id(NodeId(2)).unwrap().is_focused());
        assert!(!tree.state().node_by_id(NodeId(1)).unwrap().is_focused());
    }

    #[test]
    fn update_node() {
        let child_node = Node::new(Role::Button);
        let first_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = child_node.clone();
                    node.set_label("foo");
                    node
                }),
            ],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(
            Some("foo".into()),
            tree.state().node_by_id(NodeId(1)).unwrap().label()
        );
        let second_update = TreeUpdate {
            nodes: vec![(NodeId(1), {
                let mut node = child_node;
                node.set_label("bar");
                node
            })],
            tree: None,
            tree_id: TreeId::ROOT,
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
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == NodeId(1)
                    && old_node.label() == Some("foo".into())
                    && new_node.label() == Some("bar".into())
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
            tree.state().node_by_id(NodeId(1)).unwrap().label()
        );
    }

    // Verify that if an update consists entirely of node data and tree data
    // that's the same as before, no changes are reported. This is useful
    // for a provider that constructs a fresh tree every time, such as
    // an immediate-mode GUI.
    #[test]
    fn no_change_update() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::Button);
                    node.set_label("foo");
                    node
                }),
            ],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update.clone(), false);
        struct Handler;
        fn unexpected_change() {
            panic!("expected no changes");
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
            fn node_updated(&mut self, _old_node: &crate::Node, _new_node: &crate::Node) {
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
        let mut handler = Handler {};
        tree.update_and_process_changes(update, &mut handler);
    }

    #[test]
    fn move_node() {
        struct Handler {
            got_updated_root: bool,
            got_updated_child: bool,
            got_removed_container: bool,
        }

        fn unexpected_change() {
            panic!("expected only updated root and removed container");
        }

        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _node: &crate::Node) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == NodeId(0)
                    && old_node.child_ids().collect::<Vec<NodeId>>() == vec![NodeId(1)]
                    && new_node.child_ids().collect::<Vec<NodeId>>() == vec![NodeId(2)]
                {
                    self.got_updated_root = true;
                    return;
                }
                if new_node.id() == NodeId(2)
                    && old_node.parent_id() == Some(NodeId(1))
                    && new_node.parent_id() == Some(NodeId(0))
                {
                    self.got_updated_child = true;
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
                if node.id() == NodeId(1) {
                    self.got_removed_container = true;
                    return;
                }
                unexpected_change();
            }
        }

        let mut root = Node::new(Role::Window);
        root.set_children([NodeId(1)]);
        let mut container = Node::new(Role::GenericContainer);
        container.set_children([NodeId(2)]);
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), root.clone()),
                (NodeId(1), container),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = crate::Tree::new(update, false);
        root.set_children([NodeId(2)]);
        let mut handler = Handler {
            got_updated_root: false,
            got_updated_child: false,
            got_removed_container: false,
        };
        tree.update_and_process_changes(
            TreeUpdate {
                nodes: vec![(NodeId(0), root)],
                tree: None,
                tree_id: TreeId::ROOT,
                focus: NodeId(0),
            },
            &mut handler,
        );
        assert!(handler.got_updated_root);
        assert!(handler.got_updated_child);
        assert!(handler.got_removed_container);
        assert_eq!(
            tree.state()
                .node_by_id(NodeId(0))
                .unwrap()
                .child_ids()
                .collect::<Vec<NodeId>>(),
            vec![NodeId(2)]
        );
        assert!(tree.state().node_by_id(NodeId(1)).is_none());
        assert_eq!(
            tree.state().node_by_id(NodeId(2)).unwrap().parent_id(),
            Some(NodeId(0))
        );
    }
}
