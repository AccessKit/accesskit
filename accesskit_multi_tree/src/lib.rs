use std::{collections::HashMap, sync::atomic::AtomicUsize};

use accesskit::{NodeId, TreeUpdate};

pub static NEXT_TREE_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Adapter {
    // TODO: servoshell on Android and OpenHarmony do not use winit
    inner: accesskit_winit::Adapter,

    next_tree_id: TreeId,
    root_tree_id: TreeId,
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
        Self {
            inner,
            next_tree_id: TreeId(1),
            root_tree_id: TreeId(0),
            subtrees: HashMap::default(),
            id_map: HashMap::default(),
            next_node_id: NodeId(0),
        }
    }

    pub fn register_new_subtree(&mut self, parent_tree_id: TreeId, parent_node_id: NodeId) {
        let subtree_id = self.next_tree_id();
        assert!(self.subtrees.insert(subtree_id, SubtreeInfo { parent_tree_id, parent_node_id }).is_none());
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
        if let Some(tree) = subtree_update.tree.as_mut() {
            tree.root = self.map_id(tree_id, tree.root);
        }
        #[expect(unused_variables)]
        for (node_id, node) in subtree_update.nodes.iter_mut() {
            *node_id = self.map_id(tree_id, *node_id);
            // TODO: map ids of all node references:
            // children, controls, labelled_by, details, described_by, flow_to, ...
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
