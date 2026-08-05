// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Node, NodeId, TreeId, TreeInfo, TreeUpdate};
use alloc::{vec, vec::Vec};
use core::fmt;
use hashbrown::{HashMap, HashSet};

use crate::node::{FullNodeId, NodeRef, NodeState, ParentAndIndex};

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
    pub(crate) root: FullNodeId,
    pub(crate) focus: FullNodeId,
}

#[derive(Clone, Debug)]
pub struct TreeState {
    pub(crate) nodes: HashMap<FullNodeId, NodeState>,
    pub(crate) info: TreeInfo,
    pub(crate) root: FullNodeId,
    pub(crate) focus: FullNodeId,
    is_host_focused: bool,
    pub(crate) subtrees: HashMap<TreeId, SubtreeState>,
    pub(crate) graft_parents: HashMap<TreeId, FullNodeId>,
    tree_index_map: TreeIndexMap,
}

#[derive(Default)]
struct InternalChanges {
    added_node_ids: HashSet<FullNodeId>,
    updated_node_ids: HashSet<FullNodeId>,
    removed_node_ids: HashSet<FullNodeId>,
}

impl TreeState {
    fn validate_global(&self) {
        if !self.nodes.contains_key(&self.root) {
            panic!("Root ID {:?} is not in the node list", self.info.root);
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
    fn compute_effective_focus(&self) -> FullNodeId {
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

    fn update(
        &mut self,
        update: TreeUpdate,
        is_host_focused: bool,
        mut changes: Option<&mut InternalChanges>,
    ) {
        let tree_index = self.tree_index_map.get_or_create_index(update.tree_id);
        let map_id = |id: NodeId| FullNodeId::new(id, tree_index);

        let mut unreachable = HashSet::new();
        let mut seen_child_ids = HashSet::new();

        let tree_id = update.tree_id;
        if tree_id != TreeId::ROOT {
            let subtree_exists = self.subtrees.contains_key(&tree_id);
            if update.tree.is_some() && !self.graft_parents.contains_key(&tree_id) {
                panic!(
                    "Cannot push subtree {:?}: no graft node exists for this tree. \
                     Push the graft node (with tree_id property set) before pushing the subtree.",
                    tree_id
                );
            }
            if !subtree_exists && update.tree.is_none() {
                panic!(
                    "Cannot update subtree {:?}: subtree does not exist. \
                     The first update for a subtree must include tree data.",
                    tree_id
                );
            }
        }

        let new_tree_root = if let Some(tree) = update.tree {
            let new_root = map_id(tree.root);
            if tree_id == TreeId::ROOT {
                if tree.root != self.info.root {
                    unreachable.insert(self.root);
                }
                self.root = new_root;
                self.info = tree;
            } else if let Some(subtree) = self.subtrees.get(&tree_id) {
                if subtree.root != new_root {
                    unreachable.insert(subtree.root);
                }
            }
            Some(new_root)
        } else {
            None
        };

        let root = new_tree_root
            .map(|r| r.to_components().0)
            .unwrap_or_else(|| {
                self.subtrees
                    .get(&tree_id)
                    .map(|s| s.root.to_components().0)
                    .unwrap_or(self.info.root)
            });

        let mut pending_nodes: HashMap<FullNodeId, _> = HashMap::new();
        let mut pending_children = HashMap::new();
        let mut pending_grafts: HashMap<TreeId, FullNodeId> = HashMap::new();
        let mut grafts_to_remove: HashSet<TreeId> = HashSet::new();

        fn record_graft(
            pending_grafts: &mut HashMap<TreeId, FullNodeId>,
            subtree_id: TreeId,
            graft_node_id: FullNodeId,
        ) {
            if subtree_id == TreeId::ROOT {
                panic!("Cannot graft the root tree");
            }
            if let Some(existing_graft) = pending_grafts.get(&subtree_id) {
                panic!(
                    "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
                    subtree_id,
                    existing_graft.to_components().0,
                    graft_node_id.to_components().0
                );
            }
            pending_grafts.insert(subtree_id, graft_node_id);
        }

        fn add_node(
            nodes: &mut HashMap<FullNodeId, NodeState>,
            pending_grafts: &mut HashMap<TreeId, FullNodeId>,
            changes: &mut Option<&mut InternalChanges>,
            parent_and_index: Option<ParentAndIndex>,
            id: FullNodeId,
            data: Node,
        ) {
            if let Some(subtree_id) = data.tree_id() {
                if !data.children().is_empty() {
                    panic!(
                        "Node {:?} has both tree_id and children. \
                         A graft node's only child comes from its subtree.",
                        id.to_components().0
                    );
                }
                record_graft(pending_grafts, subtree_id, id);
            }
            let state = NodeState {
                parent_and_index,
                data,
            };
            nodes.insert(id, state);
            if let Some(changes) = changes {
                changes.added_node_ids.insert(id);
            }
        }

        for (local_node_id, node_data) in update.nodes {
            let node_id = map_id(local_node_id);
            unreachable.remove(&node_id);

            for (child_index, child_id) in node_data.children().iter().enumerate() {
                let mapped_child_id = map_id(*child_id);
                if !seen_child_ids.insert(mapped_child_id) {
                    panic!("TreeUpdate includes duplicate child {:?}", child_id);
                }
                unreachable.remove(&mapped_child_id);
                let parent_and_index = ParentAndIndex(node_id, child_index);
                if let Some(child_state) = self.nodes.get_mut(&mapped_child_id) {
                    if child_state.parent_and_index != Some(parent_and_index) {
                        child_state.parent_and_index = Some(parent_and_index);
                        if let Some(changes) = &mut changes {
                            changes.updated_node_ids.insert(mapped_child_id);
                        }
                    }
                } else if let Some(child_data) = pending_nodes.remove(&mapped_child_id) {
                    add_node(
                        &mut self.nodes,
                        &mut pending_grafts,
                        &mut changes,
                        Some(parent_and_index),
                        mapped_child_id,
                        child_data,
                    );
                } else {
                    pending_children.insert(mapped_child_id, parent_and_index);
                }
            }

            if let Some(node_state) = self.nodes.get_mut(&node_id) {
                if local_node_id == root {
                    node_state.parent_and_index = None;
                }
                for child_id in node_state.data.children().iter() {
                    let mapped_existing_child_id = map_id(*child_id);
                    if !seen_child_ids.contains(&mapped_existing_child_id) {
                        unreachable.insert(mapped_existing_child_id);
                    }
                }
                if node_state.data != node_data {
                    if node_data.tree_id().is_some() && !node_data.children().is_empty() {
                        panic!(
                            "Node {:?} has both tree_id and children. \
                             A graft node's only child comes from its subtree.",
                            node_id.to_components().0
                        );
                    }
                    let old_tree_id = node_state.data.tree_id();
                    let new_tree_id = node_data.tree_id();
                    if old_tree_id != new_tree_id {
                        if let Some(old_subtree_id) = old_tree_id {
                            grafts_to_remove.insert(old_subtree_id);
                        }
                        if let Some(new_subtree_id) = new_tree_id {
                            record_graft(&mut pending_grafts, new_subtree_id, node_id);
                        }
                    }
                    node_state.data.clone_from(&node_data);
                    if let Some(changes) = &mut changes {
                        changes.updated_node_ids.insert(node_id);
                    }
                }
            } else if let Some(parent_and_index) = pending_children.remove(&node_id) {
                add_node(
                    &mut self.nodes,
                    &mut pending_grafts,
                    &mut changes,
                    Some(parent_and_index),
                    node_id,
                    node_data,
                );
            } else if local_node_id == root {
                add_node(
                    &mut self.nodes,
                    &mut pending_grafts,
                    &mut changes,
                    None,
                    node_id,
                    node_data,
                );
            } else {
                pending_nodes.insert(node_id, node_data);
            }
        }

        if !pending_nodes.is_empty() {
            panic!(
                "TreeUpdate includes {} nodes which are neither in the current tree nor a child of another node from the update: {}",
                pending_nodes.len(),
                ShortNodeList(&pending_nodes)
            );
        }
        if !pending_children.is_empty() {
            panic!(
                "TreeUpdate's nodes include {} children ids which are neither in the current tree nor the ID of another node from the update: {}",
                pending_children.len(),
                ShortNodeList(&pending_children)
            );
        }

        let tree_focus = map_id(update.focus);
        if let Some(new_root) = new_tree_root {
            self.subtrees.insert(
                tree_id,
                SubtreeState {
                    root: new_root,
                    focus: tree_focus,
                },
            );
        } else if let Some(subtree) = self.subtrees.get_mut(&tree_id) {
            subtree.focus = tree_focus;
        } else if tree_id == TreeId::ROOT {
            self.subtrees.insert(
                tree_id,
                SubtreeState {
                    root: self.root,
                    focus: tree_focus,
                },
            );
        }

        self.is_host_focused = is_host_focused;

        if !unreachable.is_empty() {
            fn traverse_unreachable(
                nodes: &mut HashMap<FullNodeId, NodeState>,
                grafts_to_remove: &mut HashSet<TreeId>,
                changes: &mut Option<&mut InternalChanges>,
                seen_child_ids: &HashSet<FullNodeId>,
                new_tree_root: Option<FullNodeId>,
                root: FullNodeId,
            ) {
                let mut stack = vec![root];
                while let Some(id) = stack.pop() {
                    if let Some(changes) = changes {
                        changes.removed_node_ids.insert(id);
                    }
                    let node = nodes.remove(&id).unwrap();
                    if let Some(subtree_id) = node.data.tree_id() {
                        grafts_to_remove.insert(subtree_id);
                    }
                    let (_, tree_index) = id.to_components();
                    for child_id in node.data.children().iter() {
                        let child_node_id = FullNodeId::new(*child_id, tree_index);
                        if !seen_child_ids.contains(&child_node_id)
                            && new_tree_root != Some(child_node_id)
                        {
                            stack.push(child_node_id);
                        }
                    }
                }
            }

            for id in unreachable {
                traverse_unreachable(
                    &mut self.nodes,
                    &mut grafts_to_remove,
                    &mut changes,
                    &seen_child_ids,
                    new_tree_root,
                    id,
                );
            }
        }

        fn traverse_subtree(
            nodes: &mut HashMap<FullNodeId, NodeState>,
            subtrees_to_remove: &mut Vec<TreeId>,
            subtrees_queued: &mut HashSet<TreeId>,
            changes: &mut Option<&mut InternalChanges>,
            root: FullNodeId,
        ) {
            let mut stack = vec![root];
            while let Some(id) = stack.pop() {
                let Some(node) = nodes.remove(&id) else {
                    continue;
                };
                if let Some(changes) = changes {
                    changes.removed_node_ids.insert(id);
                }
                if let Some(nested_subtree_id) = node.data.tree_id() {
                    if subtrees_queued.insert(nested_subtree_id) {
                        subtrees_to_remove.push(nested_subtree_id);
                    }
                }
                let (_, tree_index) = id.to_components();
                for child_id in node.data.children().iter() {
                    stack.push(FullNodeId::new(*child_id, tree_index));
                }
            }
        }

        let mut subtrees_queued: HashSet<TreeId> = grafts_to_remove;
        let mut subtrees_to_remove: Vec<TreeId> = subtrees_queued.iter().copied().collect();
        let mut i = 0;
        while i < subtrees_to_remove.len() {
            let subtree_id = subtrees_to_remove[i];
            i += 1;

            if self.graft_parents.remove(&subtree_id).is_none() {
                continue;
            }

            if pending_grafts.contains_key(&subtree_id) {
                continue;
            }
            if let Some(subtree) = self.subtrees.remove(&subtree_id) {
                traverse_subtree(
                    &mut self.nodes,
                    &mut subtrees_to_remove,
                    &mut subtrees_queued,
                    &mut changes,
                    subtree.root,
                );
            }
        }

        for (subtree_id, node_id) in pending_grafts {
            if let Some(&existing_graft) = self.graft_parents.get(&subtree_id) {
                panic!(
                    "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
                    subtree_id,
                    existing_graft.to_components().0,
                    node_id.to_components().0
                );
            }
            self.graft_parents.insert(subtree_id, node_id);
            if let Some(subtree) = self.subtrees.get(&subtree_id) {
                let subtree_root_id = subtree.root;
                if let Some(root_state) = self.nodes.get_mut(&subtree_root_id) {
                    root_state.parent_and_index = Some(ParentAndIndex(node_id, 0));
                    if let Some(changes) = &mut changes {
                        if !changes.added_node_ids.contains(&subtree_root_id) {
                            changes.updated_node_ids.insert(subtree_root_id);
                        }
                    }
                }
            }
        }

        if let Some(new_root_id) = new_tree_root {
            if let Some(&graft_node_id) = self.graft_parents.get(&tree_id) {
                if let Some(root_state) = self.nodes.get_mut(&new_root_id) {
                    root_state.parent_and_index = Some(ParentAndIndex(graft_node_id, 0));
                    if let Some(changes) = &mut changes {
                        if !changes.added_node_ids.contains(&new_root_id) {
                            changes.updated_node_ids.insert(new_root_id);
                        }
                    }
                }
            }
        }

        self.focus = self.compute_effective_focus();

        self.validate_global();
    }

    fn update_host_focus_state(
        &mut self,
        is_host_focused: bool,
        changes: Option<&mut InternalChanges>,
    ) {
        let (focus, _) = self.focus.to_components();
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus,
        };
        self.update(update, is_host_focused, changes);
    }

    pub fn has_node(&self, id: FullNodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn node_by_id(&self, id: FullNodeId) -> Option<NodeRef<'_>> {
        self.nodes.get(&id).map(|node_state| NodeRef {
            tree_state: self,
            id,
            state: node_state,
        })
    }

    pub fn node_by_tree_local_id(
        &self,
        local_node_id: NodeId,
        tree_id: TreeId,
    ) -> Option<NodeRef<'_>> {
        let tree_index = self.tree_index_map.get_index(tree_id)?;
        self.node_by_id(FullNodeId::new(local_node_id, tree_index))
    }

    pub fn root_id(&self) -> FullNodeId {
        self.root
    }

    pub fn root(&self) -> NodeRef<'_> {
        self.node_by_id(self.root_id()).unwrap()
    }

    /// Returns the root NodeId of the subtree with the given TreeId, if it exists.
    pub fn subtree_root(&self, tree_id: TreeId) -> Option<FullNodeId> {
        self.subtrees.get(&tree_id).map(|s| s.root)
    }

    pub fn is_host_focused(&self) -> bool {
        self.is_host_focused
    }

    pub fn focus_id_in_tree(&self) -> FullNodeId {
        self.focus
    }

    pub fn focus_in_tree(&self) -> NodeRef<'_> {
        self.node_by_id(self.focus_id_in_tree()).unwrap()
    }

    pub fn focus_id(&self) -> Option<FullNodeId> {
        self.is_host_focused.then_some(self.focus)
    }

    pub fn focus(&self) -> Option<NodeRef<'_>> {
        self.focus_id().map(|id| {
            let focused = self.node_by_id(id).unwrap();
            focused.active_descendant().unwrap_or(focused)
        })
    }

    pub fn active_dialog(&self) -> Option<NodeRef<'_>> {
        let mut node = self.focus();
        while let Some(candidate) = node {
            if candidate.is_dialog() {
                return Some(candidate);
            }
            node = candidate.parent();
        }
        None
    }

    pub fn toolkit_name(&self) -> &str {
        self.info.toolkit_name.as_deref().unwrap_or("AccessKit")
    }

    pub fn toolkit_version(&self) -> Option<&str> {
        self.info
            .toolkit_version
            .as_deref()
            .filter(|_| self.info.toolkit_name.is_some())
    }

    pub fn locate_node(&self, node_id: FullNodeId) -> Option<(NodeId, TreeId)> {
        if !self.has_node(node_id) {
            return None;
        }
        let (local_id, tree_index) = node_id.to_components();
        self.tree_index_map
            .get_id(tree_index)
            .map(|tree_id| (local_id, tree_id))
    }
}

pub trait ChangeHandler {
    fn node_added(&mut self, node: &NodeRef);
    fn node_updated(&mut self, old_node: &NodeRef, new_node: &NodeRef);
    fn focus_moved(&mut self, old_node: Option<&NodeRef>, new_node: Option<&NodeRef>);
    fn node_removed(&mut self, node: &NodeRef);
}

#[derive(Debug)]
pub struct Tree {
    state: TreeState,
    next_state: TreeState,
}

impl Tree {
    pub fn new(mut initial_state: TreeUpdate, is_host_focused: bool) -> Self {
        let Some(tree) = initial_state.tree.take() else {
            panic!(
                "Tried to initialize the accessibility tree without a root tree. TreeUpdate::tree must be Some."
            );
        };
        if initial_state.tree_id != TreeId::ROOT {
            panic!("Cannot initialize with a subtree. TreeUpdate::tree_id must be TreeId::ROOT.");
        }
        let mut tree_index_map = TreeIndexMap::default();
        let tree_index = tree_index_map.get_or_create_index(initial_state.tree_id);
        let mut state = TreeState {
            nodes: HashMap::new(),
            root: FullNodeId::new(tree.root, tree_index),
            info: tree,
            focus: FullNodeId::new(initial_state.focus, tree_index),
            is_host_focused,
            subtrees: HashMap::new(),
            graft_parents: HashMap::new(),
            tree_index_map,
        };
        state.update(initial_state, is_host_focused, None);
        Self {
            next_state: state.clone(),
            state,
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
        if self.state.info != self.next_state.info {
            self.state.info.clone_from(&self.next_state.info);
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

    pub fn state(&self) -> &TreeState {
        &self.state
    }
}

struct ShortNodeList<'a, T>(&'a HashMap<FullNodeId, T>);

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
    use accesskit::{Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate, Uuid};
    use alloc::{vec, vec::Vec};

    use super::{TreeIndex, TreeIndexMap};
    use crate::node::FullNodeId;

    struct NoOpHandler;
    impl super::ChangeHandler for NoOpHandler {
        fn node_added(&mut self, _: &crate::NodeRef) {}
        fn node_updated(&mut self, _: &crate::NodeRef, _: &crate::NodeRef) {}
        fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
        fn node_removed(&mut self, _: &crate::NodeRef) {}
    }

    fn node_id(n: u64) -> FullNodeId {
        FullNodeId::new(NodeId(n), TreeIndex(0))
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
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);
        assert_eq!(node_id(0), tree.state().root().id());
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
            tree: Some(TreeInfo::new(NodeId(0))),
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
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);
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
        let root_node = Node::new(Role::Window);
        let first_update = TreeUpdate {
            nodes: vec![(NodeId(0), root_node.clone())],
            tree: Some(TreeInfo::new(NodeId(0))),
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
            fn node_added(&mut self, node: &crate::NodeRef) {
                if node.id() == node_id(1) {
                    self.got_new_child_node = true;
                    return;
                }
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::NodeRef, new_node: &crate::NodeRef) {
                if new_node.id() == node_id(0)
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
                _old_node: Option<&crate::NodeRef>,
                _new_node: Option<&crate::NodeRef>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::NodeRef) {
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
        assert_eq!(node_id(1), state.root().children().next().unwrap().id());
        assert_eq!(
            node_id(0),
            state.node_by_id(node_id(1)).unwrap().parent().unwrap().id()
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
            tree: Some(TreeInfo::new(NodeId(0))),
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
            fn node_added(&mut self, _node: &crate::NodeRef) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::NodeRef, new_node: &crate::NodeRef) {
                if new_node.id() == node_id(0)
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
                _old_node: Option<&crate::NodeRef>,
                _new_node: Option<&crate::NodeRef>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, node: &crate::NodeRef) {
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
        tree.update_and_process_changes(second_update, &mut handler);
        assert!(handler.got_updated_root_node);
        assert!(handler.got_removed_child_node);
        assert_eq!(0, tree.state().root().children().count());
        assert!(tree.state().node_by_id(node_id(1)).is_none());
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
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let mut tree = super::Tree::new(first_update, true);
        assert!(tree.state().node_by_id(node_id(1)).unwrap().is_focused());
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
            fn node_added(&mut self, _node: &crate::NodeRef) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::NodeRef, new_node: &crate::NodeRef) {
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
                old_node: Option<&crate::NodeRef>,
                new_node: Option<&crate::NodeRef>,
            ) {
                if let (Some(old_node), Some(new_node)) = (old_node, new_node) {
                    if old_node.id() == node_id(1) && new_node.id() == node_id(2) {
                        self.got_focus_change = true;
                        return;
                    }
                }
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::NodeRef) {
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
        assert!(tree.state().node_by_id(node_id(2)).unwrap().is_focused());
        assert!(!tree.state().node_by_id(node_id(1)).unwrap().is_focused());
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
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(
            Some("foo".into()),
            tree.state().node_by_id(node_id(1)).unwrap().label()
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
            fn node_added(&mut self, _node: &crate::NodeRef) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::NodeRef, new_node: &crate::NodeRef) {
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
                _old_node: Option<&crate::NodeRef>,
                _new_node: Option<&crate::NodeRef>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::NodeRef) {
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
            tree.state().node_by_id(node_id(1)).unwrap().label()
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
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update.clone(), false);
        struct Handler;
        fn unexpected_change() {
            panic!("expected no changes");
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _node: &crate::NodeRef) {
                unexpected_change();
            }
            fn node_updated(&mut self, _old_node: &crate::NodeRef, _new_node: &crate::NodeRef) {
                unexpected_change();
            }
            fn focus_moved(
                &mut self,
                _old_node: Option<&crate::NodeRef>,
                _new_node: Option<&crate::NodeRef>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, _node: &crate::NodeRef) {
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
            fn node_added(&mut self, _node: &crate::NodeRef) {
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::NodeRef, new_node: &crate::NodeRef) {
                if new_node.id() == node_id(0)
                    && old_node.child_ids().collect::<Vec<FullNodeId>>() == vec![node_id(1)]
                    && new_node.child_ids().collect::<Vec<FullNodeId>>() == vec![node_id(2)]
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
                _old_node: Option<&crate::NodeRef>,
                _new_node: Option<&crate::NodeRef>,
            ) {
                unexpected_change();
            }
            fn node_removed(&mut self, node: &crate::NodeRef) {
                if node.id() == node_id(1) {
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
            tree: Some(TreeInfo::new(NodeId(0))),
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
                .node_by_id(node_id(0))
                .unwrap()
                .child_ids()
                .collect::<Vec<FullNodeId>>(),
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
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );
    }

    #[test]
    #[should_panic(expected = "already has a graft parent")]
    fn duplicate_graft_parent_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (NodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let _ = super::Tree::new(update, false);
    }

    #[test]
    fn reparent_subtree_by_removing_old_graft() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (NodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(2)]);
                    node
                }),
                (NodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(2))
        );
    }

    #[test]
    fn reparent_subtree_by_clearing_old_graft_tree_id() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (NodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(1), Node::new(Role::GenericContainer)),
                (NodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(2))
        );
    }

    #[test]
    #[should_panic(expected = "already has a graft parent")]
    fn duplicate_graft_parent_on_update_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (NodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let update = TreeUpdate {
            nodes: vec![(NodeId(2), {
                let mut node = Node::new(Role::GenericContainer);
                node.set_tree_id(subtree_id());
                node
            })],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
    }

    #[test]
    #[should_panic(expected = "Cannot graft the root tree")]
    fn graft_root_tree_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(TreeId::ROOT);
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let _ = super::Tree::new(update, false);
    }

    #[test]
    #[should_panic(expected = "Cannot graft the root tree")]
    fn graft_root_tree_on_update_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::GenericContainer)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let update = TreeUpdate {
            nodes: vec![(NodeId(1), {
                let mut node = Node::new(Role::GenericContainer);
                node.set_tree_id(TreeId::ROOT);
                node
            })],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
    }

    fn subtree_node_id(id: u64) -> FullNodeId {
        FullNodeId::new(NodeId(id), TreeIndex(1))
    }

    #[test]
    fn node_by_tree_local_id_finds_root_tree_node() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);

        let root = tree
            .state()
            .node_by_tree_local_id(NodeId(0), TreeId::ROOT)
            .unwrap();
        assert_eq!(root.id(), node_id(0));
        assert_eq!(root.role(), Role::Window);

        let child = tree
            .state()
            .node_by_tree_local_id(NodeId(1), TreeId::ROOT)
            .unwrap();
        assert_eq!(child.id(), node_id(1));
        assert_eq!(child.role(), Role::Button);
    }

    #[test]
    fn node_by_tree_local_id_finds_subtree_node() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let sub_root = tree
            .state()
            .node_by_tree_local_id(NodeId(0), subtree_id())
            .unwrap();
        assert_eq!(sub_root.id(), subtree_node_id(0));
        assert_eq!(sub_root.role(), Role::Document);

        let sub_child = tree
            .state()
            .node_by_tree_local_id(NodeId(1), subtree_id())
            .unwrap();
        assert_eq!(sub_child.id(), subtree_node_id(1));
        assert_eq!(sub_child.role(), Role::Paragraph);

        let graft = tree
            .state()
            .node_by_tree_local_id(NodeId(1), TreeId::ROOT)
            .unwrap();
        assert_eq!(graft.id(), node_id(1));
        assert_eq!(graft.role(), Role::GenericContainer);
    }

    #[test]
    fn node_by_tree_local_id_returns_none_for_unknown_tree_id() {
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);

        assert!(
            tree.state()
                .node_by_tree_local_id(NodeId(0), subtree_id())
                .is_none()
        );
    }

    #[test]
    fn node_by_tree_local_id_returns_none_for_unknown_local_id() {
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);

        assert!(
            tree.state()
                .node_by_tree_local_id(NodeId(999), TreeId::ROOT)
                .is_none()
        );
    }

    #[test]
    fn subtree_root_parent_is_graft_when_graft_exists_first() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

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
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);
    }

    #[test]
    #[should_panic(expected = "subtree does not exist")]
    fn subtree_update_without_tree_data_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: None,
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);
    }

    #[test]
    fn subtree_nodes_removed_when_graft_removed() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: nested_subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());
        assert!(tree.state().node_by_id(nested_subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(nested_subtree_node_id(1)).is_some());

        let update = TreeUpdate {
            nodes: vec![(NodeId(0), {
                let mut node = Node::new(Role::Window);
                node.set_children(vec![]);
                node
            })],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());
        assert!(tree.state().node_by_id(nested_subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(nested_subtree_node_id(1)).is_none());
        assert!(tree.state().subtrees.get(&subtree_id()).is_none());
        assert!(tree.state().subtrees.get(&nested_subtree_id()).is_none());
    }

    #[test]
    fn subtree_nodes_removed_when_graft_tree_id_cleared() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        let update = TreeUpdate {
            nodes: vec![(NodeId(1), Node::new(Role::GenericContainer))],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());
        assert!(tree.state().subtrees.get(&subtree_id()).is_none());
    }

    #[test]
    fn graft_node_has_no_children_when_subtree_not_pushed() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let tree = super::Tree::new(update, false);

        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        assert_eq!(graft_node.child_ids().count(), 0);
        assert_eq!(graft_node.children().count(), 0);
    }

    #[test]
    #[should_panic(expected = "has both tree_id")]
    fn graft_node_with_children_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node.set_children(vec![NodeId(2)]);
                    node
                }),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        super::Tree::new(update, false);
    }

    #[test]
    fn node_added_called_when_subtree_pushed() {
        struct Handler {
            added_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::NodeRef) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _: &crate::NodeRef, _: &crate::NodeRef) {}
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, _: &crate::NodeRef) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let mut handler = Handler {
            added_nodes: Vec::new(),
        };

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

        assert_eq!(handler.added_nodes.len(), 3,);
        assert!(handler.added_nodes.contains(&subtree_node_id(0)),);
        assert!(handler.added_nodes.contains(&subtree_node_id(1)),);
        assert!(handler.added_nodes.contains(&subtree_node_id(2)),);
    }

    #[test]
    fn node_removed_called_when_graft_removed() {
        struct Handler {
            removed_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::NodeRef) {}
            fn node_updated(&mut self, _: &crate::NodeRef, _: &crate::NodeRef) {}
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, node: &crate::NodeRef) {
                self.removed_nodes.push(node.id());
            }
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        let mut handler = Handler {
            removed_nodes: Vec::new(),
        };

        let update = TreeUpdate {
            nodes: vec![(NodeId(0), {
                let mut node = Node::new(Role::Window);
                node.set_children(vec![]);
                node
            })],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut handler);

        assert!(handler.removed_nodes.contains(&node_id(1)),);
        assert!(handler.removed_nodes.contains(&subtree_node_id(0)),);
        assert!(handler.removed_nodes.contains(&subtree_node_id(1)),);
        assert_eq!(handler.removed_nodes.len(), 3,);
    }

    #[test]
    fn node_updated_called_when_subtree_reparented() {
        struct Handler {
            updated_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::NodeRef) {}
            fn node_updated(&mut self, _old: &crate::NodeRef, new: &crate::NodeRef) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, _: &crate::NodeRef) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (NodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent().unwrap().id(), node_id(1));

        let mut handler = Handler {
            updated_nodes: Vec::new(),
        };

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(1), Node::new(Role::GenericContainer)),
                (NodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        tree.update_and_process_changes(update, &mut handler);

        assert!(handler.updated_nodes.contains(&subtree_node_id(0)),);

        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent().unwrap().id(), node_id(2));
    }

    #[test]
    fn focus_moved_called_when_focus_moves_to_subtree() {
        struct Handler {
            focus_moves: Vec<(Option<FullNodeId>, Option<FullNodeId>)>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::NodeRef) {}
            fn node_updated(&mut self, _: &crate::NodeRef, _: &crate::NodeRef) {}
            fn focus_moved(&mut self, old: Option<&crate::NodeRef>, new: Option<&crate::NodeRef>) {
                self.focus_moves
                    .push((old.map(|n| n.id()), new.map(|n| n.id())));
            }
            fn node_removed(&mut self, _: &crate::NodeRef) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let mut handler = Handler {
            focus_moves: Vec::new(),
        };

        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        tree.update_and_process_changes(update, &mut handler);

        assert_eq!(handler.focus_moves.len(), 1,);
        let (old_focus, new_focus) = &handler.focus_moves[0];
        assert_eq!(*old_focus, Some(node_id(0)),);
        assert_eq!(*new_focus, Some(subtree_node_id(0)),);
    }

    #[test]
    fn focus_moved_called_when_subtree_focus_changes() {
        struct Handler {
            focus_moves: Vec<(Option<FullNodeId>, Option<FullNodeId>)>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::NodeRef) {}
            fn node_updated(&mut self, _: &crate::NodeRef, _: &crate::NodeRef) {}
            fn focus_moved(&mut self, old: Option<&crate::NodeRef>, new: Option<&crate::NodeRef>) {
                self.focus_moves
                    .push((old.map(|n| n.id()), new.map(|n| n.id())));
            }
            fn node_removed(&mut self, _: &crate::NodeRef) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let root_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        tree.update_and_process_changes(root_update, &mut NoOpHandler);

        let mut handler = Handler {
            focus_moves: Vec::new(),
        };

        let subtree_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

        assert_eq!(handler.focus_moves.len(), 1,);
        let (old_focus, new_focus) = &handler.focus_moves[0];
        assert_eq!(*old_focus, Some(subtree_node_id(0)),);
        assert_eq!(*new_focus, Some(subtree_node_id(1)),);
    }

    fn nested_subtree_id() -> TreeId {
        TreeId(Uuid::from_u128(2))
    }

    fn nested_subtree_node_id(n: u64) -> FullNodeId {
        FullNodeId::new(NodeId(n), TreeIndex(2))
    }

    #[test]
    fn nested_subtree_focus_follows_graft_chain() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Group);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: nested_subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(1)),);
    }

    #[test]
    fn nested_subtree_focus_update_changes_effective_focus() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Group);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: nested_subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        let root_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        tree.update_and_process_changes(root_update, &mut NoOpHandler);

        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(1)));

        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: nested_subtree_id(),
            focus: NodeId(2),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(2)),);
    }

    #[test]
    #[should_panic(expected = "Graft nodes cannot be focused without their subtree")]
    fn removing_nested_subtree_while_intermediate_focus_on_graft_panics() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Button))],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: nested_subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        let update = TreeUpdate {
            nodes: vec![(NodeId(1), Node::new(Role::GenericContainer))],
            tree: None,
            tree_id: subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
    }

    #[test]
    fn nested_subtree_root_lookup_for_focus_only_update() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1), NodeId(2)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
                (NodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Group);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: nested_subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: subtree_id(),
            focus: NodeId(2),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        assert_eq!(
            tree.state().subtrees.get(&subtree_id()).unwrap().focus,
            subtree_node_id(2),
        );
    }

    #[test]
    fn subtree_root_change_updates_graft_and_parent() {
        struct Handler {
            updated_nodes: Vec<FullNodeId>,
            added_nodes: Vec<FullNodeId>,
            removed_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::NodeRef) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::NodeRef, new: &crate::NodeRef) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, node: &crate::NodeRef) {
                self.removed_nodes.push(node.id());
            }
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(2), {
                    let mut node = Node::new(Role::Article);
                    node.set_children(vec![NodeId(3)]);
                    node
                }),
                (NodeId(3), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(2))),
            tree_id: subtree_id(),
            focus: NodeId(2),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

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
            updated_nodes: Vec<FullNodeId>,
            added_nodes: Vec<FullNodeId>,
            removed_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::NodeRef) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::NodeRef, new: &crate::NodeRef) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, node: &crate::NodeRef) {
                self.removed_nodes.push(node.id());
            }
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::Article);
                    node.set_children(vec![NodeId(2)]);
                    node
                }),
                (NodeId(2), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

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

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(1), {
                    let mut node = Node::new(Role::Article);
                    node.set_children(vec![NodeId(2)]);
                    node
                }),
                (NodeId(2), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(1))),
            tree_id: subtree_id(),
            focus: NodeId(1),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

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
            updated_nodes: Vec<FullNodeId>,
            added_nodes: Vec<FullNodeId>,
            removed_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::NodeRef) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::NodeRef, new: &crate::NodeRef) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, node: &crate::NodeRef) {
                self.removed_nodes.push(node.id());
            }
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(2), {
                    let mut node = Node::new(Role::Article);
                    node.set_children(vec![NodeId(0)]);
                    node
                }),
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(TreeInfo::new(NodeId(2))),
            tree_id: subtree_id(),
            focus: NodeId(2),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

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
            updated_nodes: Vec<FullNodeId>,
            added_nodes: Vec<FullNodeId>,
            removed_nodes: Vec<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::NodeRef) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _old: &crate::NodeRef, new: &crate::NodeRef) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::NodeRef>, _: Option<&crate::NodeRef>) {}
            fn node_removed(&mut self, node: &crate::NodeRef) {
                self.removed_nodes.push(node.id());
            }
        }

        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::Paragraph);
                    node.set_label("original");
                    node
                }),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let mut handler = Handler {
            updated_nodes: Vec::new(),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
        };

        let subtree_update = TreeUpdate {
            nodes: vec![(NodeId(1), {
                let mut node = Node::new(Role::Paragraph);
                node.set_label("modified");
                node
            })],
            tree: None,
            tree_id: subtree_id(),
            focus: NodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

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
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let tree = super::Tree::new(update, true);
        assert_eq!(tree.state().focus().unwrap().id(), node_id(1));
    }

    #[test]
    fn focus_returns_active_descendant() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::ListBox);
                    node.set_children(vec![NodeId(2)]);
                    node.set_active_descendant(NodeId(2));
                    node
                }),
                (NodeId(2), Node::new(Role::ListBoxOption)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let tree = super::Tree::new(update, true);
        assert_eq!(tree.state().focus().unwrap().id(), node_id(2));
    }

    #[test]
    fn focus_moved_when_active_descendant_changes() {
        let update = TreeUpdate {
            nodes: vec![
                (NodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![NodeId(1)]);
                    node
                }),
                (NodeId(1), {
                    let mut node = Node::new(Role::ListBox);
                    node.set_children(vec![NodeId(2), NodeId(3)]);
                    node.set_active_descendant(NodeId(2));
                    node
                }),
                (NodeId(2), Node::new(Role::ListBoxOption)),
                (NodeId(3), Node::new(Role::ListBoxOption)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let mut tree = super::Tree::new(update, true);

        struct Handler {
            focus_moved_called: bool,
            old_focus: Option<FullNodeId>,
            new_focus: Option<FullNodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::NodeRef) {}
            fn node_updated(&mut self, _: &crate::NodeRef, _: &crate::NodeRef) {}
            fn focus_moved(&mut self, old: Option<&crate::NodeRef>, new: Option<&crate::NodeRef>) {
                self.focus_moved_called = true;
                self.old_focus = old.map(|n| n.id());
                self.new_focus = new.map(|n| n.id());
            }
            fn node_removed(&mut self, _: &crate::NodeRef) {}
        }

        let mut handler = Handler {
            focus_moved_called: false,
            old_focus: None,
            new_focus: None,
        };

        let update = TreeUpdate {
            nodes: vec![(NodeId(1), {
                let mut node = Node::new(Role::ListBox);
                node.set_children(vec![NodeId(2), NodeId(3)]);
                node.set_active_descendant(NodeId(3));
                node
            })],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        tree.update_and_process_changes(update, &mut handler);

        assert!(handler.focus_moved_called);
        assert_eq!(handler.old_focus, Some(node_id(2)));
        assert_eq!(handler.new_focus, Some(node_id(3)));
    }

    fn tree_with_toolkit_info(name: Option<&str>, version: Option<&str>) -> super::Tree {
        let mut tree_info = TreeInfo::new(NodeId(0));
        tree_info.toolkit_name = name.map(Into::into);
        tree_info.toolkit_version = version.map(Into::into);
        let update = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(tree_info),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        super::Tree::new(update, false)
    }

    #[test]
    fn toolkit_info_is_reported_when_both_name_and_version_are_set() {
        let tree = tree_with_toolkit_info(Some("egui"), Some("0.30.0"));
        let state = tree.state();
        assert_eq!("egui", state.toolkit_name());
        assert_eq!(Some("0.30.0"), state.toolkit_version());
    }

    #[test]
    fn toolkit_version_is_none_when_only_name_is_set() {
        let tree = tree_with_toolkit_info(Some("egui"), None);
        let state = tree.state();
        assert_eq!("egui", state.toolkit_name());
        assert_eq!(None, state.toolkit_version());
    }

    #[test]
    fn toolkit_name_falls_back_to_accesskit_when_unset() {
        let tree = tree_with_toolkit_info(None, None);
        let state = tree.state();
        assert_eq!("AccessKit", state.toolkit_name());
        assert_eq!(None, state.toolkit_version());
    }

    #[test]
    fn toolkit_version_is_suppressed_when_name_is_unset() {
        // A version without a name would be misreported as the version of AccessKit.
        let tree = tree_with_toolkit_info(None, Some("0.30.0"));
        let state = tree.state();
        assert_eq!("AccessKit", state.toolkit_name());
        assert_eq!(None, state.toolkit_version());
    }
}
