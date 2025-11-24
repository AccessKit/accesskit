use accesskit::{ActionData, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, NodeId, TreeUpdate};
use accesskit_winit::{Event, WindowEvent};
use std::collections::HashMap;
use std::ptr::NonNull;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

// #[cfg(not(test))]
type InnerAdapter = accesskit_winit::Adapter;
// #[cfg(test)]
// type InnerAdapter = self::test::InnerAdapter;

const ROOT_SUBTREE_ID: SubtreeId = SubtreeId(0);

//winit stuff that should just be exposed by accesskit imo
struct WinitActivationHandler<T: From<Event> + Send + 'static> {
    window_id: WindowId,
    proxy: EventLoopProxy<T>,
}

impl<T: From<Event> + Send + 'static> ActivationHandler for WinitActivationHandler<T> {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let event = Event {
            window_id: self.window_id,
            window_event: WindowEvent::InitialTreeRequested,
        };
        self.proxy.send_event(event.into()).ok();
        None
    }
}

struct WinitActionHandler<T: From<Event> + Send + 'static> {
    window_id: WindowId,
    proxy: EventLoopProxy<T>,
}

impl<T: From<Event> + Send + 'static> ActionHandler for WinitActionHandler<T> {
    fn do_action(&mut self, request: ActionRequest) {
        let event = Event {
            window_id: self.window_id,
            window_event: WindowEvent::ActionRequested(request),
        };
        self.proxy.send_event(event.into()).ok();
    }
}

struct WinitDeactivationHandler<T: From<Event> + Send + 'static> {
    window_id: WindowId,
    proxy: EventLoopProxy<T>,
}

/// Adapter that combines a root subtree with any number of other subtrees.
///
/// The root subtree is always defined, and has id [`Self::root_subtree_id`]. To define additional
/// subtrees, call [`Self::register_new_subtree`].
pub struct Adapter {
    // TODO: servoshell on Android and OpenHarmony do not use winit.
    // Allow switching to non-winit inner Adapter, maybe using conditional compilation?
    inner: Option<InnerAdapter>,

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

impl<T: From<Event> + Send + 'static> DeactivationHandler for WinitDeactivationHandler<T> {
    fn deactivate_accessibility(&mut self) {
        let event = Event {
            window_id: self.window_id,
            window_event: WindowEvent::AccessibilityDeactivated,
        };
        self.proxy.send_event(event.into()).ok();
    }
}

impl Adapter {
    // TODO: ensure that the ActivationHandler in `inner` is well-behaved (pushes updates to this adapter).
    // We suspect we can only guarantee this if *we* set the handler, which means *we* create the InnerAdapter.
    // Same goes for ActionHandler, we can intercept the caller’s handler and rewrite the target id.
    // TODO: if using a winit inner Adapter, expose the two winit-specific constructors.
    pub fn with_event_loop_proxy<T: From<Event> + Send + 'static>(
        event_loop: &ActiveEventLoop,
        window: &Window,
        proxy: EventLoopProxy<T>,
    ) -> Self {
        let window_id = window.id();
        let activation_handler = WinitActivationHandler {
            window_id,
            proxy: proxy.clone(),
        };
        let action_handler = WinitActionHandler {
            window_id,
            proxy: proxy.clone(),
        };
        let deactivation_handler = WinitDeactivationHandler { window_id, proxy };
        Self::with_direct_handlers(
            event_loop,
            window,
            activation_handler,
            action_handler,
            deactivation_handler,
        )
    }

    pub fn with_direct_handlers(
        event_loop: &ActiveEventLoop,
        window: &Window,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        deactivation_handler: impl 'static + DeactivationHandler + Send,
    ) -> Self {
        let mut result = Adapter {
            inner: None,
            next_subtree_id: SubtreeId(1),
            child_subtrees: HashMap::new(),
            id_map: HashMap::new(),
            reverse_id_map: HashMap::new(),
            next_node_id: NodeId(0),
        };

        let adapter_ptr = NonNull::from(&mut result);

        struct ActivationHandlerWrapper<H> {
            inner: H,
            adapter: NonNull<Adapter>
        }
        unsafe impl<H: ActivationHandler> Send for ActivationHandlerWrapper<H> {}
        impl<H: ActivationHandler> ActivationHandler for ActivationHandlerWrapper<H> {
            fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
                // TODO for now we just require users of this adapter to send updates via update_if_active.
                None
            }
        }

        struct ActionHandlerWrapper<H> {
            inner: H,
            adapter: NonNull<Adapter>
        }
        unsafe impl<H: ActionHandler> Send for ActionHandlerWrapper<H> {}
        impl<H: ActionHandler> ActionHandler for ActionHandlerWrapper<H> {
            fn do_action(&mut self, mut request: ActionRequest) {
                let adapter = unsafe { self.adapter.as_mut() };
                // Map from the global node id to the local node id and forward to the provided handlers
                request.target = adapter.reverse_map_id(request.target).1;
                if let Some(data) = request.data.as_mut() {
                    match data {
                        ActionData::SetTextSelection(selection) => {
                            let new_anchor = adapter.reverse_map_id(selection.anchor.node).1;
                            selection.anchor.node = new_anchor;
                            let new_focus = adapter.reverse_map_id(selection.focus.node).1;
                            selection.focus.node = new_focus;
                        }
                        _ => {}
                    }
                }
                self.inner.do_action(request)
            }
        }

        let activation_wrapper = ActivationHandlerWrapper {
            inner: activation_handler,
            adapter: adapter_ptr
        };

        let action_wrapper = ActionHandlerWrapper {
            inner: action_handler,
            adapter: adapter_ptr,
        };

        let inner = InnerAdapter::with_direct_handlers(
            event_loop,
            window,
            activation_wrapper,
            action_wrapper,
            deactivation_handler,
        );

        result.inner = Some(inner);

        assert!(result.id_map.insert(result.root_subtree_id(), HashMap::default()).is_none());
        result
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

    fn inner(&mut self) -> &mut InnerAdapter {
        self.inner.as_mut().expect("Adapter.inner used before initialisation")
    }

    pub fn update_if_active(&mut self, subtree_id: SubtreeId, updater: impl FnOnce() -> TreeUpdate) {
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
        self.inner().update_if_active(|| subtree_update);
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

    // #[cfg(test)]
    // fn take_tree_updates(&mut self) -> Vec<TreeUpdate> {
    //     self.inner.take_tree_updates()
    // }
}
//
// #[cfg(test)]
// mod test {
//     use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
//
//     use crate::Adapter;
//
//     #[derive(Default)]
//     pub struct InnerAdapter {
//         tree_updates: Vec<TreeUpdate>,
//     }
//
//     impl InnerAdapter {
//         pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
//             self.tree_updates.push(updater());
//         }
//         pub fn take_tree_updates(&mut self) -> Vec<TreeUpdate> {
//             std::mem::take(&mut self.tree_updates)
//         }
//     }
//
//     fn node(children: impl Into<Vec<NodeId>>) -> Node {
//         let mut result = Node::new(Role::Unknown);
//         result.set_children(children.into());
//         result
//     }
//
//     #[test]
//     fn test_update() {
//         let mut adapter = Adapter::new(InnerAdapter::default());
//         adapter.update_if_active(adapter.root_subtree_id(), || TreeUpdate {
//             nodes: vec![
//                 (NodeId(13), node([NodeId(15), NodeId(14)])),
//                 (NodeId(15), node([])),
//                 (NodeId(14), node([])),
//             ],
//             tree: Some(Tree {
//                 root: NodeId(13),
//                 toolkit_name: None,
//                 toolkit_version: None,
//             }),
//             focus: NodeId(13),
//         });
//         let child_subtree_id = adapter.register_child_subtree(adapter.root_subtree_id(), NodeId(15));
//         adapter.update_if_active(child_subtree_id, || TreeUpdate {
//             nodes: vec![
//                 (NodeId(25), node([NodeId(27), NodeId(26)])),
//                 (NodeId(27), node([])),
//                 (NodeId(26), node([])),
//             ],
//             tree: Some(Tree {
//                 root: NodeId(25),
//                 toolkit_name: None,
//                 toolkit_version: None,
//             }),
//             focus: NodeId(25),
//         });
//         let actual_updates = adapter.take_tree_updates();
//         assert_eq!(actual_updates, vec![
//             TreeUpdate {
//                 nodes: vec![
//                     (NodeId(0), node([NodeId(1), NodeId(2)])),
//                     (NodeId(1), node([])),
//                     (NodeId(2), node([])),
//                 ],
//                 tree: Some(Tree {
//                     root: NodeId(0),
//                     toolkit_name: None,
//                     toolkit_version: None,
//                 }),
//                 focus: NodeId(0),
//             },
//             TreeUpdate {
//                 nodes: vec![
//                     (NodeId(3), node([NodeId(4), NodeId(5)])),
//                     (NodeId(4), node([])),
//                     (NodeId(5), node([])),
//                 ],
//                 tree: Some(Tree {
//                     // FIXME: this is incorrect
//                     root: NodeId(3),
//                     toolkit_name: None,
//                     toolkit_version: None,
//                 }),
//                 // FIXME: this is incorrect
//                 focus: NodeId(3),
//             },
//         ]);
//     }
// }
