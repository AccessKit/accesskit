// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Node as NodeData, NodeId as LocalNodeId, Role, Tree as TreeData, TreeId, TreeUpdate,
};
use alloc::{vec, vec::Vec};
use core::fmt;
use hashbrown::{HashMap, HashSet};

use crate::node::{Node, NodeId, NodeState, ParentAndIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct TreeIndex(pub(crate) u32);

#[derive(Clone, Debug, Default)]
struct TreeIndexMap {
    id_to_index: HashMap<TreeId, TreeIndex>,
    index_to_id: HashMap<TreeIndex, TreeId>,
    next: u32,
}

impl TreeIndexMap {
    fn get_or_create_index(&mut self, id: TreeId) -> TreeIndex {
        *self.id_to_index.entry(id).or_insert_with(|| {
            let tree_index = TreeIndex(self.next);
            self.next += 1;
            self.index_to_id.insert(tree_index, id);
            tree_index
        })
    }

    fn get_index(&self, id: TreeId) -> Option<TreeIndex> {
        self.id_to_index.get(&id).copied()
    }

    fn get_id(&self, index: TreeIndex) -> Option<TreeId> {
        self.index_to_id.get(&index).copied()
    }
}

/// State for a subtree, including its root node and current focus.
#[derive(Clone, Debug)]
pub(crate) struct SubtreeState {
    pub(crate) root: NodeId,
    pub(crate) focus: NodeId,
}

#[derive(Clone, Debug)]
pub struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) data: TreeData,
    pub(crate) root: NodeId,
    pub(crate) focus: NodeId,
    is_host_focused: bool,
    pub(crate) subtrees: HashMap<TreeId, SubtreeState>,
    pub(crate) graft_parents: HashMap<TreeId, NodeId>,
    tree_index_map: TreeIndexMap,
}

impl State {
    fn validate_global(&self) {
        if !self.nodes.contains_key(&self.root) {
            panic!("Root ID {:?} is not in the node list", self.data.root);
        }
        if !self.nodes.contains_key(&self.focus) {
            panic!(
                "Focused ID {:?} is not in the node list",
                self.focus.to_components().0
            );
        }
    }

    /// Computes the effective focus by following the graft chain from ROOT.
    /// If ROOT's focus is on a graft node, follows through to that subtree's focus,
    /// and continues recursively until reaching a non-graft node.
    fn compute_effective_focus(&self) -> NodeId {
        let Some(root_subtree) = self.subtrees.get(&TreeId::ROOT) else {
            return self.focus;
        };

        let mut current_focus = root_subtree.focus;
        while let Some(node_state) = self.nodes.get(&current_focus) {
            let Some(subtree_id) = node_state.data.tree_id() else {
                break;
            };
            let subtree = self.subtrees.get(&subtree_id).unwrap_or_else(|| {
                panic!(
                    "Focus is on graft node {:?} but subtree {:?} does not exist. \
                     Graft nodes cannot be focused without their subtree.",
                    current_focus.to_components().0,
                    subtree_id
                );
            });
            current_focus = subtree.focus;
        }
        current_focus
    }

    pub fn has_node(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn node_by_id(&self, id: NodeId) -> Option<Node<'_>> {
        self.nodes.get(&id).map(|node_state| Node {
            tree_state: self,
            id,
            state: node_state,
        })
    }

    pub fn node_by_tree_local_id(
        &self,
        local_node_id: LocalNodeId,
        tree_id: TreeId,
    ) -> Option<Node<'_>> {
        let tree_index = self.tree_index_map.get_index(tree_id)?;
        self.node_by_id(NodeId::new(local_node_id, tree_index))
    }

    pub fn root_id(&self) -> NodeId {
        self.root
    }

    pub fn root(&self) -> Node<'_> {
        self.node_by_id(self.root_id()).unwrap()
    }

    /// Returns the root NodeId of the subtree with the given TreeId, if it exists.
    pub fn subtree_root(&self, tree_id: TreeId) -> Option<NodeId> {
        self.subtrees.get(&tree_id).map(|s| s.root)
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
        self.focus_id().map(|id| {
            let focused = self.node_by_id(id).unwrap();
            focused.active_descendant().unwrap_or(focused)
        })
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

    pub fn locate_node(&self, node_id: NodeId) -> Option<(LocalNodeId, TreeId)> {
        if !self.has_node(node_id) {
            return None;
        }
        let (local_id, tree_index) = node_id.to_components();
        self.tree_index_map
            .get_id(tree_index)
            .map(|tree_id| (local_id, tree_id))
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
    pending_grafts: HashMap<TreeId, NodeId>,
    grafts_to_remove: HashSet<TreeId>,
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
        debug_assert!(self.pending_grafts.is_empty());
        debug_assert!(self.grafts_to_remove.is_empty());
    }
}

/// This is the concrete [`accesskit::TreeUpdate`] implementation that most
/// platform adapters pass to application-provided callbacks. Applications do not
/// create this directly.
pub struct Update<'a> {
    nodes: &'a mut HashMap<NodeId, NodeState>,
    prev_state: Option<&'a State>,
    state: &'a mut UpdateState,
    tree_id: TreeId,
    tree_index: TreeIndex,
    new_tree: Option<TreeData>,
    new_focus: Option<NodeId>,
}

impl<'a> Update<'a> {
    fn new(
        nodes: &'a mut HashMap<NodeId, NodeState>,
        prev_state: Option<&'a State>,
        state: &'a mut UpdateState,
        tree_id: TreeId,
        tree_index: TreeIndex,
    ) -> Self {
        state.debug_assert_empty();
        Self {
            nodes,
            prev_state,
            state,
            tree_id,
            tree_index,
            new_tree: None,
            new_focus: None,
        }
    }
}

fn record_graft(state: &mut UpdateState, subtree_id: TreeId, graft_node_id: NodeId) {
    if subtree_id == TreeId::ROOT {
        panic!("Cannot graft the root tree");
    }
    if let Some(existing_graft) = state.pending_grafts.get(&subtree_id) {
        panic!(
            "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
            subtree_id,
            existing_graft.to_components().0,
            graft_node_id.to_components().0
        );
    }
    state.pending_grafts.insert(subtree_id, graft_node_id);
}

impl Update<'_> {
    fn map_id(&self, local_id: LocalNodeId) -> NodeId {
        NodeId::new(local_id, self.tree_index)
    }

    fn add_node(&mut self, id: NodeId, parent_and_index: Option<ParentAndIndex>, data: NodeData) {
        if let Some(subtree_id) = data.tree_id() {
            if !data.children().is_empty() {
                panic!(
                    "Node {:?} has both tree_id and children. \
                     A graft node's only child comes from its subtree.",
                    id.to_components().0
                );
            }
            record_graft(self.state, subtree_id, id);
        }
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
            return Some(self.map_id(tree.root));
        }
        if let Some(state) = self.prev_state {
            if self.tree_id == TreeId::ROOT {
                return Some(state.root);
            }
            return state.subtrees.get(&self.tree_id).map(|s| s.root);
        }
        None
    }

    fn finish(self) -> (Option<TreeData>, Option<NodeId>) {
        if !self.state.pending_nodes.is_empty() {
            panic!(
                "TreeUpdate includes {} nodes which are neither in the current tree nor a child of another node from the update: {}",
                self.state.pending_nodes.len(),
                ShortNodeList(&self.state.pending_nodes)
            );
        }
        if !self.state.pending_children.is_empty() {
            panic!(
                "TreeUpdate's nodes include {} children ids which are neither in the current tree nor the id of another node from the update: {}",
                self.state.pending_children.len(),
                ShortNodeList(&self.state.pending_children)
            );
        }

        fn traverse_unreachable(
            nodes: &mut HashMap<NodeId, NodeState>,
            changes: &mut InternalChanges,
            grafts_to_remove: &mut HashSet<TreeId>,
            id: NodeId,
        ) {
            let Some(node) = nodes.remove(&id) else {
                return;
            };
            changes.removed_node_ids.insert(id);
            if let Some(subtree_id) = node.data.tree_id() {
                grafts_to_remove.insert(subtree_id);
            }
            let (_, tree_index) = id.to_components();
            for child_id in node.data.children().iter() {
                let child = NodeId::new(*child_id, tree_index);
                let still_attached = nodes
                    .get(&child)
                    .and_then(|c| c.parent_and_index)
                    .is_some_and(|ParentAndIndex(parent, _)| parent == id);
                if still_attached {
                    traverse_unreachable(nodes, changes, grafts_to_remove, child);
                }
            }
        }

        for id in self.state.unreachable.drain() {
            traverse_unreachable(
                self.nodes,
                &mut self.state.changes,
                &mut self.state.grafts_to_remove,
                id,
            );
        }
        self.state.seen_child_ids.clear();

        (self.new_tree, self.new_focus)
    }
}

impl TreeUpdate for Update<'_> {
    fn set_node(&mut self, local_id: LocalNodeId, role: Role, fill: impl FnOnce(&mut NodeData)) {
        let id = self.map_id(local_id);
        let root = self.root();
        let tree_index = self.tree_index;
        self.state.unreachable.remove(&id);

        if let Some(node_state) = self.nodes.get_mut(&id) {
            let old_tree_id = node_state.data.tree_id();
            self.state.processing_children.clear();
            self.state.processing_children.extend(
                node_state
                    .data
                    .children()
                    .iter()
                    .map(|child_id| NodeId::new(*child_id, tree_index)),
            );
            node_state.data.reset(role);
            fill(&mut node_state.data);
            let new_tree_id = node_state.data.tree_id();
            if new_tree_id.is_some() && !node_state.data.children().is_empty() {
                panic!(
                    "Node {:?} has both tree_id and children. \
                     A graft node's only child comes from its subtree.",
                    local_id
                );
            }

            if let Some(prev_node_state) = self.prev_state.and_then(|p| p.nodes.get(&id)) {
                if *prev_node_state != *node_state {
                    self.state.changes.updated_node_ids.insert(id);
                }
            }

            if old_tree_id != new_tree_id {
                if let Some(old_subtree_id) = old_tree_id {
                    self.state.grafts_to_remove.insert(old_subtree_id);
                }
                if let Some(new_subtree_id) = new_tree_id {
                    record_graft(self.state, new_subtree_id, id);
                }
            }

            let children_differ = self.state.processing_children.len()
                != node_state.data.children().len()
                || self
                    .state
                    .processing_children
                    .iter()
                    .zip(node_state.data.children())
                    .any(|(old, new)| *old != NodeId::new(*new, tree_index));
            if children_differ {
                for child_id in self.state.processing_children.drain(..) {
                    if root != Some(child_id) {
                        self.state.unreachable.insert(child_id);
                    }
                }
                self.state.processing_children.extend(
                    node_state
                        .data
                        .children()
                        .iter()
                        .map(|child_id| NodeId::new(*child_id, tree_index)),
                );
                self.process_children(id);
            } else {
                self.state.processing_children.clear();
            }
            return;
        }

        let mut data = NodeData::new(role);
        fill(&mut data);
        self.state.processing_children.clear();
        for child_id in data.children() {
            self.state
                .processing_children
                .push(NodeId::new(*child_id, tree_index));
        }
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

    fn update_node(&mut self, local_id: LocalNodeId, update: impl FnOnce(&mut NodeData)) {
        let id = self.map_id(local_id);
        let root = self.root();
        let tree_index = self.tree_index;
        self.state.unreachable.remove(&id);

        if let Some(node_state) = self.nodes.get_mut(&id) {
            let old_tree_id = node_state.data.tree_id();
            self.state.processing_children.clear();
            self.state.processing_children.extend(
                node_state
                    .data
                    .children()
                    .iter()
                    .map(|child_id| NodeId::new(*child_id, tree_index)),
            );
            update(&mut node_state.data);
            let new_tree_id = node_state.data.tree_id();
            if new_tree_id.is_some() && !node_state.data.children().is_empty() {
                panic!(
                    "Node {:?} has both tree_id and children. \
                     A graft node's only child comes from its subtree.",
                    local_id
                );
            }

            if let Some(prev_node_state) = self.prev_state.and_then(|p| p.nodes.get(&id)) {
                if *prev_node_state != *node_state {
                    self.state.changes.updated_node_ids.insert(id);
                }
            }

            if old_tree_id != new_tree_id {
                if let Some(old_subtree_id) = old_tree_id {
                    self.state.grafts_to_remove.insert(old_subtree_id);
                }
                if let Some(new_subtree_id) = new_tree_id {
                    record_graft(self.state, new_subtree_id, id);
                }
            }

            let children_differ = self.state.processing_children.len()
                != node_state.data.children().len()
                || self
                    .state
                    .processing_children
                    .iter()
                    .zip(node_state.data.children())
                    .any(|(old, new)| *old != NodeId::new(*new, tree_index));
            if children_differ {
                for child_id in self.state.processing_children.drain(..) {
                    if root != Some(child_id) {
                        self.state.unreachable.insert(child_id);
                    }
                }
                self.state.processing_children.extend(
                    node_state
                        .data
                        .children()
                        .iter()
                        .map(|child_id| NodeId::new(*child_id, tree_index)),
                );
                self.process_children(id);
            } else {
                self.state.processing_children.clear();
            }
            return;
        }

        let data = self.state.pending_nodes.get_mut(&id).unwrap();
        update(data);
        self.state.processing_children.clear();
        self.state.processing_children.extend(
            data.children()
                .iter()
                .map(|child_id| NodeId::new(*child_id, tree_index)),
        );
        self.process_children(id);
    }

    fn set_tree(&mut self, tree: TreeData) {
        let new_root = self.map_id(tree.root);
        let tree_index = self.tree_index;

        let old_root = self.prev_state.and_then(|prev_state| {
            let old_root_local = if self.tree_id == TreeId::ROOT {
                Some(prev_state.data.root)
            } else {
                prev_state
                    .subtrees
                    .get(&self.tree_id)
                    .map(|s| s.root.to_components().0)
            };
            old_root_local.map(|local| NodeId::new(local, tree_index))
        });
        if let Some(old_root) = old_root {
            if old_root != new_root {
                let graft = self
                    .prev_state
                    .and_then(|p| p.graft_parents.get(&self.tree_id).copied());
                if let Some(old_root_state) = self.nodes.get(&old_root) {
                    let parent = old_root_state
                        .parent_and_index
                        .map(|ParentAndIndex(p, _)| p);
                    if parent.is_none() || parent == graft {
                        self.state.unreachable.insert(old_root);
                    }
                }
            }
        }

        if let Some(node_state) = self.nodes.get_mut(&new_root) {
            node_state.parent_and_index = None;
            if let Some(prev_state) = self.prev_state {
                if let Some(prev_node_state) = prev_state.nodes.get(&new_root) {
                    if prev_node_state.parent_and_index.is_some() {
                        self.state.changes.updated_node_ids.insert(new_root);
                    }
                }
            }
        } else if let Some(data) = self.state.pending_nodes.remove(&new_root) {
            self.add_node(new_root, None, data);
        }
        self.state.unreachable.remove(&new_root);
        self.new_tree = Some(tree);
    }

    fn set_focus(&mut self, focus: LocalNodeId) {
        self.new_focus = Some(self.map_id(focus));
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
        let mut tree_index_map = TreeIndexMap::default();
        let tree_index = tree_index_map.get_or_create_index(TreeId::ROOT);
        let mut nodes = HashMap::new();
        let mut update_state = UpdateState::default();
        let mut update = Update::new(
            &mut nodes,
            None,
            &mut update_state,
            TreeId::ROOT,
            tree_index,
        );
        fill(&mut update);
        let (tree, focus) = update.finish();
        update_state.changes.added_node_ids.clear();
        update_state.grafts_to_remove.clear();
        debug_assert!(update_state.changes.updated_node_ids.is_empty());
        debug_assert!(update_state.changes.removed_node_ids.is_empty());
        let tree = tree?;
        let Some(focus) = focus else {
            panic!("Tried to initialize the accessibility tree without initial focus.");
        };
        let root = NodeId::new(tree.root, tree_index);
        let mut subtrees = HashMap::new();
        subtrees.insert(TreeId::ROOT, SubtreeState { root, focus });
        let mut graft_parents = HashMap::new();
        for (subtree_id, node_id) in update_state.pending_grafts.drain() {
            graft_parents.insert(subtree_id, node_id);
        }
        let mut state = State {
            nodes,
            data: tree,
            root,
            focus,
            is_host_focused,
            subtrees,
            graft_parents,
            tree_index_map,
        };
        state.focus = state.compute_effective_focus();
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

    pub fn update(
        &mut self,
        tree_id: TreeId,
        handler: &mut impl ChangeHandler,
        fill: impl FnOnce(&mut Update),
    ) {
        let tree_index = self.next_state.tree_index_map.get_or_create_index(tree_id);
        let subtree_existed = self.next_state.subtrees.contains_key(&tree_id);
        let graft_exists = self.next_state.graft_parents.contains_key(&tree_id);

        let mut update = Update::new(
            &mut self.next_state.nodes,
            Some(&self.state),
            &mut self.update_state,
            tree_id,
            tree_index,
        );
        fill(&mut update);

        if tree_id != TreeId::ROOT {
            if update.new_tree.is_some() && !graft_exists {
                panic!(
                    "Cannot push subtree {:?}: no graft node exists for this tree. \
                     Push the graft node (with tree_id property set) before pushing the subtree.",
                    tree_id
                );
            }
            if !subtree_existed && update.new_tree.is_none() {
                panic!(
                    "Cannot update subtree {:?}: subtree does not exist. \
                     The first update for a subtree must include tree data.",
                    tree_id
                );
            }
        }

        let (tree, focus) = update.finish();

        let new_tree_root = tree.as_ref().map(|t| NodeId::new(t.root, tree_index));

        if let Some(tree) = tree {
            if tree_id == TreeId::ROOT {
                self.next_state.root = NodeId::new(tree.root, tree_index);
                self.next_state.data = tree;
            }
        }

        match new_tree_root {
            Some(new_root) => {
                let focus = focus.unwrap_or(new_root);
                self.next_state.subtrees.insert(
                    tree_id,
                    SubtreeState {
                        root: new_root,
                        focus,
                    },
                );
            }
            None => {
                if let Some(focus) = focus {
                    if let Some(subtree) = self.next_state.subtrees.get_mut(&tree_id) {
                        subtree.focus = focus;
                    }
                }
            }
        }

        self.reconcile_grafts(tree_id, new_tree_root);

        self.next_state.focus = self.next_state.compute_effective_focus();
        self.next_state.validate_global();
        self.process_changes(handler);
    }

    fn reconcile_grafts(&mut self, tree_id: TreeId, new_tree_root: Option<NodeId>) {
        let next = &mut self.next_state;
        let us = &mut self.update_state;

        let mut subtrees_queued: HashSet<TreeId> = us.grafts_to_remove.drain().collect();
        let mut subtrees_to_remove: Vec<TreeId> = subtrees_queued.iter().copied().collect();
        let mut i = 0;
        while i < subtrees_to_remove.len() {
            let subtree_id = subtrees_to_remove[i];
            i += 1;
            if next.graft_parents.remove(&subtree_id).is_none() {
                continue;
            }
            if us.pending_grafts.contains_key(&subtree_id) {
                continue;
            }
            if let Some(subtree) = next.subtrees.remove(&subtree_id) {
                traverse_subtree(
                    &mut next.nodes,
                    &mut subtrees_to_remove,
                    &mut subtrees_queued,
                    &mut us.changes,
                    subtree.root,
                );
            }
        }

        for (subtree_id, node_id) in us.pending_grafts.drain() {
            if let Some(&existing_graft) = next.graft_parents.get(&subtree_id) {
                panic!(
                    "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
                    subtree_id,
                    existing_graft.to_components().0,
                    node_id.to_components().0
                );
            }
            next.graft_parents.insert(subtree_id, node_id);
            if let Some(subtree) = next.subtrees.get(&subtree_id) {
                let subtree_root_id = subtree.root;
                if let Some(root_state) = next.nodes.get_mut(&subtree_root_id) {
                    root_state.parent_and_index = Some(ParentAndIndex(node_id, 0));
                    if !us.changes.added_node_ids.contains(&subtree_root_id) {
                        us.changes.updated_node_ids.insert(subtree_root_id);
                    }
                }
            }
        }

        if let Some(new_root_id) = new_tree_root {
            if let Some(&graft_node_id) = next.graft_parents.get(&tree_id) {
                if let Some(root_state) = next.nodes.get_mut(&new_root_id) {
                    root_state.parent_and_index = Some(ParentAndIndex(graft_node_id, 0));
                    if !us.changes.added_node_ids.contains(&new_root_id) {
                        us.changes.updated_node_ids.insert(new_root_id);
                    }
                }
            }
        }
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
        let old_focus = self.state.focus();
        let new_focus = self.next_state.focus();
        if old_focus.as_ref().map(|n| n.id()) != new_focus.as_ref().map(|n| n.id()) {
            if let Some(old_node) = &old_focus {
                let id = old_node.id();
                if !changes.updated_node_ids.contains(&id)
                    && !changes.removed_node_ids.contains(&id)
                {
                    if let Some(old_node_new_version) = self.next_state.node_by_id(id) {
                        handler.node_updated(old_node, &old_node_new_version);
                    }
                }
            }
            if let Some(new_node) = &new_focus {
                let id = new_node.id();
                if !changes.added_node_ids.contains(&id) && !changes.updated_node_ids.contains(&id)
                {
                    if let Some(new_node_old_version) = self.state.node_by_id(id) {
                        handler.node_updated(&new_node_old_version, new_node);
                    }
                }
            }
            handler.focus_moved(old_focus.as_ref(), new_focus.as_ref());
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
        self.state.root = self.next_state.root;
        self.state.focus = self.next_state.focus;
        self.state.is_host_focused = self.next_state.is_host_focused;
        self.state.subtrees.clone_from(&self.next_state.subtrees);
        self.state
            .graft_parents
            .clone_from(&self.next_state.graft_parents);
        self.state
            .tree_index_map
            .clone_from(&self.next_state.tree_index_map);
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

fn traverse_subtree(
    nodes: &mut HashMap<NodeId, NodeState>,
    subtrees_to_remove: &mut Vec<TreeId>,
    subtrees_queued: &mut HashSet<TreeId>,
    changes: &mut InternalChanges,
    root: NodeId,
) {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        let Some(node) = nodes.remove(&id) else {
            continue;
        };
        changes.removed_node_ids.insert(id);
        if let Some(nested_subtree_id) = node.data.tree_id() {
            if subtrees_queued.insert(nested_subtree_id) {
                subtrees_to_remove.push(nested_subtree_id);
            }
        }
        let (_, tree_index) = id.to_components();
        for child_id in node.data.children().iter() {
            stack.push(NodeId::new(*child_id, tree_index));
        }
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
            write!(f, "{:?}", id.to_components().0)?;
        }
        if iter.next().is_some() {
            write!(f, " ...")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use accesskit::{NodeId as LocalNodeId, Role, Tree, TreeId, TreeUpdate, Uuid};
    use alloc::{vec, vec::Vec};

    use super::{TreeIndex, TreeIndexMap};
    use crate::node::NodeId;

    struct NoOpHandler;
    impl super::ChangeHandler for NoOpHandler {
        fn node_added(&mut self, _: &crate::Node) {}
        fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
        fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
        fn node_removed(&mut self, _: &crate::Node) {}
    }

    fn node_id(n: u64) -> NodeId {
        NodeId::new(LocalNodeId(n), TreeIndex(0))
    }

    #[test]
    fn tree_index_map_assigns_sequential_indices() {
        let mut map = TreeIndexMap::default();
        let id1 = TreeId::ROOT;
        let id2 = TreeId(Uuid::from_u128(1));
        let id3 = TreeId(Uuid::from_u128(2));

        let index1 = map.get_or_create_index(id1);
        let index2 = map.get_or_create_index(id2);
        let index3 = map.get_or_create_index(id3);

        assert_eq!(index1, TreeIndex(0));
        assert_eq!(index2, TreeIndex(1));
        assert_eq!(index3, TreeIndex(2));
    }

    #[test]
    fn tree_index_map_returns_same_index_for_same_id() {
        let mut map = TreeIndexMap::default();
        let id = TreeId::ROOT;

        let index1 = map.get_or_create_index(id);
        let index2 = map.get_or_create_index(id);

        assert_eq!(index1, index2);
    }

    #[test]
    fn tree_index_map_get_id_returns_correct_id() {
        let mut map = TreeIndexMap::default();
        let id1 = TreeId::ROOT;
        let id2 = TreeId(Uuid::from_u128(1));

        let index1 = map.get_or_create_index(id1);
        let index2 = map.get_or_create_index(id2);

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
    fn tree_index_map_get_index_returns_existing_index() {
        let mut map = TreeIndexMap::default();
        let id1 = TreeId::ROOT;
        let id2 = TreeId(Uuid::from_u128(1));

        let created1 = map.get_or_create_index(id1);
        let created2 = map.get_or_create_index(id2);

        assert_eq!(map.get_index(id1), Some(created1));
        assert_eq!(map.get_index(id2), Some(created2));
    }

    #[test]
    fn tree_index_map_get_index_returns_none_for_unknown_id() {
        let map = TreeIndexMap::default();
        assert_eq!(map.get_index(TreeId::ROOT), None);
        assert_eq!(map.get_index(TreeId(Uuid::from_u128(42))), None);
    }

    #[test]
    fn tree_index_map_get_index_does_not_create_mapping() {
        let mut map = TreeIndexMap::default();
        let unknown = TreeId(Uuid::from_u128(7));

        assert_eq!(map.get_index(unknown), None);

        let first = map.get_or_create_index(TreeId::ROOT);
        assert_eq!(first, TreeIndex(0));
        assert_eq!(map.get_index(unknown), None);
    }

    #[test]
    fn init_tree_with_root_node() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(node_id(0), tree.state().root().id());
        assert_eq!(Role::Window, tree.state().root().role());
        assert!(tree.state().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        let state = tree.state();
        assert_eq!(
            node_id(0),
            state.node_by_id(node_id(1)).unwrap().parent().unwrap().id()
        );
        assert_eq!(
            node_id(0),
            state.node_by_id(node_id(2)).unwrap().parent().unwrap().id()
        );
        assert_eq!(2, state.root().children().count());
    }

    #[test]
    fn add_child_to_root_node() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
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
                if node.id() == node_id(1) {
                    self.got_new_child_node = true;
                    return;
                }
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == node_id(0)
                    && old_node.data().children().is_empty()
                    && new_node.data().children() == [LocalNodeId(1)]
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
        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.push_child(LocalNodeId(1));
            });
            update.set_node(LocalNodeId(1), Role::RootWebArea, |_| ());
        });
        assert!(handler.got_new_child_node);
        assert!(handler.got_updated_root_node);
        let state = tree.state();
        assert_eq!(1, state.root().children().count());
        assert_eq!(node_id(1), state.root().children().next().unwrap().id());
        assert_eq!(
            node_id(0),
            state.node_by_id(node_id(1)).unwrap().parent().unwrap().id()
        );
    }

    #[test]
    fn remove_child_from_root_node() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.push_child(LocalNodeId(1));
            });
            update.set_node(LocalNodeId(1), Role::RootWebArea, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
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
                if new_node.id() == node_id(0)
                    && old_node.data().children() == [LocalNodeId(1)]
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
                if node.id() == node_id(1) {
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
        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |_| ());
        });
        assert!(handler.got_updated_root_node);
        assert!(handler.got_removed_child_node);
        assert_eq!(0, tree.state().root().children().count());
        assert!(tree.state().node_by_id(node_id(1)).is_none());
    }

    #[test]
    fn move_focus_between_siblings() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });
        assert!(tree.state().node_by_id(node_id(1)).unwrap().is_focused());
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
                if old_node.id() == node_id(1)
                    && new_node.id() == node_id(1)
                    && old_node.is_focused()
                    && !new_node.is_focused()
                {
                    self.got_old_focus_node_update = true;
                    return;
                }
                if old_node.id() == node_id(2)
                    && new_node.id() == node_id(2)
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
                    if old_node.id() == node_id(1) && new_node.id() == node_id(2) {
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
        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_focus(LocalNodeId(2));
        });
        assert!(handler.got_old_focus_node_update);
        assert!(handler.got_new_focus_node_update);
        assert!(handler.got_focus_change);
        assert!(tree.state().node_by_id(node_id(2)).unwrap().is_focused());
        assert!(!tree.state().node_by_id(node_id(1)).unwrap().is_focused());
    }

    #[test]
    fn update_node() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |node| {
                node.set_label("foo");
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(
            Some("foo".into()),
            tree.state().node_by_id(node_id(1)).unwrap().label()
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
                if new_node.id() == node_id(1)
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
        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_node(LocalNodeId(1), Role::Button, |node| {
                node.set_label("bar");
            });
        });
        assert!(handler.got_updated_child_node);
        assert_eq!(
            Some("bar".into()),
            tree.state().node_by_id(node_id(1)).unwrap().label()
        );
    }

    // Verify that if an update consists entirely of node data and tree data
    // that's the same as before, no changes are reported. This is useful
    // for a provider that constructs a fresh tree every time, such as
    // an immediate-mode GUI.
    #[test]
    fn no_change_update() {
        let mut tree = super::Tree::new(false, crate::tests::build_test_tree);
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
        tree.update(TreeId::ROOT, &mut handler, crate::tests::build_test_tree);
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
                if new_node.id() == node_id(0)
                    && old_node.child_ids().collect::<Vec<NodeId>>() == vec![node_id(1)]
                    && new_node.child_ids().collect::<Vec<NodeId>>() == vec![node_id(2)]
                {
                    self.got_updated_root = true;
                    return;
                }
                if new_node.id() == node_id(2)
                    && old_node.parent_id() == Some(node_id(1))
                    && new_node.parent_id() == Some(node_id(0))
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
                if node.id() == node_id(1) {
                    self.got_removed_container = true;
                    return;
                }
                unexpected_change();
            }
        }

        let mut tree = crate::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_children(&[LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        let mut handler = Handler {
            got_updated_root: false,
            got_updated_child: false,
            got_removed_container: false,
        };
        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.update_node(LocalNodeId(0), |node| {
                node.set_children(&[LocalNodeId(2)]);
            });
        });
        assert!(handler.got_updated_root);
        assert!(handler.got_updated_child);
        assert!(handler.got_removed_container);
        assert_eq!(
            tree.state()
                .node_by_id(node_id(0))
                .unwrap()
                .child_ids()
                .collect::<Vec<NodeId>>(),
            vec![node_id(2)]
        );
        assert!(tree.state().node_by_id(node_id(1)).is_none());
        assert_eq!(
            tree.state().node_by_id(node_id(2)).unwrap().parent_id(),
            Some(node_id(0))
        );
    }

    fn subtree_id() -> TreeId {
        TreeId(Uuid::from_u128(1))
    }

    #[test]
    fn graft_node_tracks_subtree() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );
    }

    #[test]
    #[should_panic(expected = "already has a graft parent")]
    fn duplicate_graft_parent_panics() {
        let _ = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_node(LocalNodeId(2), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
    }

    #[test]
    fn reparent_subtree_by_removing_old_graft() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_node(LocalNodeId(2), Role::GenericContainer, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(2), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(2))
        );
    }

    #[test]
    fn reparent_subtree_by_clearing_old_graft_tree_id() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_node(LocalNodeId(2), Role::GenericContainer, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(1), Role::GenericContainer, |_| ());
            update.set_node(LocalNodeId(2), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_focus(LocalNodeId(0));
        });
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(2))
        );
    }

    #[test]
    #[should_panic(expected = "already has a graft parent")]
    fn duplicate_graft_parent_on_update_panics() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_node(LocalNodeId(2), Role::GenericContainer, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(2), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_focus(LocalNodeId(0));
        });
    }

    #[test]
    #[should_panic(expected = "Cannot graft the root tree")]
    fn graft_root_tree_panics() {
        let _ = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(TreeId::ROOT);
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
    }

    #[test]
    #[should_panic(expected = "Cannot graft the root tree")]
    fn graft_root_tree_on_update_panics() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(TreeId::ROOT);
            });
            update.set_focus(LocalNodeId(0));
        });
    }

    fn subtree_node_id(id: u64) -> NodeId {
        NodeId::new(LocalNodeId(id), TreeIndex(1))
    }

    #[test]
    fn node_by_tree_local_id_finds_root_tree_node() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let root = tree
            .state()
            .node_by_tree_local_id(LocalNodeId(0), TreeId::ROOT)
            .unwrap();
        assert_eq!(root.id(), node_id(0));
        assert_eq!(root.role(), Role::Window);

        let child = tree
            .state()
            .node_by_tree_local_id(LocalNodeId(1), TreeId::ROOT)
            .unwrap();
        assert_eq!(child.id(), node_id(1));
        assert_eq!(child.role(), Role::Button);
    }

    #[test]
    fn node_by_tree_local_id_finds_subtree_node() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let sub_root = tree
            .state()
            .node_by_tree_local_id(LocalNodeId(0), subtree_id())
            .unwrap();
        assert_eq!(sub_root.id(), subtree_node_id(0));
        assert_eq!(sub_root.role(), Role::Document);

        let sub_child = tree
            .state()
            .node_by_tree_local_id(LocalNodeId(1), subtree_id())
            .unwrap();
        assert_eq!(sub_child.id(), subtree_node_id(1));
        assert_eq!(sub_child.role(), Role::Paragraph);

        let graft = tree
            .state()
            .node_by_tree_local_id(LocalNodeId(1), TreeId::ROOT)
            .unwrap();
        assert_eq!(graft.id(), node_id(1));
        assert_eq!(graft.role(), Role::GenericContainer);
    }

    #[test]
    fn node_by_tree_local_id_returns_none_for_unknown_tree_id() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        assert!(
            tree.state()
                .node_by_tree_local_id(LocalNodeId(0), subtree_id())
                .is_none()
        );
    }

    #[test]
    fn node_by_tree_local_id_returns_none_for_unknown_local_id() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        assert!(
            tree.state()
                .node_by_tree_local_id(LocalNodeId(999), TreeId::ROOT)
                .is_none()
        );
    }

    #[test]
    fn subtree_root_parent_is_graft_when_graft_exists_first() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent_id(), Some(node_id(1)));

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        let children: Vec<_> = graft_node.child_ids().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], subtree_node_id(0));
    }

    #[test]
    #[should_panic(expected = "no graft node exists for this tree")]
    fn subtree_push_without_graft_panics() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
    }

    #[test]
    #[should_panic(expected = "subtree does not exist")]
    fn subtree_update_without_tree_data_panics() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |_| ());
            update.set_focus(LocalNodeId(0));
        });
    }

    #[test]
    fn subtree_nodes_removed_when_graft_removed() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(nested_subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(nested_subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());
        assert!(tree.state().node_by_id(nested_subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(nested_subtree_node_id(1)).is_some());

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[]);
            });
            update.set_focus(LocalNodeId(0));
        });

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());
        assert!(tree.state().node_by_id(nested_subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(nested_subtree_node_id(1)).is_none());
        assert!(tree.state().subtrees.get(&subtree_id()).is_none());
        assert!(tree.state().subtrees.get(&nested_subtree_id()).is_none());
    }

    #[test]
    fn subtree_nodes_removed_when_graft_tree_id_cleared() {
        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(1), Role::GenericContainer, |_| ());
            update.set_focus(LocalNodeId(0));
        });

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());
        assert!(tree.state().subtrees.get(&subtree_id()).is_none());
    }

    #[test]
    fn graft_node_has_no_children_when_subtree_not_pushed() {
        let tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        assert_eq!(graft_node.child_ids().count(), 0);
        assert_eq!(graft_node.children().count(), 0);
    }

    #[test]
    #[should_panic(expected = "has both tree_id")]
    fn graft_node_with_children_panics() {
        super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
                node.set_children(&[LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });
    }

    #[test]
    fn node_added_called_when_subtree_pushed() {
        struct Handler {
            added_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let mut handler = Handler {
            added_nodes: Vec::new(),
        };

        tree.update(subtree_id(), &mut handler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        assert_eq!(handler.added_nodes.len(), 3,);
        assert!(handler.added_nodes.contains(&subtree_node_id(0)),);
        assert!(handler.added_nodes.contains(&subtree_node_id(1)),);
        assert!(handler.added_nodes.contains(&subtree_node_id(2)),);
    }

    #[test]
    fn node_removed_called_when_graft_removed() {
        struct Handler {
            removed_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, node: &crate::Node) {
                self.removed_nodes.push(node.id());
            }
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        let mut handler = Handler {
            removed_nodes: Vec::new(),
        };

        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[]);
            });
            update.set_focus(LocalNodeId(0));
        });

        assert!(handler.removed_nodes.contains(&node_id(1)),);
        assert!(handler.removed_nodes.contains(&subtree_node_id(0)),);
        assert!(handler.removed_nodes.contains(&subtree_node_id(1)),);
        assert_eq!(handler.removed_nodes.len(), 3,);
    }

    #[test]
    fn node_updated_called_when_subtree_reparented() {
        struct Handler {
            updated_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _old: &crate::Node, new: &crate::Node) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_node(LocalNodeId(2), Role::GenericContainer, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent().unwrap().id(), node_id(1));

        let mut handler = Handler {
            updated_nodes: Vec::new(),
        };

        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_node(LocalNodeId(1), Role::GenericContainer, |_| ());
            update.set_node(LocalNodeId(2), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_focus(LocalNodeId(0));
        });

        assert!(handler.updated_nodes.contains(&subtree_node_id(0)),);

        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent().unwrap().id(), node_id(2));
    }

    #[test]
    fn focus_moved_called_when_focus_moves_to_subtree() {
        struct Handler {
            focus_moves: Vec<(Option<NodeId>, Option<NodeId>)>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, old: Option<&crate::Node>, new: Option<&crate::Node>) {
                self.focus_moves
                    .push((old.map(|n| n.id()), new.map(|n| n.id())));
            }
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let mut handler = Handler {
            focus_moves: Vec::new(),
        };

        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_focus(LocalNodeId(1));
        });

        assert_eq!(handler.focus_moves.len(), 1,);
        let (old_focus, new_focus) = &handler.focus_moves[0];
        assert_eq!(*old_focus, Some(node_id(0)),);
        assert_eq!(*new_focus, Some(subtree_node_id(0)),);
    }

    #[test]
    fn focus_moved_called_when_subtree_focus_changes() {
        struct Handler {
            focus_moves: Vec<(Option<NodeId>, Option<NodeId>)>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, old: Option<&crate::Node>, new: Option<&crate::Node>) {
                self.focus_moves
                    .push((old.map(|n| n.id()), new.map(|n| n.id())));
            }
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_focus(LocalNodeId(1));
        });

        let mut handler = Handler {
            focus_moves: Vec::new(),
        };

        tree.update(subtree_id(), &mut handler, |update| {
            update.set_focus(LocalNodeId(1));
        });

        assert_eq!(handler.focus_moves.len(), 1,);
        let (old_focus, new_focus) = &handler.focus_moves[0];
        assert_eq!(*old_focus, Some(subtree_node_id(0)),);
        assert_eq!(*new_focus, Some(subtree_node_id(1)),);
    }

    fn nested_subtree_id() -> TreeId {
        TreeId(Uuid::from_u128(2))
    }

    fn nested_subtree_node_id(n: u64) -> NodeId {
        NodeId::new(LocalNodeId(n), TreeIndex(2))
    }

    #[test]
    fn nested_subtree_focus_follows_graft_chain() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(nested_subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(nested_subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Group, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_focus(LocalNodeId(1));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_focus(LocalNodeId(1));
        });

        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(1)),);
    }

    #[test]
    fn nested_subtree_focus_update_changes_effective_focus() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(nested_subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });

        tree.update(nested_subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Group, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });

        tree.update(TreeId::ROOT, &mut NoOpHandler, |update| {
            update.set_focus(LocalNodeId(1));
        });

        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(1)));

        tree.update(nested_subtree_id(), &mut NoOpHandler, |update| {
            update.set_focus(LocalNodeId(2));
        });

        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(2)),);
    }

    #[test]
    #[should_panic(expected = "Graft nodes cannot be focused without their subtree")]
    fn removing_nested_subtree_while_intermediate_focus_on_graft_panics() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(nested_subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });

        tree.update(nested_subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(1), Role::GenericContainer, |_| ());
            update.set_focus(LocalNodeId(1));
        });
    }

    #[test]
    fn nested_subtree_root_lookup_for_focus_only_update() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1), LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(nested_subtree_id());
            });
            update.set_node(LocalNodeId(2), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(nested_subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Group, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_focus(LocalNodeId(2));
        });

        assert_eq!(
            tree.state().subtrees.get(&subtree_id()).unwrap().focus,
            subtree_node_id(2),
        );
    }

    #[test]
    fn subtree_root_change_updates_graft_and_parent() {
        struct Handler {
            updated_nodes: Vec<NodeId>,
            added_nodes: Vec<NodeId>,
            removed_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::Node, new: &crate::Node) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, node: &crate::Node) {
                self.removed_nodes.push(node.id());
            }
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        tree.update(subtree_id(), &mut handler, |update| {
            update.set_node(LocalNodeId(2), Role::Article, |node| {
                node.set_children(&[LocalNodeId(3)]);
            });
            update.set_node(LocalNodeId(3), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(2)));
            update.set_focus(LocalNodeId(2));
        });

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        let children: Vec<_> = graft_node.child_ids().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], subtree_node_id(2));

        let new_subtree_root = tree.state().node_by_id(subtree_node_id(2)).unwrap();
        assert_eq!(new_subtree_root.parent_id(), Some(node_id(1)));
        assert_eq!(new_subtree_root.role(), Role::Article);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());

        assert!(tree.state().node_by_id(subtree_node_id(2)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(3)).is_some());

        assert!(handler.removed_nodes.contains(&subtree_node_id(0)),);
        assert!(handler.removed_nodes.contains(&subtree_node_id(1)),);
        assert!(handler.added_nodes.contains(&subtree_node_id(2)),);
        assert!(handler.added_nodes.contains(&subtree_node_id(3)),);
    }

    #[test]
    fn subtree_root_change_to_existing_child() {
        struct Handler {
            updated_nodes: Vec<NodeId>,
            added_nodes: Vec<NodeId>,
            removed_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::Node, new: &crate::Node) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, node: &crate::Node) {
                self.removed_nodes.push(node.id());
            }
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Article, |node| {
                node.set_children(&[LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(2), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        assert_eq!(graft_node.child_ids().next(), Some(subtree_node_id(0)));

        let old_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(old_root.role(), Role::Document);
        assert_eq!(old_root.parent_id(), Some(node_id(1)));

        let child = tree.state().node_by_id(subtree_node_id(1)).unwrap();
        assert_eq!(child.role(), Role::Article);
        assert_eq!(child.parent_id(), Some(subtree_node_id(0)));

        let grandchild = tree.state().node_by_id(subtree_node_id(2)).unwrap();
        assert_eq!(grandchild.parent_id(), Some(subtree_node_id(1)));

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        tree.update(subtree_id(), &mut handler, |update| {
            update.set_node(LocalNodeId(1), Role::Article, |node| {
                node.set_children(&[LocalNodeId(2)]);
            });
            update.set_node(LocalNodeId(2), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(1)));
            update.set_focus(LocalNodeId(1));
        });

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        let children: Vec<_> = graft_node.child_ids().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], subtree_node_id(1));

        let new_root = tree.state().node_by_id(subtree_node_id(1)).unwrap();
        assert_eq!(new_root.parent_id(), Some(node_id(1)));
        assert_eq!(new_root.role(), Role::Article);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none(),);

        let grandchild = tree.state().node_by_id(subtree_node_id(2)).unwrap();
        assert_eq!(grandchild.parent_id(), Some(subtree_node_id(1)));

        assert!(handler.removed_nodes.contains(&subtree_node_id(0)),);
        assert!(handler.updated_nodes.contains(&subtree_node_id(1)),);
        assert!(!handler.added_nodes.contains(&subtree_node_id(1)),);
        assert!(!handler.added_nodes.contains(&subtree_node_id(2)),);
    }

    #[test]
    fn subtree_root_change_to_new_parent_of_old_root() {
        struct Handler {
            updated_nodes: Vec<NodeId>,
            added_nodes: Vec<NodeId>,
            removed_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::Node, new: &crate::Node) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, node: &crate::Node) {
                self.removed_nodes.push(node.id());
            }
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        tree.update(subtree_id(), &mut handler, |update| {
            update.set_node(LocalNodeId(2), Role::Article, |node| {
                node.set_children(&[LocalNodeId(0)]);
            });
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |_| ());
            update.set_tree(Tree::new(LocalNodeId(2)));
            update.set_focus(LocalNodeId(2));
        });

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        let children: Vec<_> = graft_node.child_ids().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], subtree_node_id(2));

        let new_root = tree.state().node_by_id(subtree_node_id(2)).unwrap();
        assert_eq!(new_root.parent_id(), Some(node_id(1)));
        assert_eq!(new_root.role(), Role::Article);

        let old_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(old_root.parent_id(), Some(subtree_node_id(2)));
        assert_eq!(old_root.role(), Role::Document);

        let grandchild = tree.state().node_by_id(subtree_node_id(1)).unwrap();
        assert_eq!(grandchild.parent_id(), Some(subtree_node_id(0)));

        assert!(handler.added_nodes.contains(&subtree_node_id(2)));
        assert!(handler.updated_nodes.contains(&subtree_node_id(0)));
        assert!(!handler.removed_nodes.contains(&subtree_node_id(0)));
        assert!(!handler.removed_nodes.contains(&subtree_node_id(1)));
    }

    #[test]
    fn subtree_update_without_tree_preserves_root() {
        struct Handler {
            updated_nodes: Vec<NodeId>,
            added_nodes: Vec<NodeId>,
            removed_nodes: Vec<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::Node, new: &crate::Node) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, node: &crate::Node) {
                self.removed_nodes.push(node.id());
            }
        }

        let mut tree = super::Tree::new(false, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::GenericContainer, |node| {
                node.set_tree_id(subtree_id());
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        tree.update(subtree_id(), &mut NoOpHandler, |update| {
            update.set_node(LocalNodeId(0), Role::Document, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Paragraph, |node| {
                node.set_label("original");
            });
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(0));
        });

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        tree.update(subtree_id(), &mut handler, |update| {
            update.set_node(LocalNodeId(1), Role::Paragraph, |node| {
                node.set_label("modified");
            });
            update.set_focus(LocalNodeId(0));
        });

        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.role(), Role::Document);
        assert_eq!(subtree_root.parent_id(), Some(node_id(1)));

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        assert_eq!(graft_node.child_ids().next(), Some(subtree_node_id(0)));

        let child = tree.state().node_by_id(subtree_node_id(1)).unwrap();
        assert_eq!(child.label().as_deref(), Some("modified"));

        assert!(handler.removed_nodes.is_empty(),);
        assert!(handler.added_nodes.is_empty());
        assert!(handler.updated_nodes.contains(&subtree_node_id(1)),);
        assert!(!handler.updated_nodes.contains(&subtree_node_id(0)),);
    }

    #[test]
    fn focus_returns_focused_node() {
        let tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::Button, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });
        assert_eq!(tree.state().focus().unwrap().id(), node_id(1));
    }

    #[test]
    fn focus_returns_active_descendant() {
        let tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::ListBox, |node| {
                node.set_children(&[LocalNodeId(2)]);
                node.set_active_descendant(LocalNodeId(2));
            });
            update.set_node(LocalNodeId(2), Role::ListBoxOption, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });
        assert_eq!(tree.state().focus().unwrap().id(), node_id(2));
    }

    #[test]
    fn focus_moved_when_active_descendant_changes() {
        let mut tree = super::Tree::new(true, |update| {
            update.set_node(LocalNodeId(0), Role::Window, |node| {
                node.set_children(&[LocalNodeId(1)]);
            });
            update.set_node(LocalNodeId(1), Role::ListBox, |node| {
                node.set_children(&[LocalNodeId(2), LocalNodeId(3)]);
                node.set_active_descendant(LocalNodeId(2));
            });
            update.set_node(LocalNodeId(2), Role::ListBoxOption, |_| ());
            update.set_node(LocalNodeId(3), Role::ListBoxOption, |_| ());
            update.set_tree(Tree::new(LocalNodeId(0)));
            update.set_focus(LocalNodeId(1));
        });

        struct Handler {
            focus_moved_called: bool,
            old_focus: Option<NodeId>,
            new_focus: Option<NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, old: Option<&crate::Node>, new: Option<&crate::Node>) {
                self.focus_moved_called = true;
                self.old_focus = old.map(|n| n.id());
                self.new_focus = new.map(|n| n.id());
            }
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let mut handler = Handler {
            focus_moved_called: false,
            old_focus: None,
            new_focus: None,
        };

        tree.update(TreeId::ROOT, &mut handler, |update| {
            update.set_node(LocalNodeId(1), Role::ListBox, |node| {
                node.set_children(&[LocalNodeId(2), LocalNodeId(3)]);
                node.set_active_descendant(LocalNodeId(3));
            });
            update.set_focus(LocalNodeId(1));
        });

        assert!(handler.focus_moved_called);
        assert_eq!(handler.old_focus, Some(node_id(2)));
        assert_eq!(handler.new_focus, Some(node_id(3)));
    }
}
