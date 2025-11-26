use accesskit::{ActionData, ActionHandler, ActionRequest, ActivationHandler, NodeId, TreeUpdate};
use std::collections::HashMap;
use std::ptr::NonNull;

const ROOT_SUBTREE_ID: SubtreeId = SubtreeId(0);

pub struct MultiTreeAdapterState {
    next_subtree_id: SubtreeId,
    child_subtrees: HashMap<SubtreeId, SubtreeInfo>,
    id_map: HashMap<SubtreeId, HashMap<NodeId, NodeId>>,
    reverse_id_map: HashMap<NodeId, (SubtreeId, NodeId)>,
    next_node_id: NodeId,
}

pub struct SubtreeInfo {
    parent_subtree_id: SubtreeId,
    parent_node_id: NodeId,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubtreeId(u64);

impl MultiTreeAdapterState {
    pub fn new() -> Self {
        let mut result = MultiTreeAdapterState {
            next_subtree_id: SubtreeId(1),
            child_subtrees: HashMap::new(),
            id_map: HashMap::new(),
            reverse_id_map: HashMap::new(),
            next_node_id: NodeId(0),
        };

        assert!(result.id_map.insert(result.root_subtree_id(), HashMap::default()).is_none());
        result
    }

    pub fn wrap_activation_handler(
        &self,
        activation_handler: impl 'static + ActivationHandler + Send
    ) -> impl 'static + ActivationHandler + Send {
        struct ActivationHandlerWrapper<H> {
            inner: H,
        }
        unsafe impl<H: ActivationHandler> Send for ActivationHandlerWrapper<H> {}
        impl<H: ActivationHandler> ActivationHandler for ActivationHandlerWrapper<H> {
            fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
                // TODO for now we just require users of this adapter to send updates via update_if_active.
                None
            }
        }

        ActivationHandlerWrapper { inner: activation_handler }
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

    pub fn register_child_subtree(&mut self, parent_subtree_id: SubtreeId, parent_node_id: NodeId) -> SubtreeId {
        let subtree_id = self.next_subtree_id();
        assert!(self.subtree_is_registered(parent_subtree_id));
        assert!(self.child_subtrees.insert(subtree_id, SubtreeInfo { parent_subtree_id, parent_node_id }).is_none());
        assert!(self.id_map.insert(subtree_id, HashMap::default()).is_none());
        subtree_id
    }

    #[expect(unused)]
    pub fn unregister_subtree(&mut self, subtree_id: SubtreeId) {
        // Assert not root subtree id
        // Remove from subtrees
        // Remove from id map
        // No need to send a TreeUpdate, the provider of the parent subtree will do it
    }

    pub fn rewrite_tree_update(&mut self, subtree_id: SubtreeId, updater: impl FnOnce() -> TreeUpdate) -> impl FnOnce() -> TreeUpdate {
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
        // TODO possibly we don't want to call updater until our return FnOnce is invoked?
        let mut subtree_update = updater();
        // TODO: rewrite the focus correctly.
        // We think the model is something like: every subtree has its local idea of the focused
        // node, but that node may not end up being the focused node globally. The globally focused
        // node is the focused node of the root subtree, unless that node is a graft node, in which
        // case it’s the focused node of the child subtree being grafted there (recursively).
        subtree_update.focus = self.map_id(subtree_id, subtree_update.focus);
        // TODO: rewrite the root correctly
        if let Some(tree) = subtree_update.tree.as_mut() {
            tree.root = self.map_id(subtree_id, tree.root);
        }
        for (node_id, node) in subtree_update.nodes.iter_mut() {
            *node_id = self.map_id(subtree_id, *node_id);
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
        }
        || subtree_update
    }

    fn map_id(&mut self, subtree_id: SubtreeId, node_id: NodeId) -> NodeId {
        let map = self.id_map.get_mut(&subtree_id).expect("Subtree not registered");
        if let Some(result) = map.get(&node_id) {
            return *result;
        }
        let result = self.next_node_id();
        let map = self.id_map.get_mut(&subtree_id).expect("Subtree not registered");
        assert!(map.insert(node_id, result).is_none());
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
}
