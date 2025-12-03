use accesskit::{ActionData, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, TreeUpdate};
use std::collections::HashMap;
use std::ptr::NonNull;

const ROOT_SUBTREE_ID: SubtreeId = SubtreeId(0);

#[derive(Debug, PartialEq)]
pub struct MultiTreeAdapterState {
    next_subtree_id: SubtreeId,
    /// child subtree [`SubtreeId`] → child subtree [`SubtreeInfo`]
    child_subtrees: HashMap<SubtreeId, SubtreeInfo>,
    /// (parent subtree [`SubtreeId`], parent-subtree-local [`NodeId`]) → child subtree [`SubtreeId`]
    grafts: HashMap<(SubtreeId, NodeId), SubtreeId>,
    /// ([`SubtreeId`], local [`NodeId`]) → global [`NodeId`]
    id_map: HashMap<(SubtreeId, NodeId), NodeId>,
    /// global [`NodeId`] → ([`SubtreeId`], local [`NodeId`])
    reverse_id_map: HashMap<NodeId, (SubtreeId, NodeId)>,
    next_node_id: NodeId,
}

#[derive(Debug, PartialEq)]
pub struct SubtreeInfo {
    /// global [`NodeId`] of root node in child subtree
    root_node_id: NodeId,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubtreeId(u64);

impl MultiTreeAdapterState {
    pub fn new() -> Self {
        Self {
            next_subtree_id: SubtreeId(1),
            child_subtrees: HashMap::new(),
            grafts: HashMap::new(),
            id_map: HashMap::new(),
            reverse_id_map: HashMap::new(),
            next_node_id: NodeId(0),
        }
    }

    pub fn wrap_activation_handler(
        &self,
        activation_handler: impl 'static + ActivationHandler + Send
    ) -> impl 'static + ActivationHandler + Send {
        struct ActivationHandlerWrapper<H> {
            inner: H,
            // is this actually safe?
            adapter_state: NonNull<MultiTreeAdapterState>
        }
        unsafe impl<H: ActivationHandler> Send for ActivationHandlerWrapper<H> {}
        impl<H: ActivationHandler> ActivationHandler for ActivationHandlerWrapper<H> {
            fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
                let tree_update = self.inner.request_initial_tree();
                if let Some(tree_update) = tree_update {
                    let adapter_state = unsafe { self.adapter_state.as_mut() };
                    // TODO for now only the root subtree is allowed to use request_initial_tree
                    return Some(adapter_state.rewrite_tree_update(adapter_state.root_subtree_id(), tree_update))
                }
                None
            }
        }
        let adapter_state_ptr = NonNull::from(self);

        ActivationHandlerWrapper { inner: activation_handler, adapter_state: adapter_state_ptr }
    }

    pub fn wrap_action_handler(
        &mut self,
        action_handler: impl 'static + ActionHandler + Send
    ) -> impl 'static + ActionHandler + Send {
        struct ActionHandlerWrapper<H> {
            inner: H,
            // is this actually safe?
            adapter_state: NonNull<MultiTreeAdapterState>
        }
        unsafe impl<H: ActionHandler> Send for ActionHandlerWrapper<H> {}
        impl<H: ActionHandler> ActionHandler for ActionHandlerWrapper<H> {
            fn do_action(&mut self, mut request: ActionRequest) {
                let adapter_state = unsafe { self.adapter_state.as_ref() };
                // Map from the global node id to the local node id and forward to the provided handlers
                request.target = adapter_state.reverse_map_id(request.target).1;
                if let Some(data) = request.data.as_mut() {
                    match data {
                        ActionData::SetTextSelection(selection) => {
                            let new_anchor = adapter_state.reverse_map_id(selection.anchor.node).1;
                            selection.anchor.node = new_anchor;
                            let new_focus = adapter_state.reverse_map_id(selection.focus.node).1;
                            selection.focus.node = new_focus;
                        }
                        _ => {}
                    }
                }
                self.inner.do_action(request)
            }
        }
        let adapter_state_ptr = NonNull::from(self);

        ActionHandlerWrapper { inner: action_handler, adapter_state: adapter_state_ptr }
    }

    pub fn root_subtree_id(&self) -> SubtreeId {
        ROOT_SUBTREE_ID
    }

    pub fn register_child_subtree(&mut self, parent_subtree_id: SubtreeId, parent_node_id: NodeId, child_id: NodeId, mut parent_node: Node) -> (SubtreeId, TreeUpdate) {
        let subtree_id = self.next_subtree_id();
        assert!(self.subtree_is_registered(parent_subtree_id));
        // Maybe store the global id for parent_node?
        assert!(self.grafts.insert((parent_subtree_id, parent_node_id), subtree_id).is_none());
        let global_id_for_child = self.map_id(subtree_id, child_id);
        assert!(self.child_subtrees.insert(subtree_id, SubtreeInfo { root_node_id: global_id_for_child }).is_none());
        let parent_node_global_id = self.rewrite_node(parent_subtree_id, parent_node_id, &mut parent_node);

        let mut nodes: Vec<(NodeId, Node)> = Vec::new();
        nodes.insert(0, (parent_node_global_id, parent_node.clone()));
        nodes.insert(1, (global_id_for_child, Node::default()));
        let tree_update = TreeUpdate {
            nodes,
            tree: None,
            // Absolutely not correct whatsoever
            focus: global_id_for_child
        };
        (subtree_id, tree_update)
    }

    #[expect(unused)]
    pub fn unregister_subtree(&mut self, subtree_id: SubtreeId) {
        // Assert not root subtree id
        // Remove from subtrees
        // Remove from id map
        // No need to send a TreeUpdate, the provider of the parent subtree will do it
    }

    pub fn rewrite_tree_update(&mut self, subtree_id: SubtreeId, mut subtree_update: TreeUpdate) -> TreeUpdate {
        if let Some(tree) = subtree_update.tree.as_mut() {
            let global_id_of_root_of_subtree = self.map_id(subtree_id, tree.root);
            tree.root = global_id_of_root_of_subtree;
            if subtree_id != self.root_subtree_id() {
                let child_subtree_info = self.child_subtrees.get_mut(&subtree_id).expect("Must be registered");
                child_subtree_info.root_node_id = global_id_of_root_of_subtree;
                subtree_update.tree = None;
            }
        }

        // Q: what happens if the graft node (`parent_node_id`) gets removed by the parent subtree?
        // If we keep the registration, then we need to detect when the graft node gets readded
        // (if ever), and resend the subtree in full, which requires caching the subtree or telling
        // the provider to resend it in full (via initial tree request?).
        // If we remove the registration, then we need to detect when the graft node got removed,
        // which could happen at any ancestor, which requires caching the parent subtree.
        // For now we just leave this undefined: if you remove the graft node, you must unregister.
        // - Maybe we could use an accesskit_consumer::ChangeHandler to detect these cases?
        //     - Easy way: we set up our own accesskit_consumer::Tree and ChangeHandler
        //     - Hard way: we plumb through the lower-level Adapters and expose the one in
        //       atspi common / android / windows? macOS has one too, but maybe used differently?
        // TODO: rewrite the focus correctly.
        // We think the model is something like: every subtree has its local idea of the focused
        // node, but that node may not end up being the focused node globally. The globally focused
        // node is the focused node of the root subtree, unless that node is a graft node, in which
        // case it’s the focused node of the child subtree being grafted there (recursively).
        subtree_update.focus = self.map_id(subtree_id, subtree_update.focus);
        // TODO: rewrite the root correctly
        for (node_id, node) in subtree_update.nodes.iter_mut() {
            *node_id = self.rewrite_node(subtree_id, *node_id, node);
        }

        // TODO We need to ensure that we put the subtree root node id as a child of the parent node id.

        subtree_update
    }

    pub fn rewrite_node(&mut self, subtree_id: SubtreeId, node_id: NodeId, node: &mut Node) -> NodeId {
        let grafted_node_id: Option<NodeId> = self.get_root_id_for_grafted_subtree(subtree_id, node_id);
        let global_node_id = self.map_id(subtree_id, node_id);
        // Map ids of all node references.
        // These correspond to the `node_id_vec_property_methods` and `node_id_property_methods`
        // lists in `accesskit/src/lib.rs`.
        // TODO: could we make the vec rewrites avoid allocation?
        self.map_node_id_vec_property(subtree_id, node.children().to_owned(), |new| node.set_children(new));
        self.map_node_id_vec_property(subtree_id, node.controls().to_owned(), |new| node.set_controls(new));
        self.map_node_id_vec_property(subtree_id, node.details().to_owned(), |new| node.set_details(new));
        self.map_node_id_vec_property(subtree_id, node.described_by().to_owned(), |new| node.set_described_by(new));
        self.map_node_id_vec_property(subtree_id, node.flow_to().to_owned(), |new| node.set_flow_to(new));
        self.map_node_id_vec_property(subtree_id, node.labelled_by().to_owned(), |new| node.set_labelled_by(new));
        self.map_node_id_vec_property(subtree_id, node.owns().to_owned(), |new| node.set_owns(new));
        self.map_node_id_vec_property(subtree_id, node.radio_group().to_owned(), |new| node.set_radio_group(new));
        node.active_descendant().map(|node_id| node.set_active_descendant(self.map_id(subtree_id, node_id)));
        node.error_message().map(|node_id| node.set_error_message(self.map_id(subtree_id, node_id)));
        node.in_page_link_target().map(|node_id| node.set_in_page_link_target(self.map_id(subtree_id, node_id)));
        node.member_of().map(|node_id| node.set_member_of(self.map_id(subtree_id, node_id)));
        node.next_on_line().map(|node_id| node.set_next_on_line(self.map_id(subtree_id, node_id)));
        node.previous_on_line().map(|node_id| node.set_previous_on_line(self.map_id(subtree_id, node_id)));
        node.popup_for().map(|node_id| node.set_popup_for(self.map_id(subtree_id, node_id)));
        // TODO: what do we do about .level()?

        if let Some(grafted_node_id) = grafted_node_id {
            node.push_child(grafted_node_id);
        }
        global_node_id
    }

    fn map_id(&mut self, subtree_id: SubtreeId, node_id: NodeId) -> NodeId {
        if let Some(result) = self.id_map.get(&(subtree_id, node_id)) {
            return *result;
        }
        let result = self.next_node_id();
        assert!(self.id_map.insert((subtree_id, node_id), result).is_none());
        assert!(self.reverse_id_map.insert(result, (subtree_id, node_id)).is_none());
        result
    }

    fn reverse_map_id(&self, global_node_id: NodeId) -> (SubtreeId, NodeId) {
        *self.reverse_id_map.get(&global_node_id).expect("Node not registered")
    }

    fn map_node_id_vec_property(&mut self, subtree_id: SubtreeId, node_ids: Vec<NodeId>, setter: impl FnOnce(Vec<NodeId>)) {
        // If node id vec properties return an empty slice from their getters, don’t bother
        // calling the setters. This may be slightly more efficient, and also works around a
        // potentially busted derived impl PartialEq for Properties where PropertyId::Unset in
        // indices is considered unequal to PropertyValue::NodeIdVec(vec![]). It should be
        // equal, because all properties currently default to empty by definition:
        // <https://github.com/AccessKit/accesskit/blob/accesskit-v0.21.1/common/src/lib.rs#L1035>
        if node_ids.is_empty() {
            return;
        }
        let mut node_ids = node_ids;
        for node_id in node_ids.iter_mut() {
            *node_id = self.map_id(subtree_id, *node_id);
        }
        setter(node_ids);
    }

    fn next_subtree_id(&mut self) -> SubtreeId {
        let subtree_id = self.next_subtree_id;
        // TODO handle wrapping? Seems unnecessary for sequential usize = u64
        self.next_subtree_id = SubtreeId(subtree_id.0.checked_add(1).expect("SubtreeId overflow"));
        subtree_id
    }

    fn next_node_id(&mut self) -> NodeId {
        let node_id = self.next_node_id;
        // TODO handle wrapping? Seems unnecessary for sequential usize = u64
        self.next_node_id = NodeId(node_id.0.checked_add(1).expect("NodeId overflow"));
        node_id
    }

    fn subtree_is_registered(&self, subtree_id: SubtreeId) -> bool {
        subtree_id == self.root_subtree_id() || self.child_subtrees.contains_key(&subtree_id)
    }

    fn get_root_id_for_grafted_subtree(&mut self, subtree_id: SubtreeId, local_node_id: NodeId) -> Option<NodeId> {
        if let Some(local_nodes_subtree_id) = self.grafts.get(&(subtree_id, local_node_id)) {
            let child_subtree_info = self.child_subtrees.get(&local_nodes_subtree_id).expect("must be registered");
            return Some(child_subtree_info.root_node_id);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, hash::Hash};

    use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};

    use crate::{MultiTreeAdapterState, ROOT_SUBTREE_ID, SubtreeId, SubtreeInfo};

    fn node(children: impl Into<Vec<NodeId>>) -> Node {
        let children = children.into();
        let mut result = Node::new(Role::Unknown);
        if !children.is_empty() {
            result.set_children(children);
        }
        result
    }

    fn map<K: Eq + Hash, V>(entries: impl Into<HashMap<K, V>>) -> HashMap<K, V> {
        entries.into()
    }

    fn subtree_info(root_node_id: NodeId) -> SubtreeInfo {
        SubtreeInfo { root_node_id }
    }

    #[test]
    fn test_update() {
        let mut multitree = MultiTreeAdapterState::new();
        let graft_node = node([]);
        assert_eq!(
            multitree.rewrite_tree_update(ROOT_SUBTREE_ID, TreeUpdate {
                nodes: vec![
                    (NodeId(13), node([NodeId(15), NodeId(14)])),
                    (NodeId(15), graft_node.clone()),
                    (NodeId(14), node([])),
                ],
                tree: Some(Tree {
                    root: NodeId(13),
                    toolkit_name: None,
                    toolkit_version: None,
                }),
                focus: NodeId(13),
            }),
            TreeUpdate {
                nodes: vec![
                    (NodeId(0), node([NodeId(1), NodeId(2)])),
                    (NodeId(1), node([])),
                    (NodeId(2), node([])),
                ],
                tree: Some(Tree {
                    root: NodeId(0),
                    toolkit_name: None,
                    toolkit_version: None,
                }),
                focus: NodeId(0),
            },
        );
        let (child_subtree_id, tree_update) = multitree.register_child_subtree(ROOT_SUBTREE_ID, NodeId(15), NodeId(25), graft_node);
        assert_eq!(child_subtree_id, SubtreeId(1));
        assert_eq!(
            tree_update,
            TreeUpdate {
                nodes: vec![
                    (NodeId(1), node([NodeId(3)])),
                    (NodeId(3), node([])),
                ],
                tree: None,
                // FIXME: assertion failed: actual #3, expected #0
                focus: NodeId(3),
            },
        );
        assert_eq!(
            multitree.rewrite_tree_update(child_subtree_id, TreeUpdate {
                nodes: vec![
                    (NodeId(25), node([NodeId(27), NodeId(26)])),
                    (NodeId(27), node([])),
                    (NodeId(26), node([])),
                ],
                tree: Some(Tree {
                    root: NodeId(25),
                    toolkit_name: None,
                    toolkit_version: None,
                }),
                focus: NodeId(25),
            }),
            TreeUpdate {
                nodes: vec![
                    (NodeId(3), node([NodeId(4), NodeId(5)])),
                    (NodeId(4), node([])),
                    (NodeId(5), node([])),
                ],
                tree: None,
                focus: NodeId(3),
            },
        );
        assert_eq!(
            multitree,
            MultiTreeAdapterState {
                next_subtree_id: SubtreeId(2),
                child_subtrees: map([
                    (child_subtree_id, subtree_info(NodeId(3))),
                ]),
                grafts: map([
                    ((ROOT_SUBTREE_ID, NodeId(15)), child_subtree_id),
                ]),
                id_map: map([
                    ((ROOT_SUBTREE_ID, NodeId(13)), NodeId(0)),
                    ((ROOT_SUBTREE_ID, NodeId(15)), NodeId(1)),
                    ((ROOT_SUBTREE_ID, NodeId(14)), NodeId(2)),
                    ((child_subtree_id, NodeId(25)), NodeId(3)),
                    ((child_subtree_id, NodeId(27)), NodeId(4)),
                    ((child_subtree_id, NodeId(26)), NodeId(5)),
                ]),
                reverse_id_map: map([
                    (NodeId(0), (ROOT_SUBTREE_ID, NodeId(13))),
                    (NodeId(1), (ROOT_SUBTREE_ID, NodeId(15))),
                    (NodeId(2), (ROOT_SUBTREE_ID, NodeId(14))),
                    (NodeId(3), (child_subtree_id, NodeId(25))),
                    (NodeId(4), (child_subtree_id, NodeId(27))),
                    (NodeId(5), (child_subtree_id, NodeId(26))),
                ]),
                next_node_id: NodeId(6),
            },
        );
    }
}
