// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Node as NodeData, NodeId as NodeIdContent, Tree as TreeData, TreeId, TreeUpdate};
use alloc::{vec, vec::Vec};
use core::fmt;
use hashbrown::{HashMap, HashSet};

use crate::node::{Node, NodeId, NodeState, ParentAndIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TreeIndex(pub(crate) u32);

#[derive(Debug)]
struct TreeIndexMap {
    id_to_index: HashMap<TreeId, TreeIndex>,
    index_to_id: HashMap<TreeIndex, TreeId>,
    next: u32,
}

impl Default for TreeIndexMap {
    fn default() -> Self {
        let root_index = TreeIndex(0);
        let mut id_to_index = HashMap::new();
        let mut index_to_id = HashMap::new();
        id_to_index.insert(TreeId::ROOT, root_index);
        index_to_id.insert(root_index, TreeId::ROOT);
        Self {
            id_to_index,
            index_to_id,
            next: 1,
        }
    }
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
    /// Maps TreeId to the state of each subtree (root and focus).
    pub(crate) subtrees: HashMap<TreeId, SubtreeState>,
    /// Maps TreeId to the graft NodeId that owns it (reverse of the tree_id property on graft nodes).
    pub(crate) graft_parents: HashMap<TreeId, NodeId>,
}

#[derive(Default)]
struct InternalChanges {
    added_node_ids: HashSet<NodeId>,
    updated_node_ids: HashSet<NodeId>,
    removed_node_ids: HashSet<NodeId>,
}

impl State {
    fn validate_global(&self) {
        if !self.nodes.contains_key(&self.root) {
            panic!("Root ID {:?} is not in the node list", self.data.root);
        }
        if !self.nodes.contains_key(&self.focus) {
            panic!("Focused ID {:?} is not in the node list", self.focus);
        }
    }

    /// Computes the effective focus by following the graft chain from ROOT.
    /// If ROOT's focus is on a graft node, follows through to that subtree's focus,
    /// and continues recursively until reaching a non-graft node.
    fn compute_effective_focus(&self) -> NodeId {
        let Some(root_subtree) = self.subtrees.get(&TreeId::ROOT) else {
            // No ROOT subtree stored yet, return current focus
            return self.focus;
        };

        let mut current_focus = root_subtree.focus;
        // Follow graft chain to find effective focus
        loop {
            // Check if current focus node is a graft (has tree_id property)
            let Some(node_state) = self.nodes.get(&current_focus) else {
                break;
            };
            let Some(subtree_id) = node_state.data.tree_id() else {
                // Not a graft node, we've found the effective focus
                break;
            };
            // Current focus is a graft, follow into subtree's focus
            let Some(subtree) = self.subtrees.get(&subtree_id) else {
                // Subtree doesn't have focus set yet, stay on graft node
                break;
            };
            current_focus = subtree.focus;
        }
        current_focus
    }

    fn update(
        &mut self,
        update: TreeUpdate,
        is_host_focused: bool,
        mut changes: Option<&mut InternalChanges>,
        tree_index: TreeIndex,
    ) {
        let map_id = |id: NodeIdContent| NodeId::new(id, tree_index);

        let mut unreachable: HashSet<NodeId> = HashSet::new();
        let mut seen_child_ids: HashSet<NodeId> = HashSet::new();

        let tree_id = update.tree_id;

        // Validate: pushing a new subtree requires its graft node to already exist
        if tree_id != TreeId::ROOT
            && update.tree.is_some()
            && !self.graft_parents.contains_key(&tree_id)
        {
            panic!(
                "Cannot push subtree {:?}: no graft node exists for this tree. \
                 Push the graft node (with tree_id property set) before pushing the subtree.",
                tree_id
            );
        }
        // Track if a new tree is being pushed (for setting subtree root parent later)
        let update_root = update.tree.as_ref().map(|t| t.root);
        let new_tree_root = if let Some(tree) = update.tree {
            let new_root = map_id(tree.root);
            if tree_id == TreeId::ROOT {
                // Only update main tree root/data for ROOT tree
                if tree.root != self.data.root {
                    unreachable.insert(self.root);
                }
                self.root = new_root;
                self.data = tree;
            }
            Some(new_root)
        } else {
            None
        };

        // Use the tree's root from the update, or fallback to existing subtree/main tree root
        let root = update_root.unwrap_or_else(|| {
            self.subtrees
                .get(&tree_id)
                .map(|s| s.root.to_components().0)
                .unwrap_or(self.data.root)
        });
        let mut pending_nodes: HashMap<NodeId, _> = HashMap::new();
        let mut pending_children: HashMap<NodeId, ParentAndIndex> = HashMap::new();
        // Collect new graft assignments to validate after unreachable nodes are removed
        let mut pending_grafts: HashMap<TreeId, NodeId> = HashMap::new();
        // Collect subtrees to remove when their graft is removed or has tree_id cleared
        // Tuple: (subtree_id, graft_node_id) - graft_node_id is used for focus validation
        let mut subtrees_to_remove: Vec<(TreeId, NodeId)> = Vec::new();

        fn add_node(
            nodes: &mut HashMap<NodeId, NodeState>,
            pending_grafts: &mut HashMap<TreeId, NodeId>,
            changes: &mut Option<&mut InternalChanges>,
            parent_and_index: Option<ParentAndIndex>,
            id: NodeId,
            data: NodeData,
        ) {
            // If this node is a graft (has tree_id property), validate within this update
            if let Some(subtree_id) = data.tree_id() {
                if let Some(existing_graft) = pending_grafts.get(&subtree_id) {
                    panic!(
                        "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
                        subtree_id, existing_graft, id
                    );
                }
                pending_grafts.insert(subtree_id, id);
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

        for (node_id, node_data) in update.nodes {
            let mapped_node_id = map_id(node_id);
            unreachable.remove(&mapped_node_id);

            for (child_index, child_id) in node_data.children().iter().enumerate() {
                let mapped_child_id = map_id(*child_id);
                if seen_child_ids.contains(&mapped_child_id) {
                    panic!("TreeUpdate includes duplicate child {:?}", child_id);
                }
                seen_child_ids.insert(mapped_child_id);
                unreachable.remove(&mapped_child_id);
                let parent_and_index = ParentAndIndex(mapped_node_id, child_index);
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

            if let Some(node_state) = self.nodes.get_mut(&mapped_node_id) {
                if node_id == root {
                    node_state.parent_and_index = None;
                }
                for child_id in node_state.data.children().iter() {
                    let mapped_existing_child_id = map_id(*child_id);
                    if !seen_child_ids.contains(&mapped_existing_child_id) {
                        unreachable.insert(mapped_existing_child_id);
                    }
                }
                if node_state.data != node_data {
                    // Handle graft parent changes - defer validation until after unreachable nodes removed
                    let old_tree_id = node_state.data.tree_id();
                    let new_tree_id = node_data.tree_id();
                    if old_tree_id != new_tree_id {
                        // Remove old graft parent mapping if it existed
                        if let Some(old_subtree_id) = old_tree_id {
                            self.graft_parents.remove(&old_subtree_id);
                            // Mark subtree for removal (include graft node for focus validation)
                            subtrees_to_remove.push((old_subtree_id, mapped_node_id));
                        }
                        // Record new graft assignment for later validation
                        if let Some(new_subtree_id) = new_tree_id {
                            // Validate within this update
                            if let Some(existing_graft) = pending_grafts.get(&new_subtree_id) {
                                panic!(
                                    "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
                                    new_subtree_id, existing_graft, mapped_node_id
                                );
                            }
                            pending_grafts.insert(new_subtree_id, mapped_node_id);
                        }
                    }
                    node_state.data.clone_from(&node_data);
                    if let Some(changes) = &mut changes {
                        changes.updated_node_ids.insert(mapped_node_id);
                    }
                }
            } else if let Some(parent_and_index) = pending_children.remove(&mapped_node_id) {
                add_node(
                    &mut self.nodes,
                    &mut pending_grafts,
                    &mut changes,
                    Some(parent_and_index),
                    mapped_node_id,
                    node_data,
                );
            } else if node_id == root {
                add_node(
                    &mut self.nodes,
                    &mut pending_grafts,
                    &mut changes,
                    None,
                    mapped_node_id,
                    node_data,
                );
            } else {
                pending_nodes.insert(mapped_node_id, node_data);
            }
        }

        if !pending_nodes.is_empty() {
            panic!("TreeUpdate includes {} nodes which are neither in the current tree nor a child of another node from the update: {}", pending_nodes.len(), ShortNodeList(&pending_nodes));
        }
        if !pending_children.is_empty() {
            panic!("TreeUpdate's nodes include {} children ids which are neither in the current tree nor the ID of another node from the update: {}", pending_children.len(), ShortNodeList(&pending_children));
        }

        // Store subtree state (root and focus) per tree
        let tree_focus = map_id(update.focus);
        if let Some(new_root) = new_tree_root {
            // New tree: insert both root and focus
            self.subtrees.insert(
                tree_id,
                SubtreeState {
                    root: new_root,
                    focus: tree_focus,
                },
            );
        } else if let Some(subtree) = self.subtrees.get_mut(&tree_id) {
            // Existing tree: just update focus
            subtree.focus = tree_focus;
        } else if tree_id == TreeId::ROOT {
            // ROOT tree focus update without tree change (e.g., during Tree::new after take())
            // Use the main tree's root for the subtree state
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
                nodes: &mut HashMap<NodeId, NodeState>,
                graft_parents: &mut HashMap<TreeId, NodeId>,
                subtrees_to_remove: &mut Vec<(TreeId, NodeId)>,
                changes: &mut Option<&mut InternalChanges>,
                seen_child_ids: &HashSet<NodeId>,
                id: NodeId,
                map_id: impl Fn(NodeIdContent) -> NodeId + Copy,
            ) {
                if let Some(changes) = changes {
                    changes.removed_node_ids.insert(id);
                }
                let node = nodes.remove(&id).unwrap();
                // If this node was a graft, mark its subtree for removal
                if let Some(subtree_id) = node.data.tree_id() {
                    graft_parents.remove(&subtree_id);
                    subtrees_to_remove.push((subtree_id, id));
                }
                for child_id in node.data.children().iter() {
                    let mapped_child_id = map_id(*child_id);
                    if !seen_child_ids.contains(&mapped_child_id) {
                        traverse_unreachable(
                            nodes,
                            graft_parents,
                            subtrees_to_remove,
                            changes,
                            seen_child_ids,
                            mapped_child_id,
                            map_id,
                        );
                    }
                }
            }

            for id in unreachable {
                traverse_unreachable(
                    &mut self.nodes,
                    &mut self.graft_parents,
                    &mut subtrees_to_remove,
                    &mut changes,
                    &seen_child_ids,
                    id,
                    map_id,
                );
            }
        }

        // Remove all nodes from subtrees whose graft was removed or had tree_id cleared,
        // UNLESS the subtree is being reparented to a new graft in the same update
        for (subtree_id, graft_node_id) in subtrees_to_remove {
            // Skip removal if the subtree is being reparented to a new graft
            if pending_grafts.contains_key(&subtree_id) {
                continue;
            }
            if let Some(subtree) = self.subtrees.get(&subtree_id) {
                let subtree_root_id = subtree.root;
                let (_, subtree_tree_index) = subtree_root_id.to_components();
                let nodes_to_remove: Vec<NodeId> = self
                    .nodes
                    .keys()
                    .filter(|id| id.to_components().1 == subtree_tree_index)
                    .copied()
                    .collect();
                // Validate: focus must not be on a subtree node or on the graft node
                // (focus on graft means effective focus is in the subtree)
                // Check effective focus
                let focus_on_subtree = nodes_to_remove.contains(&self.focus);
                let focus_on_graft = self.focus == graft_node_id;
                // Also check if any subtree's focus points to the graft node
                // (handles nested subtrees where graft is focused by an intermediate tree)
                let any_subtree_focuses_graft =
                    self.subtrees.values().any(|s| s.focus == graft_node_id);
                if focus_on_subtree || focus_on_graft || any_subtree_focuses_graft {
                    panic!(
                        "Cannot remove subtree {:?}: focus is on a node in this subtree. \
                         Move focus to a node outside the subtree before removing the graft.",
                        subtree_id
                    );
                }
                for id in nodes_to_remove {
                    self.nodes.remove(&id);
                    if let Some(changes) = &mut changes {
                        changes.removed_node_ids.insert(id);
                    }
                }
                self.subtrees.remove(&subtree_id);
            }
        }

        // Now validate and apply pending graft assignments
        for (subtree_id, node_id) in pending_grafts {
            if let Some(existing_graft) = self.graft_parents.get(&subtree_id) {
                panic!(
                    "Subtree {:?} already has a graft parent {:?}, cannot assign to {:?}",
                    subtree_id, existing_graft, node_id
                );
            }
            self.graft_parents.insert(subtree_id, node_id);
            // If the subtree already exists, set its root's parent to the graft node
            if let Some(subtree) = self.subtrees.get(&subtree_id) {
                let subtree_root_id = subtree.root;
                if let Some(root_state) = self.nodes.get_mut(&subtree_root_id) {
                    root_state.parent_and_index = Some(ParentAndIndex(node_id, 0));
                    // Only mark as updated if not already in added_node_ids (node added in previous update)
                    if let Some(changes) = &mut changes {
                        if !changes.added_node_ids.contains(&subtree_root_id) {
                            changes.updated_node_ids.insert(subtree_root_id);
                        }
                    }
                }
            }
        }

        // If a new tree was pushed and a graft parent already exists, set the subtree root's parent
        if let Some(new_root_id) = new_tree_root {
            if let Some(&graft_node_id) = self.graft_parents.get(&tree_id) {
                if let Some(root_state) = self.nodes.get_mut(&new_root_id) {
                    root_state.parent_and_index = Some(ParentAndIndex(graft_node_id, 0));
                    // Only mark as updated if not already in added_node_ids (node added in previous update)
                    if let Some(changes) = &mut changes {
                        if !changes.added_node_ids.contains(&new_root_id) {
                            changes.updated_node_ids.insert(new_root_id);
                        }
                    }
                }
            }
        }

        // Compute effective focus by following graft chain from ROOT
        // (after all node additions/removals are complete)
        self.focus = self.compute_effective_focus();

        self.validate_global();
    }

    fn update_host_focus_state(
        &mut self,
        is_host_focused: bool,
        changes: Option<&mut InternalChanges>,
        tree_index: TreeIndex,
    ) {
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: self.focus_id_content(),
        };
        self.update(update, is_host_focused, changes, tree_index);
    }

    /// Returns the ROOT tree's focus as a NodeIdContent for use in TreeUpdate.
    fn focus_id_content(&self) -> NodeIdContent {
        self.subtrees[&TreeId::ROOT].focus.to_components().0
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

    /// Returns the root NodeId for a subtree, if it has been pushed.
    pub fn subtree_root(&self, tree_id: TreeId) -> Option<NodeId> {
        self.subtrees.get(&tree_id).map(|s| s.root)
    }

    pub fn root_id(&self) -> NodeId {
        self.root
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
        let mut tree_index_map = TreeIndexMap::default();
        let tree_id = initial_state.tree_id;
        let tree_index = tree_index_map.get_index(tree_id);
        let mut state = State {
            nodes: HashMap::new(),
            root: NodeId::new(tree.root, tree_index),
            data: tree,
            focus: NodeId::new(initial_state.focus, tree_index),
            is_host_focused,
            subtrees: HashMap::new(),
            graft_parents: HashMap::new(),
        };
        state.update(initial_state, is_host_focused, None, tree_index);
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
        let new_index = self.tree_index_map.get_index(update.tree_id);
        self.next_state.update(
            update,
            self.state.is_host_focused,
            Some(&mut changes),
            new_index,
        );
        self.process_changes(changes, handler);
    }

    pub fn update_host_focus_state_and_process_changes(
        &mut self,
        is_host_focused: bool,
        handler: &mut impl ChangeHandler,
    ) {
        let mut changes = InternalChanges::default();
        let (_, tree_index) = self.next_state.root.to_components();
        self.next_state
            .update_host_focus_state(is_host_focused, Some(&mut changes), tree_index);
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
        self.state.root = self.next_state.root;
        self.state.focus = self.next_state.focus;
        self.state.is_host_focused = self.next_state.is_host_focused;
        self.state.subtrees.clone_from(&self.next_state.subtrees);
        self.state
            .graft_parents
            .clone_from(&self.next_state.graft_parents);
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn locate_node(&self, node_id: NodeId) -> Option<(NodeIdContent, TreeId)> {
        self.state
            .has_node(node_id)
            .then(|| {
                let (node_id, tree_index) = node_id.to_components();
                self.tree_index_map
                    .get_id(tree_index)
                    .map(|tree_id| (node_id, tree_id))
            })
            .flatten()
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
    use accesskit::{Node, NodeId as SchemaNodeId, Role, Tree, TreeId, TreeUpdate, Uuid};
    use alloc::{vec, vec::Vec};

    fn tree_id() -> TreeId {
        TreeId(Uuid::nil())
    }

    fn node_id(n: u64) -> crate::node::NodeId {
        crate::node::NodeId::new(SchemaNodeId(n), crate::tree::TreeIndex(0))
    }

    #[test]
    fn init_tree_with_root_node() {
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = super::Tree::new(update, false);
        assert_eq!(node_id(0), tree.state().root().id());
        assert_eq!(Role::Window, tree.state().root().role());
        assert!(tree.state().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
            nodes: vec![(SchemaNodeId(0), root_node.clone())],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(0, tree.state().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = root_node;
                    node.push_child(SchemaNodeId(1));
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::RootWebArea)),
            ],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
                if node.id() == node_id(1) {
                    self.got_new_child_node = true;
                    return;
                }
                unexpected_change();
            }
            fn node_updated(&mut self, old_node: &crate::Node, new_node: &crate::Node) {
                if new_node.id() == node_id(0)
                    && old_node.data().children().is_empty()
                    && new_node.data().children() == [SchemaNodeId(1)]
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
                (SchemaNodeId(0), {
                    let mut node = root_node.clone();
                    node.push_child(SchemaNodeId(1));
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::RootWebArea)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(1, tree.state().root().children().count());
        let second_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), root_node)],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
                if new_node.id() == node_id(0)
                    && old_node.data().children() == [SchemaNodeId(1)]
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
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(1),
        };
        let mut tree = super::Tree::new(first_update, true);
        assert!(tree.state().node_by_id(node_id(1)).unwrap().is_focused());
        let second_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(2),
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
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = child_node.clone();
                    node.set_label("foo");
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(first_update, false);
        assert_eq!(
            Some("foo".into()),
            tree.state().node_by_id(node_id(1)).unwrap().label()
        );
        let second_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(1), {
                let mut node = child_node;
                node.set_label("bar");
                node
            })],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::Button);
                    node.set_label("foo");
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
                if new_node.id() == node_id(0)
                    && old_node.child_ids().collect::<Vec<crate::node::NodeId>>()
                        == vec![node_id(1)]
                    && new_node.child_ids().collect::<Vec<crate::node::NodeId>>()
                        == vec![node_id(2)]
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

        let mut root = Node::new(Role::Window);
        root.set_children([SchemaNodeId(1)]);
        let mut container = Node::new(Role::GenericContainer);
        container.set_children([SchemaNodeId(2)]);
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), root.clone()),
                (SchemaNodeId(1), container),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = crate::Tree::new(update, false);
        root.set_children([SchemaNodeId(2)]);
        let mut handler = Handler {
            got_updated_root: false,
            got_updated_child: false,
            got_removed_container: false,
        };
        tree.update_and_process_changes(
            TreeUpdate {
                nodes: vec![(SchemaNodeId(0), root)],
                tree: None,
                tree_id: tree_id(),
                focus: SchemaNodeId(0),
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
                .collect::<Vec<crate::node::NodeId>>(),
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
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (SchemaNodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id()); // Same subtree_id - should panic
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let _ = super::Tree::new(update, false);
    }

    #[test]
    fn reparent_subtree_by_removing_old_graft() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );

        // Remove old graft and assign to new node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(2)]); // Remove node 1
                    node
                }),
                (SchemaNodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id()); // Now node 2 is the graft
                    node
                }),
            ],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(2))
        );
    }

    #[test]
    fn reparent_subtree_by_clearing_old_graft_tree_id() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);
        assert_eq!(
            tree.state().graft_parents.get(&subtree_id()),
            Some(&node_id(1))
        );

        // Clear tree_id from old graft and assign to new node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(1), Node::new(Role::GenericContainer)), // Clear tree_id
                (SchemaNodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id()); // Now node 2 is the graft
                    node
                }),
            ],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
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
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::GenericContainer)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Try to make node 2 also a graft for the same subtree - should panic
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(2), {
                let mut node = Node::new(Role::GenericContainer);
                node.set_tree_id(subtree_id());
                node
            })],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
    }

    fn subtree_node_id(id: u64) -> crate::node::NodeId {
        // Subtree uses tree_index 1 (main tree uses 0)
        crate::node::NodeId::new(SchemaNodeId(id), super::TreeIndex(1))
    }

    #[test]
    fn subtree_root_parent_is_graft_when_graft_exists_first() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node first
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Now push the subtree
        let subtree_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Verify subtree root's parent is the graft node
        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent_id(), Some(node_id(1)));
    }

    #[test]
    #[should_panic(expected = "no graft node exists for this tree")]
    fn subtree_push_without_graft_panics() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree without graft node
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Try to push subtree without graft - should panic
        let subtree_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);
    }

    #[test]
    fn subtree_nodes_removed_when_graft_removed() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Push the subtree with multiple nodes
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Verify subtree nodes exist
        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        // Remove the graft node (focus stays on main tree root)
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), {
                let mut node = Node::new(Role::Window);
                node.set_children(vec![]); // Remove node 1
                node
            })],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        // All subtree nodes should be removed
        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());
        // subtrees should be cleaned up
        assert!(tree.state().subtrees.get(&subtree_id()).is_none());
    }

    #[test]
    fn subtree_nodes_removed_when_graft_tree_id_cleared() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Push the subtree with multiple nodes
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Verify subtree nodes exist
        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        // Clear tree_id from the graft node (focus stays on main tree root)
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(1), Node::new(Role::GenericContainer))],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        // All subtree nodes should be removed
        assert!(tree.state().node_by_id(subtree_node_id(0)).is_none());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_none());
        // subtrees should be cleaned up
        assert!(tree.state().subtrees.get(&subtree_id()).is_none());
    }

    #[test]
    #[should_panic(expected = "focus is on a node in this subtree")]
    fn removing_subtree_while_focus_on_subtree_panics() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        // Push the subtree and set subtree focus
        let subtree_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Move main tree focus to graft, which means effective focus is on subtree
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Focus on graft -> focus is on subtree
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        // Clear graft's tree_id while keeping focus on it - should panic
        // because focus will be on a subtree node that gets removed
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(1), Node::new(Role::GenericContainer))], // Clear tree_id
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Keep focus on graft (but subtree will be gone)
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
    }

    #[test]
    fn graft_node_has_no_children_when_subtree_not_pushed() {
        // Create main tree with graft node that has children property set,
        // but don't push subtree - should return empty, NOT the children property
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    // Set children property - this should be ignored, not used as fallback
                    node.set_children(vec![SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let tree = super::Tree::new(update, false);

        // Graft node should have no children since subtree doesn't exist
        // (should NOT fall back to children property)
        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        assert_eq!(graft_node.child_ids().count(), 0);
        assert_eq!(graft_node.children().count(), 0);
    }

    #[test]
    fn graft_node_has_subtree_root_as_child_when_subtree_exists() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node that also has children property set
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    // Set children property - this should be ignored for graft nodes
                    node.set_children(vec![SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Push the subtree
        let subtree_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Graft node should have subtree root as only child, NOT node 2
        let graft_node = tree.state().node_by_id(node_id(1)).unwrap();
        let children: Vec<_> = graft_node.child_ids().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], subtree_node_id(0));
    }

    #[test]
    fn node_added_called_when_subtree_pushed() {
        struct Handler {
            added_nodes: Vec<crate::node::NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, node: &crate::Node) {
                self.added_nodes.push(node.id());
            }
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        let mut handler = Handler {
            added_nodes: Vec::new(),
        };

        // Push the subtree with multiple nodes
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Paragraph)),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

        // Verify node_added was called for all subtree nodes
        assert_eq!(
            handler.added_nodes.len(),
            3,
            "node_added should be called for all 3 subtree nodes"
        );
        assert!(
            handler.added_nodes.contains(&subtree_node_id(0)),
            "subtree root should be added"
        );
        assert!(
            handler.added_nodes.contains(&subtree_node_id(1)),
            "subtree child 1 should be added"
        );
        assert!(
            handler.added_nodes.contains(&subtree_node_id(2)),
            "subtree child 2 should be added"
        );
    }

    #[test]
    fn node_removed_called_when_graft_removed() {
        struct Handler {
            removed_nodes: Vec<crate::node::NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, node: &crate::Node) {
                self.removed_nodes.push(node.id());
            }
        }

        // Create main tree with graft node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Push the subtree with multiple nodes (using NoOpHandler for setup)
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Paragraph)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Verify subtree nodes exist
        assert!(tree.state().node_by_id(subtree_node_id(0)).is_some());
        assert!(tree.state().node_by_id(subtree_node_id(1)).is_some());

        let mut handler = Handler {
            removed_nodes: Vec::new(),
        };

        // Remove the graft node
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), {
                let mut node = Node::new(Role::Window);
                node.set_children(vec![]); // Remove node 1
                node
            })],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(update, &mut handler);

        // Verify node_removed was called for graft and all subtree nodes
        assert!(
            handler.removed_nodes.contains(&node_id(1)),
            "graft node should be removed"
        );
        assert!(
            handler.removed_nodes.contains(&subtree_node_id(0)),
            "subtree root should be removed"
        );
        assert!(
            handler.removed_nodes.contains(&subtree_node_id(1)),
            "subtree child should be removed"
        );
        assert_eq!(
            handler.removed_nodes.len(),
            3,
            "node_removed should be called for graft + 2 subtree nodes"
        );
    }

    #[test]
    fn node_updated_called_when_subtree_reparented() {
        struct Handler {
            updated_nodes: Vec<crate::node::NodeId>,
        }
        impl super::ChangeHandler for Handler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _old: &crate::Node, new: &crate::Node) {
                self.updated_nodes.push(new.id());
            }
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with two potential graft nodes
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id()); // Initial graft
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::GenericContainer)), // Will become new graft
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, false);

        // Push the subtree
        let subtree_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Verify initial parent is node 1 (first graft)
        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent().unwrap().id(), node_id(1));

        let mut handler = Handler {
            updated_nodes: Vec::new(),
        };

        // Reparent: remove tree_id from node 1, add to node 2
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(1), Node::new(Role::GenericContainer)), // Clear tree_id
                (SchemaNodeId(2), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id()); // New graft
                    node
                }),
            ],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(update, &mut handler);

        // Verify subtree root was updated (parent changed)
        assert!(
            handler.updated_nodes.contains(&subtree_node_id(0)),
            "subtree root should receive node_updated when reparented"
        );

        // Verify new parent is node 2
        let subtree_root = tree.state().node_by_id(subtree_node_id(0)).unwrap();
        assert_eq!(subtree_root.parent().unwrap().id(), node_id(2));
    }

    #[test]
    fn focus_moved_called_when_focus_moves_to_subtree() {
        struct Handler {
            focus_moves: Vec<(Option<crate::node::NodeId>, Option<crate::node::NodeId>)>,
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

        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0), // Initial focus on main tree root
        };
        let mut tree = super::Tree::new(update, true); // is_host_focused = true

        // Push the subtree with focusable nodes
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let mut handler = Handler {
            focus_moves: Vec::new(),
        };

        // Move focus to a subtree node via main tree focus moving to graft
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Focus on graft -> effective focus goes to subtree
        };
        tree.update_and_process_changes(update, &mut handler);

        // Verify focus_moved was called
        assert_eq!(
            handler.focus_moves.len(),
            1,
            "focus_moved should be called once"
        );
        let (old_focus, new_focus) = &handler.focus_moves[0];
        assert_eq!(
            *old_focus,
            Some(node_id(0)),
            "old focus should be main tree root"
        );
        // When focus moves to graft, effective focus is subtree's focus
        assert_eq!(
            *new_focus,
            Some(subtree_node_id(0)),
            "new focus should be subtree focus node"
        );
    }

    #[test]
    fn focus_moved_called_when_subtree_focus_changes() {
        struct Handler {
            focus_moves: Vec<(Option<crate::node::NodeId>, Option<crate::node::NodeId>)>,
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

        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree with graft node, focus on graft
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Focus on graft from the start
        };
        let mut tree = super::Tree::new(update, true);

        // Push the subtree with focus on root
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0), // Subtree focus on its root
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let mut handler = Handler {
            focus_moves: Vec::new(),
        };

        // Move subtree focus to a different node within the subtree
        let subtree_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: subtree_id(),
            focus: SchemaNodeId(1), // Move subtree focus to button
        };
        tree.update_and_process_changes(subtree_update, &mut handler);

        // Verify focus_moved was called for the subtree focus change
        assert_eq!(
            handler.focus_moves.len(),
            1,
            "focus_moved should be called once"
        );
        let (old_focus, new_focus) = &handler.focus_moves[0];
        assert_eq!(
            *old_focus,
            Some(subtree_node_id(0)),
            "old focus should be subtree root"
        );
        assert_eq!(
            *new_focus,
            Some(subtree_node_id(1)),
            "new focus should be subtree button"
        );
    }

    // Helper for nested subtree tests
    fn nested_subtree_id() -> TreeId {
        TreeId(Uuid::from_u128(2))
    }

    fn nested_subtree_node_id(n: u64) -> crate::node::NodeId {
        crate::node::NodeId::new(SchemaNodeId(n), crate::tree::TreeIndex(2))
    }

    #[test]
    fn nested_subtree_focus_follows_graft_chain() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Create main tree: window -> graft1
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        // Push first subtree: document -> graft2
        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        // Push nested subtree: group -> button
        let nested_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Group);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: nested_subtree_id(),
            focus: SchemaNodeId(1), // Focus on button
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        // Now set focus chain: main->graft1, subtree->graft2
        // This should result in effective focus on the button in nested subtree
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Focus on graft1
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: subtree_id(),
            focus: SchemaNodeId(1), // Focus on graft2
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        // Effective focus should be on the button in nested subtree
        assert_eq!(
            tree.state().focus_id(),
            Some(nested_subtree_node_id(1)),
            "effective focus should follow graft chain to nested subtree button"
        );
    }

    #[test]
    fn nested_subtree_focus_update_changes_effective_focus() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Setup: main -> graft1 -> subtree -> graft2 -> nested subtree with 2 buttons
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Focus on graft1
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(1), // Focus on graft2
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Group);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: nested_subtree_id(),
            focus: SchemaNodeId(1), // Focus on first button
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        // Effective focus should be on button 1
        assert_eq!(tree.state().focus_id(), Some(nested_subtree_node_id(1)));

        // Change focus in nested subtree to button 2
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: nested_subtree_id(),
            focus: SchemaNodeId(2),
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        // Effective focus should now be on button 2
        assert_eq!(
            tree.state().focus_id(),
            Some(nested_subtree_node_id(2)),
            "changing nested subtree focus should update effective focus"
        );
    }

    #[test]
    #[should_panic(expected = "focus is on a node in this subtree")]
    fn removing_nested_subtree_while_intermediate_focus_on_graft_panics() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Setup nested subtrees with focus chain through grafts
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(1), // Focus on graft1
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(1), // Focus on graft2 (intermediate tree focuses on nested graft)
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![(SchemaNodeId(0), Node::new(Role::Button))],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: nested_subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        // Try to remove nested subtree by clearing graft2's tree_id
        // This should panic because subtree's focus is on graft2
        let update = TreeUpdate {
            nodes: vec![(SchemaNodeId(1), Node::new(Role::GenericContainer))], // Clear tree_id
            tree: None,
            tree_id: subtree_id(),
            focus: SchemaNodeId(1), // Keep focus on the (now non-graft) node
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);
    }

    #[test]
    fn nested_subtree_root_lookup_for_focus_only_update() {
        struct NoOpHandler;
        impl super::ChangeHandler for NoOpHandler {
            fn node_added(&mut self, _: &crate::Node) {}
            fn node_updated(&mut self, _: &crate::Node, _: &crate::Node) {}
            fn focus_moved(&mut self, _: Option<&crate::Node>, _: Option<&crate::Node>) {}
            fn node_removed(&mut self, _: &crate::Node) {}
        }

        // Setup nested subtrees
        let update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Window);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(subtree_id());
                    node
                }),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: tree_id(),
            focus: SchemaNodeId(0),
        };
        let mut tree = super::Tree::new(update, true);

        let subtree_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Document);
                    node.set_children(vec![SchemaNodeId(1), SchemaNodeId(2)]);
                    node
                }),
                (SchemaNodeId(1), {
                    let mut node = Node::new(Role::GenericContainer);
                    node.set_tree_id(nested_subtree_id());
                    node
                }),
                (SchemaNodeId(2), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(subtree_update, &mut NoOpHandler);

        let nested_update = TreeUpdate {
            nodes: vec![
                (SchemaNodeId(0), {
                    let mut node = Node::new(Role::Group);
                    node.set_children(vec![SchemaNodeId(1)]);
                    node
                }),
                (SchemaNodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(Tree::new(SchemaNodeId(0))),
            tree_id: nested_subtree_id(),
            focus: SchemaNodeId(0),
        };
        tree.update_and_process_changes(nested_update, &mut NoOpHandler);

        // Focus-only update on middle subtree (no tree: Some(...))
        // This tests the bug fix for subtree root lookup
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            tree_id: subtree_id(),
            focus: SchemaNodeId(2), // Focus on button in middle subtree
        };
        tree.update_and_process_changes(update, &mut NoOpHandler);

        // Verify subtree focus was updated correctly
        assert_eq!(
            tree.state().subtrees.get(&subtree_id()).unwrap().focus,
            subtree_node_id(2),
            "focus-only update should correctly update subtree focus"
        );
    }

    mod tree_index_map {
        use super::*;

        fn make_tree_id(n: u128) -> TreeId {
            TreeId(Uuid::from_u128(n))
        }

        #[test]
        fn root_tree_id_is_always_index_zero() {
            let mut map = super::super::TreeIndexMap::default();
            let index = map.get_index(TreeId::ROOT);
            assert_eq!(index, super::super::TreeIndex(0));
        }

        #[test]
        fn root_tree_id_is_preassigned_without_calling_get_index() {
            let map = super::super::TreeIndexMap::default();
            // get_id should work without ever calling get_index for ROOT
            assert_eq!(map.get_id(super::super::TreeIndex(0)), Some(TreeId::ROOT));
        }

        #[test]
        fn new_tree_ids_start_at_index_one() {
            let mut map = super::super::TreeIndexMap::default();
            let index = map.get_index(make_tree_id(999));
            assert_eq!(index, super::super::TreeIndex(1));
        }

        #[test]
        fn returns_same_index_for_same_tree_id() {
            let mut map = super::super::TreeIndexMap::default();
            let tree_id = make_tree_id(1);
            let index1 = map.get_index(tree_id);
            let index2 = map.get_index(tree_id);
            assert_eq!(index1, index2);
        }

        #[test]
        fn assigns_sequential_indices() {
            let mut map = super::super::TreeIndexMap::default();
            let index1 = map.get_index(make_tree_id(100));
            let index2 = map.get_index(make_tree_id(200));
            let index3 = map.get_index(make_tree_id(300));
            // Index 0 is reserved for ROOT, so new indices start at 1
            assert_eq!(index1, super::super::TreeIndex(1));
            assert_eq!(index2, super::super::TreeIndex(2));
            assert_eq!(index3, super::super::TreeIndex(3));
        }

        #[test]
        fn get_id_returns_correct_tree_id() {
            let mut map = super::super::TreeIndexMap::default();
            let tree_id = make_tree_id(42);
            let index = map.get_index(tree_id);
            assert_eq!(map.get_id(index), Some(tree_id));
        }

        #[test]
        fn get_id_returns_none_for_unknown_index() {
            let map = super::super::TreeIndexMap::default();
            // Index 0 is ROOT, but higher indices are unknown
            assert_eq!(map.get_id(super::super::TreeIndex(1)), None);
            assert_eq!(map.get_id(super::super::TreeIndex(999)), None);
        }

        #[test]
        fn bidirectional_mapping() {
            let mut map = super::super::TreeIndexMap::default();
            let tree_id1 = make_tree_id(1);
            let tree_id2 = make_tree_id(2);

            let index1 = map.get_index(tree_id1);
            let index2 = map.get_index(tree_id2);

            // Forward: TreeId -> TreeIndex
            assert_eq!(map.get_index(tree_id1), index1);
            assert_eq!(map.get_index(tree_id2), index2);

            // Reverse: TreeIndex -> TreeId
            assert_eq!(map.get_id(index1), Some(tree_id1));
            assert_eq!(map.get_id(index2), Some(tree_id2));
        }
    }
}
