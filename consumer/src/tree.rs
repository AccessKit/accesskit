// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Node as NodeData, NodeId, Role, Tree as TreeData, TreeUpdate};
use alloc::{borrow::ToOwned, vec::Vec};
use core::fmt;
use hashbrown::{HashMap, HashSet};

use crate::node::{Node, NodeState, ParentAndIndex};

#[derive(Clone, Debug)]
pub struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) data: TreeData,
    pub(crate) focus: NodeId,
    is_host_focused: bool,
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

    pub fn toolkit_name(&self) -> Option<&str> {
        self.data.toolkit_name.as_deref()
    }

    pub fn toolkit_version(&self) -> Option<&str> {
        self.data.toolkit_version.as_deref()
    }
}

#[derive(Debug, Default)]
struct InternalChanges {
    added_node_ids: HashSet<NodeId>,
    updated_node_ids: HashSet<NodeId>,
    removed_node_ids: HashSet<NodeId>,
}

fn add_node(
    nodes: &mut HashMap<NodeId, NodeState>,
    changes: &mut InternalChanges,
    id: NodeId,
    parent_and_index: Option<ParentAndIndex>,
    data: NodeData,
) {
    let state = NodeState {
        parent_and_index,
        data,
    };
    nodes.insert(id, state);
    changes.added_node_ids.insert(id);
}

#[derive(Debug, Default)]
struct UpdateState {
    changes: InternalChanges,
    unreachable: HashSet<NodeId>,
    pending_nodes: HashMap<NodeId, NodeData>,
    pending_children: HashMap<NodeId, ParentAndIndex>,
    seen_child_ids: HashSet<NodeId>,
    processing_children: Vec<NodeId>,
}

impl UpdateState {
    fn debug_assert_empty(&self) {
        debug_assert!(self.changes.added_node_ids.is_empty());
        debug_assert!(self.changes.updated_node_ids.is_empty());
        debug_assert!(self.changes.removed_node_ids.is_empty());
        debug_assert!(self.unreachable.is_empty());
        debug_assert!(self.pending_nodes.is_empty());
        debug_assert!(self.pending_children.is_empty());
        debug_assert!(self.processing_children.is_empty());
    }
}

/// This is the concrete [`accesskit::TreeUpdate`] implementation that most
/// platform adapters pass to application-provided callbacks. Applications do not
/// create this directly.
pub struct Update<'a> {
    nodes: &'a mut HashMap<NodeId, NodeState>,
    prev_state: Option<&'a State>,
    state: &'a mut UpdateState,
    new_tree: Option<TreeData>,
    new_focus: Option<NodeId>,
}

impl<'a> Update<'a> {
    fn new(
        nodes: &'a mut HashMap<NodeId, NodeState>,
        prev_state: Option<&'a State>,
        state: &'a mut UpdateState,
    ) -> Self {
        state.debug_assert_empty();
        Self {
            nodes,
            prev_state,
            state,
            new_tree: None,
            new_focus: None,
        }
    }
}

impl Update<'_> {
    fn add_node(&mut self, id: NodeId, parent_and_index: Option<ParentAndIndex>, data: NodeData) {
        add_node(
            self.nodes,
            &mut self.state.changes,
            id,
            parent_and_index,
            data,
        );
    }

    fn process_children(&mut self, parent_id: NodeId) {
        for (child_index, child_id) in self.state.processing_children.drain(..).enumerate() {
            if self.state.seen_child_ids.contains(&child_id) {
                panic!("TreeUpdate includes duplicate child {child_id:?}");
            }
            self.state.seen_child_ids.insert(child_id);
            self.state.unreachable.remove(&child_id);
            let parent_and_index = ParentAndIndex(parent_id, child_index);
            if let Some(child_state) = self.nodes.get_mut(&child_id) {
                if child_state.parent_and_index != Some(parent_and_index) {
                    child_state.parent_and_index = Some(parent_and_index);
                    if !self.state.changes.added_node_ids.contains(&child_id) {
                        self.state.changes.updated_node_ids.insert(child_id);
                    }
                }
            } else if let Some(child_data) = self.state.pending_nodes.remove(&child_id) {
                add_node(
                    self.nodes,
                    &mut self.state.changes,
                    child_id,
                    Some(parent_and_index),
                    child_data,
                );
            } else {
                self.state
                    .pending_children
                    .insert(child_id, parent_and_index);
            }
        }
    }

    fn root(&self) -> Option<NodeId> {
        if let Some(tree) = &self.new_tree {
            return Some(tree.root);
        }
        if let Some(state) = self.prev_state {
            return Some(state.data.root);
        }
        None
    }

    fn finish(self) -> (Option<TreeData>, Option<NodeId>) {
        if !self.state.pending_nodes.is_empty() {
            panic!("TreeUpdate includes {} nodes which are neither in the current tree nor a child of another node from the update: {}", self.state.pending_nodes.len(), ShortNodeList(&self.state.pending_nodes));
        }
        if !self.state.pending_children.is_empty() {
            panic!("TreeUpdate's nodes include {} children ids which are neither in the current tree nor the id of another node from the update: {}", self.state.pending_children.len(), ShortNodeList(&self.state.pending_children));
        }

        fn traverse_unreachable(
            nodes: &mut HashMap<NodeId, NodeState>,
            changes: &mut InternalChanges,
            seen_child_ids: &HashSet<NodeId>,
            id: NodeId,
        ) {
            changes.removed_node_ids.insert(id);
            let node = nodes.remove(&id).unwrap();
            for child_id in node.data.children().iter() {
                if !seen_child_ids.contains(child_id) {
                    traverse_unreachable(nodes, changes, seen_child_ids, *child_id);
                }
            }
        }

        for id in self.state.unreachable.drain() {
            traverse_unreachable(
                self.nodes,
                &mut self.state.changes,
                &self.state.seen_child_ids,
                id,
            );
        }
        self.state.seen_child_ids.clear();

        (self.new_tree, self.new_focus)
    }
}

impl TreeUpdate for Update<'_> {
    fn set_node(&mut self, id: NodeId, role: Role, fill: impl FnOnce(&mut NodeData)) {
        let root = self.root();
        self.state.unreachable.remove(&id);

        if let Some(node_state) = self.nodes.get_mut(&id) {
            node_state
                .data
                .children()
                .clone_into(&mut self.state.processing_children);
            node_state.data.reset(role);
            fill(&mut node_state.data);
            if let Some(prev_state) = self.prev_state {
                if let Some(prev_node_state) = prev_state.nodes.get(&id) {
                    if *prev_node_state != *node_state {
                        self.state.changes.updated_node_ids.insert(id);
                    }
                }
            }
            if self.state.processing_children == node_state.data.children() {
                self.state.processing_children.clear();
            } else {
                for child_id in self.state.processing_children.drain(..) {
                    if root != Some(child_id) {
                        self.state.unreachable.insert(child_id);
                    }
                }
                node_state
                    .data
                    .children()
                    .clone_into(&mut self.state.processing_children);
                self.process_children(id);
            }
            return;
        }

        let mut data = NodeData::new(role);
        fill(&mut data);
        data.children()
            .clone_into(&mut self.state.processing_children);
        self.process_children(id);
        if let Some(parent_and_index) = self.state.pending_children.remove(&id) {
            self.add_node(id, Some(parent_and_index), data);
            return;
        }
        if root == Some(id) {
            self.add_node(id, None, data);
            return;
        }
        self.state.pending_nodes.insert(id, data);
    }

    fn update_node(&mut self, id: NodeId, update: impl FnOnce(&mut NodeData)) {
        let root = self.root();
        self.state.unreachable.remove(&id);

        if let Some(node_state) = self.nodes.get_mut(&id) {
            node_state
                .data
                .children()
                .clone_into(&mut self.state.processing_children);
            update(&mut node_state.data);
            if let Some(prev_state) = self.prev_state {
                if let Some(prev_node_state) = prev_state.nodes.get(&id) {
                    if *prev_node_state != *node_state {
                        self.state.changes.updated_node_ids.insert(id);
                    }
                }
            }
            if self.state.processing_children == node_state.data.children() {
                self.state.processing_children.clear();
            } else {
                for child_id in self.state.processing_children.drain(..) {
                    if root != Some(child_id) {
                        self.state.unreachable.insert(child_id);
                    }
                }
                node_state
                    .data
                    .children()
                    .clone_into(&mut self.state.processing_children);
                self.process_children(id);
            }
            return;
        }

        let data = self.state.pending_nodes.get_mut(&id).unwrap();
        update(data);
        data.children()
            .clone_into(&mut self.state.processing_children);
        self.process_children(id);
    }

    fn set_tree(&mut self, tree: TreeData) {
        if let Some(prev_state) = self.prev_state {
            if prev_state.data.root != tree.root {
                let new_node_state = self.nodes.get(&prev_state.data.root).unwrap();
                if new_node_state.parent_and_index.is_none() {
                    self.state.unreachable.insert(prev_state.data.root);
                }
            }
        }
        if let Some(node_state) = self.nodes.get_mut(&tree.root) {
            node_state.parent_and_index = None;
            if let Some(prev_state) = self.prev_state {
                if let Some(prev_node_state) = prev_state.nodes.get(&tree.root) {
                    if prev_node_state.parent_and_index.is_some() {
                        self.state.changes.updated_node_ids.insert(tree.root);
                    }
                }
            }
        } else if let Some(data) = self.state.pending_nodes.remove(&tree.root) {
            self.add_node(tree.root, None, data);
        }
        self.state.unreachable.remove(&tree.root);
        self.new_tree = Some(tree);
    }

    fn set_focus(&mut self, focus: NodeId) {
        self.new_focus = Some(focus);
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
    update_state: UpdateState,
}

impl Tree {
    pub fn new_optional(is_host_focused: bool, fill: impl FnOnce(&mut Update)) -> Option<Self> {
        let mut nodes = HashMap::new();
        let mut update_state = UpdateState::default();
        let mut update = Update::new(&mut nodes, None, &mut update_state);
        fill(&mut update);
        let (tree, focus) = update.finish();
        update_state.changes.added_node_ids.clear();
        debug_assert!(update_state.changes.updated_node_ids.is_empty());
        debug_assert!(update_state.changes.removed_node_ids.is_empty());
        let tree = tree?;
        let Some(focus) = focus else {
            panic!("Tried to initialize the accessibility tree without initial focus.");
        };
        let state = State {
            nodes,
            data: tree,
            focus,
            is_host_focused,
        };
        state.validate_global();
        Some(Self {
            next_state: state.clone(),
            state,
            update_state,
        })
    }

    pub fn new(is_host_focused: bool, fill: impl FnOnce(&mut Update)) -> Self {
        let Some(tree) = Self::new_optional(is_host_focused, fill) else {
            panic!("Tried to initialize the accessibility tree without global tree info.");
        };
        tree
    }

    pub fn update(&mut self, handler: &mut impl ChangeHandler, fill: impl FnOnce(&mut Update)) {
        let mut update = Update::new(
            &mut self.next_state.nodes,
            Some(&self.state),
            &mut self.update_state,
        );
        fill(&mut update);
        let (tree, focus) = update.finish();
        if let Some(tree) = tree {
            self.next_state.data = tree;
        }
        if let Some(focus) = focus {
            self.next_state.focus = focus;
        }
        self.next_state.validate_global();
        self.process_changes(handler);
    }

    pub fn update_host_focus_state(
        &mut self,
        is_host_focused: bool,
        handler: &mut impl ChangeHandler,
    ) {
        self.update_state.debug_assert_empty();
        self.next_state.is_host_focused = is_host_focused;
        self.process_changes(handler);
    }

    fn process_changes(&mut self, handler: &mut impl ChangeHandler) {
        let changes = &mut self.update_state.changes;
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
        for id in changes.added_node_ids.drain() {
            self.state
                .nodes
                .insert(id, self.next_state.nodes.get(&id).unwrap().clone());
        }
        for id in changes.updated_node_ids.drain() {
            self.state
                .nodes
                .get_mut(&id)
                .unwrap()
                .clone_from(self.next_state.nodes.get(&id).unwrap());
        }
        for id in changes.removed_node_ids.drain() {
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
    use accesskit::{NodeId, Role, Tree, TreeUpdate};
    use alloc::{vec, vec::Vec};

    #[test]
    fn init_tree_with_root_node() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_eq!(NodeId(0), tree.state().root().id());
        assert_eq!(Role::Window, tree.state().root().role());
        assert!(tree.state().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1), NodeId(2)]);
            });
            update.set_node(NodeId(1), Role::Button, |_| ());
            update.set_node(NodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
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
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_eq!(0, tree.state().root().children().count());
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
        tree.update(&mut handler, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.push_child(NodeId(1));
            });
            update.set_node(NodeId(1), Role::RootWebArea, |_| ());
        });
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
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.push_child(NodeId(1));
            });
            update.set_node(NodeId(1), Role::RootWebArea, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_eq!(1, tree.state().root().children().count());
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
        tree.update(&mut handler, |update| {
            update.set_node(NodeId(0), Role::Window, |_| ());
        });
        assert!(handler.got_updated_root_node);
        assert!(handler.got_removed_child_node);
        assert_eq!(0, tree.state().root().children().count());
        assert!(tree.state().node_by_id(NodeId(1)).is_none());
    }

    #[test]
    fn move_focus_between_siblings() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1), NodeId(2)]);
            });
            update.set_node(NodeId(1), Role::Button, |_| ());
            update.set_node(NodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(1));
        });
        assert!(tree.state().node_by_id(NodeId(1)).unwrap().is_focused());
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
        tree.update(&mut handler, |update| {
            update.set_focus(NodeId(2));
        });
        assert!(handler.got_old_focus_node_update);
        assert!(handler.got_new_focus_node_update);
        assert!(handler.got_focus_change);
        assert!(tree.state().node_by_id(NodeId(2)).unwrap().is_focused());
        assert!(!tree.state().node_by_id(NodeId(1)).unwrap().is_focused());
    }

    #[test]
    fn update_node() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::Button, |node| {
                node.set_label("foo");
            });
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        assert_eq!(
            Some("foo".into()),
            tree.state().node_by_id(NodeId(1)).unwrap().label()
        );
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
        tree.update(&mut handler, |update| {
            update.set_node(NodeId(1), Role::Button, |node| {
                node.set_label("bar");
            });
        });
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
        let mut tree = super::Tree::new(true, crate::tests::build_test_tree);
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
        tree.update(&mut handler, crate::tests::build_test_tree);
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

        let mut tree = crate::Tree::new(false, |update| {
            update.set_node(NodeId(0), Role::Window, |node| {
                node.set_children(&[NodeId(1)]);
            });
            update.set_node(NodeId(1), Role::GenericContainer, |node| {
                node.set_children(&[NodeId(2)]);
            });
            update.set_node(NodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(NodeId(0)));
            update.set_focus(NodeId(0));
        });
        let mut handler = Handler {
            got_updated_root: false,
            got_updated_child: false,
            got_removed_container: false,
        };
        tree.update(&mut handler, |update| {
            update.update_node(NodeId(0), |node| {
                node.set_children(&[NodeId(2)]);
            });
        });
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
