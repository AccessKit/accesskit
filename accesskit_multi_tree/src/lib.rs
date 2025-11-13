use std::{collections::HashMap, sync::atomic::AtomicUsize};

use accesskit::{NodeId, TreeUpdate};

const ROOT_TREE_ID: TreeId = TreeId(0);

pub struct Adapter {
    // TODO: servoshell on Android and OpenHarmony do not use winit
    inner: accesskit_winit::Adapter,

    next_tree_id: TreeId,
    subtrees: HashMap<TreeId, SubtreeInfo>,
    id_map: HashMap<TreeId, HashMap<NodeId, NodeId>>,
    next_node_id: NodeId,
}

pub struct SubtreeInfo {
    parent_tree_id: TreeId,
    parent_node_id: NodeId,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeId(u64);

impl Adapter {
    pub fn new(inner: accesskit_winit::Adapter) -> Self {
        let mut result = Self {
            inner,
            next_tree_id: TreeId(1),
            subtrees: HashMap::default(),
            id_map: HashMap::default(),
            next_node_id: NodeId(0),
        };
        assert!(result.id_map.insert(result.root_tree_id(), HashMap::default()).is_none());
        result
    }

    pub fn root_tree_id(&self) -> TreeId {
        ROOT_TREE_ID
    }

    pub fn register_new_subtree(&mut self, parent_tree_id: TreeId, parent_node_id: NodeId) {
        let subtree_id = self.next_tree_id();
        assert!(self.subtrees.insert(subtree_id, SubtreeInfo { parent_tree_id, parent_node_id }).is_none());
        assert!(self.id_map.insert(subtree_id, HashMap::default()).is_none());
    }

    pub fn unregister_subtree(&mut self, subtree_id: TreeId) {
        // Assert not root tree id
        // Remove from subtrees
        // Remove from id map
        // No need to send a TreeUpdate, the provider of the parent subtree will do it
    }

    pub fn update_if_active(&mut self, tree_id: TreeId, updater: impl FnOnce() -> TreeUpdate) {
        // Q: what happens if the graft node (`parent_node_id`) gets removed by the parent tree?
        // If we keep the registration, then we need to detect when the graft node gets readded
        // (if ever), and resend the subtree in full, which requires caching the subtree or telling
        // the provider to resend it in full (via initial tree request?).
        // If we remove the registration, then we need to detect when the graft node got removed,
        // which could happen at any ancestor, which requires caching the parent tree.
        // For now we just leave this undefined: if you remove the graft node, you must unregister.
        // - Maybe we could use an accesskit_consumer::ChangeHandler to detect these cases?
        //     - Easy way: we set up our own accesskit_consumer::Tree and ChangeHandler
        //     - Hard way: we plumb through the lower-level Adapters and expose the one in
        //       atspi common / android / windows? macOS has one too, but maybe used differently?
        let mut subtree_update = updater();
        subtree_update.focus = self.map_id(tree_id, subtree_update.focus);
        if let Some(tree) = subtree_update.tree.as_mut() {
            tree.root = self.map_id(tree_id, tree.root);
        }
        for (node_id, node) in subtree_update.nodes.iter_mut() {
            *node_id = self.map_id(tree_id, *node_id);
            // Map ids of all node references.
            // These correspond to the `node_id_vec_property_methods` and `node_id_property_methods`
            // lists in `accesskit/src/lib.rs`.
            // TODO: could we make the vec rewrites avoid allocation?
            self.map_node_id_vec_property(tree_id, node.children().to_owned(), |new| node.set_children(new));
            self.map_node_id_vec_property(tree_id, node.controls().to_owned(), |new| node.set_controls(new));
            self.map_node_id_vec_property(tree_id, node.details().to_owned(), |new| node.set_details(new));
            self.map_node_id_vec_property(tree_id, node.described_by().to_owned(), |new| node.set_described_by(new));
            self.map_node_id_vec_property(tree_id, node.flow_to().to_owned(), |new| node.set_flow_to(new));
            self.map_node_id_vec_property(tree_id, node.labelled_by().to_owned(), |new| node.set_labelled_by(new));
            self.map_node_id_vec_property(tree_id, node.owns().to_owned(), |new| node.set_owns(new));
            self.map_node_id_vec_property(tree_id, node.radio_group().to_owned(), |new| node.set_radio_group(new));
            node.active_descendant().map(|node_id| node.set_active_descendant(self.map_id(tree_id, node_id)));
            node.error_message().map(|node_id| node.set_error_message(self.map_id(tree_id, node_id)));
            node.in_page_link_target().map(|node_id| node.set_in_page_link_target(self.map_id(tree_id, node_id)));
            node.member_of().map(|node_id| node.set_member_of(self.map_id(tree_id, node_id)));
            node.next_on_line().map(|node_id| node.set_next_on_line(self.map_id(tree_id, node_id)));
            node.previous_on_line().map(|node_id| node.set_previous_on_line(self.map_id(tree_id, node_id)));
            node.popup_for().map(|node_id| node.set_popup_for(self.map_id(tree_id, node_id)));
            // TODO: what do we do about .level()?
        }
        self.inner.update_if_active(|| subtree_update);
    }

    fn map_id(&mut self, tree_id: TreeId, node_id: NodeId) -> NodeId {
        let map = self.id_map.get_mut(&tree_id).expect("Tree not registered");
        if let Some(result) = map.get(&node_id) {
            return *result;
        }
        let result = self.next_node_id();
        let map = self.id_map.get_mut(&tree_id).expect("Tree not registered");
        assert!(map.insert(node_id, result).is_none());
        result
    }

    fn map_node_id_vec_property(&mut self, tree_id: TreeId, node_ids: Vec<NodeId>, setter: impl FnOnce(Vec<NodeId>)) {
        let mut children = node_ids;
        for node_id in children.iter_mut() {
            *node_id = self.map_id(tree_id, *node_id);
        }
        setter(children);
    }

    fn next_tree_id(&mut self) -> TreeId {
        let subtree_id = self.next_tree_id;
        // TODO handle wrapping? Seems unnecessary for sequential usize = u64
        self.next_tree_id = TreeId(subtree_id.0.checked_add(1).expect("TreeId overflow"));
        subtree_id
    }

    fn next_node_id(&mut self) -> NodeId {
        let node_id = self.next_node_id;
        // TODO handle wrapping? Seems unnecessary for sequential usize = u64
        self.next_node_id = NodeId(node_id.0.checked_add(1).expect("NodeId overflow"));
        node_id
    }
}
