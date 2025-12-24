use accesskit::{ActionData, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, TreeUpdate};
use std::ptr::NonNull;

const ROOT_SUBTREE_ID: SubtreeId = SubtreeId(0);

#[cfg(test)]
type Map<K, V> = std::collections::BTreeMap<K, V>;
#[cfg(test)]
type Set<V> = std::collections::BTreeSet<V>;
#[cfg(not(test))]
type Map<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(test))]
type Set<V> = std::collections::HashSet<V>;

#[derive(Debug, PartialEq)]
pub struct MultiTreeAdapterState {
    next_subtree_id: SubtreeId,
    /// [`SubtreeId`] → [`SubtreeInfo`] (or None if the root is not yet known)
    subtrees: Map<SubtreeId, Option<SubtreeInfo>>,
    /// (parent subtree [`SubtreeId`], parent-subtree-local [`NodeId`]) → child subtree [`SubtreeId`]
    grafts: Map<(SubtreeId, NodeId), SubtreeId>,
    /// ([`SubtreeId`], local [`NodeId`]) → global [`NodeId`]
    id_map: Map<(SubtreeId, NodeId), NodeId>,
    /// global [`NodeId`] → [`NodeInfo`]
    node_info: Map<NodeId, NodeInfo>,
    next_node_id: NodeId,
}

#[derive(Debug, PartialEq)]
pub struct SubtreeInfo {
    /// global [`NodeId`] of root node in subtree
    root_node_id: NodeId,
}

#[derive(Debug, PartialEq)]
struct NodeInfo {
    /// reverse mapping: [`SubtreeId`]
    subtree_id: SubtreeId,
    /// reverse mapping: local [`NodeId`]
    local_node_id: NodeId,
    /// global [`NodeId`] of children
    children: Vec<NodeId>,
}
impl NodeInfo {
    fn new(subtree_id: SubtreeId, local_node_id: NodeId) -> Self {
        Self {
            subtree_id,
            local_node_id,
            children: vec![],
        }
    }
    #[cfg(test)]
    fn with_children(subtree_id: SubtreeId, local_node_id: NodeId, children: impl Into<Vec<NodeId>>) -> Self {
        Self {
            subtree_id,
            local_node_id,
            children: children.into(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubtreeId(u64);

impl MultiTreeAdapterState {
    pub fn new() -> Self {
        Self {
            next_subtree_id: SubtreeId(1),
            subtrees: [(ROOT_SUBTREE_ID, None)].into(),
            grafts: Map::new(),
            id_map: Map::new(),
            node_info: Map::new(),
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
                request.target = adapter_state.node_info(request.target).local_node_id;
                if let Some(data) = request.data.as_mut() {
                    match data {
                        ActionData::SetTextSelection(selection) => {
                            selection.anchor.node = adapter_state.node_info(selection.anchor.node).local_node_id;
                            selection.focus.node = adapter_state.node_info(selection.focus.node).local_node_id;
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
        assert!(self.subtrees.insert(subtree_id, Some(SubtreeInfo { root_node_id: global_id_for_child })).is_none());
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
        // global [`NodeId`] of nodes that are no longer referenced.
        // initially this is all of the nodes that were previously in the subtree.
        // then we remove nodes, as we prove that they are still referenced.
        let mut garbage = if let Some(old_root_node_global_id) = self
            .subtrees
            .get(&subtree_id)
            .expect("Must be registered")
            .as_ref()
            .map(|info| info.root_node_id)
        {
            Set::from_iter(self.descendants(old_root_node_global_id))
        } else {
            Set::new()
        };

        if let Some(tree) = subtree_update.tree.as_mut() {
            let new_root_node_global_id = self.map_id(subtree_id, tree.root);
            tree.root = new_root_node_global_id;
            let subtree_info = self.subtrees.get_mut(&subtree_id).expect("Must be registered");
            if let Some(subtree_info) = subtree_info {
                let old_root_node_global_id = subtree_info.root_node_id;
                if new_root_node_global_id != old_root_node_global_id {
                    subtree_info.root_node_id = new_root_node_global_id;
                    // FIXME we also need to update the parent of the root node we were grafted into,
                    // but this means we need to retain the parent node in its entirety :(
                }
            } else {
                *subtree_info = Some(SubtreeInfo { root_node_id: new_root_node_global_id });
            }
            if subtree_id != self.root_subtree_id() {
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
        // TODO We need to ensure that we put the subtree root node id as a child of the parent node id.
        for (node_id, node) in subtree_update.nodes.iter_mut() {
            *node_id = self.rewrite_node(subtree_id, *node_id, node);
        }

        // Now compute the final set of garbage [`NodeId`], and destroy those nodes.
        if let Some(root_node_global_id) = self
            .subtrees
            .get(&subtree_id)
            .expect("Must be registered")
            .as_ref()
            .map(|info| info.root_node_id)
        {
            for child_node_global_id in self.descendants(root_node_global_id) {
                garbage.remove(&child_node_global_id);
            }
        }
        for garbage_node_global_id in dbg!(garbage) {
            let node_info = self.node_info.remove(&garbage_node_global_id).expect("Node must have info");
            self.id_map.remove(&(node_info.subtree_id, node_info.local_node_id));
        }

        subtree_update
    }

    fn rewrite_node(&mut self, subtree_id: SubtreeId, node_id: NodeId, node: &mut Node) -> NodeId {
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
        self.node_info_mut(global_node_id).children = node.children().to_owned();
        global_node_id
    }

    fn map_id(&mut self, subtree_id: SubtreeId, node_id: NodeId) -> NodeId {
        if let Some(result) = self.id_map.get(&(subtree_id, node_id)) {
            return *result;
        }
        let result = self.next_node_id();
        assert!(self.id_map.insert((subtree_id, node_id), result).is_none());
        assert!(self.node_info.insert(result, NodeInfo::new(subtree_id, node_id)).is_none());
        result
    }

    fn node_info(&self, global_node_id: NodeId) -> &NodeInfo {
        self.node_info.get(&global_node_id).expect("Node not registered")
    }

    fn node_info_mut(&mut self, global_node_id: NodeId) -> &mut NodeInfo {
        self.node_info.get_mut(&global_node_id).expect("Node not registered")
    }

    fn descendants(&self, global_node_id: NodeId) -> Descendants<'_> {
        Descendants { state: self, stack: vec![global_node_id] }
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
        self.subtrees.contains_key(&subtree_id)
    }

    fn get_root_id_for_grafted_subtree(&mut self, subtree_id: SubtreeId, local_node_id: NodeId) -> Option<NodeId> {
        if let Some(local_nodes_subtree_id) = self.grafts.get(&(subtree_id, local_node_id)) {
            let subtree_info = self.subtrees.get(&local_nodes_subtree_id).expect("must be registered");
            return subtree_info.as_ref().map(|info| info.root_node_id);
        }
        None
    }
}

/// Iterator over global [`NodeId`] of descendants.
struct Descendants<'state> {
    state: &'state MultiTreeAdapterState,
    /// next global [`NodeId`] to explore
    stack: Vec<NodeId>,
}
impl Iterator for Descendants<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(result) = self.stack.pop() else { return None };
        self.stack.extend_from_slice(&self.state.node_info(result).children);
        Some(result)
    }
}

#[cfg(test)]
mod test {
    use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};

    use crate::{Map, MultiTreeAdapterState, NodeInfo, ROOT_SUBTREE_ID, SubtreeId, SubtreeInfo};

    fn node(children: impl Into<Vec<NodeId>>) -> Node {
        let children = children.into();
        let mut result = Node::new(Role::Unknown);
        if !children.is_empty() {
            result.set_children(children);
        }
        result
    }

    fn map<K, V>(entries: impl Into<Map<K, V>>) -> Map<K, V> {
        entries.into()
    }

    fn subtree_info(root_node_id: NodeId) -> SubtreeInfo {
        SubtreeInfo { root_node_id }
    }

    #[test]
    fn test_update() {
        let mut multitree = MultiTreeAdapterState::new();
        let graft_node = node([]);
        // Check the initial root subtree update.
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
        // Register the child subtree, and check the implicit update.
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
        // Check the initial child subtree update.
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
        // Check a subsequent child subtree update that entirely replaces the tree.
        assert_eq!(
            multitree.rewrite_tree_update(child_subtree_id, TreeUpdate {
                nodes: vec![
                    (NodeId(35), node([NodeId(37), NodeId(36)])),
                    (NodeId(37), node([])),
                    (NodeId(36), node([])),
                ],
                tree: Some(Tree {
                    root: NodeId(35),
                    toolkit_name: None,
                    toolkit_version: None,
                }),
                focus: NodeId(35),
            }),
            TreeUpdate {
                nodes: vec![
                    (NodeId(6), node([NodeId(7), NodeId(8)])),
                    (NodeId(7), node([])),
                    (NodeId(8), node([])),
                ],
                tree: None,
                focus: NodeId(6),
            },
        );
        // Check the final state of the instance.
        assert_eq!(
            multitree,
            MultiTreeAdapterState {
                next_subtree_id: SubtreeId(2),
                subtrees: map([
                    (ROOT_SUBTREE_ID, Some(subtree_info(NodeId(0)))),
                    (child_subtree_id, Some(subtree_info(NodeId(6)))),
                ]),
                grafts: map([
                    ((ROOT_SUBTREE_ID, NodeId(15)), child_subtree_id),
                ]),
                id_map: map([
                    ((ROOT_SUBTREE_ID, NodeId(13)), NodeId(0)),
                    ((ROOT_SUBTREE_ID, NodeId(15)), NodeId(1)),
                    ((ROOT_SUBTREE_ID, NodeId(14)), NodeId(2)),
                    ((child_subtree_id, NodeId(35)), NodeId(6)),
                    ((child_subtree_id, NodeId(37)), NodeId(7)),
                    ((child_subtree_id, NodeId(36)), NodeId(8)),
                ]),
                node_info: map([
                    (NodeId(0), NodeInfo::with_children(ROOT_SUBTREE_ID, NodeId(13), [NodeId(1), NodeId(2)])),
                    // FIXME this should have child NodeId(6), not NodeId(3),
                    // but we don’t emit an update for the parent of a changed graft node yet
                    (NodeId(1), NodeInfo::with_children(ROOT_SUBTREE_ID, NodeId(15), [NodeId(3)])),
                    (NodeId(2), NodeInfo::new(ROOT_SUBTREE_ID, NodeId(14))),
                    (NodeId(6), NodeInfo::with_children(child_subtree_id, NodeId(35), [NodeId(7), NodeId(8)])),
                    (NodeId(7), NodeInfo::new(child_subtree_id, NodeId(37))),
                    (NodeId(8), NodeInfo::new(child_subtree_id, NodeId(36))),
                ]),
                next_node_id: NodeId(9),
            },
        );
    }
}
