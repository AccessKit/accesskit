// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::{CacheNode, Error, NodeIdOrRoot, PlatformNode, PlatformRoot};
use atspi::{CacheItem, ObjectRef, ObjectRefOwned};
use std::sync::{Arc, OnceLock};
use zbus::{
    fdo, interface,
    names::{OwnedUniqueName, UniqueName},
};

use super::map_root_error;
use crate::atspi::ObjectId;

pub(crate) fn object_ref(bus_name: &UniqueName, id: ObjectId) -> ObjectRefOwned {
    ObjectRef::new_owned(bus_name.to_owned(), id.path())
}

pub(crate) fn cache_item_for_node(
    bus_name: &UniqueName,
    node: &PlatformNode,
) -> Result<CacheItem, Error> {
    Ok(cache_item(bus_name, node.cache_node()?))
}

fn cache_item(bus_name: &UniqueName, node: CacheNode) -> CacheItem {
    let parent = match node.parent {
        NodeIdOrRoot::Root => ObjectId::Root,
        NodeIdOrRoot::Node(node_id) => ObjectId::Node {
            adapter: node.adapter_id,
            node: node_id,
        },
    };
    CacheItem {
        object: object_ref(
            bus_name,
            ObjectId::Node {
                adapter: node.adapter_id,
                node: node.id,
            },
        ),
        app: object_ref(bus_name, ObjectId::Root),
        parent: object_ref(bus_name, parent),
        index: node.index_in_parent,
        children: node.child_count,
        ifaces: node.interfaces,
        short_name: node.name,
        role: node.role,
        name: node.description,
        states: node.states,
    }
}

fn application_cache_item(
    bus_name: &UniqueName,
    root: &PlatformRoot,
    desktop: ObjectRefOwned,
) -> Result<CacheItem, Error> {
    Ok(CacheItem {
        object: object_ref(bus_name, ObjectId::Root),
        app: object_ref(bus_name, ObjectId::Root),
        parent: desktop,
        index: root.index_in_parent(),
        children: root.child_count()?,
        ifaces: root.interfaces(),
        short_name: root.name()?,
        role: root.role(),
        name: root.description()?,
        states: root.state(),
    })
}

pub(crate) struct CacheInterface {
    bus_name: OwnedUniqueName,
    root: PlatformRoot,
    desktop: Arc<OnceLock<ObjectRefOwned>>,
}

impl CacheInterface {
    pub fn new(
        bus_name: OwnedUniqueName,
        root: PlatformRoot,
        desktop: Arc<OnceLock<ObjectRefOwned>>,
    ) -> Self {
        Self {
            bus_name,
            root,
            desktop,
        }
    }

    fn items(&self) -> Result<Vec<CacheItem>, Error> {
        let bus_name = self.bus_name.inner();
        let descendants: Vec<CacheItem> = self
            .root
            .map_descendant_cache_nodes(|node| cache_item(bus_name, node))?;

        let desktop = self.desktop.get().cloned().unwrap_or_default();
        let mut items = Vec::with_capacity(descendants.len() + 1);
        items.push(application_cache_item(bus_name, &self.root, desktop)?);
        items.extend(descendants);
        Ok(items)
    }
}

#[interface(name = "org.a11y.atspi.Cache")]
impl CacheInterface {
    fn get_items(&self) -> fdo::Result<Vec<CacheItem>> {
        self.items().map_err(map_root_error)
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheInterface, object_ref};
    use crate::atspi::ObjectId;
    use accesskit::{
        ActionHandler, ActionRequest, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate,
    };
    use accesskit_atspi_common::{
        Adapter, AdapterCallback, AppContext, Event, FullNodeId, PlatformRoot, WindowBounds,
    };
    use atspi::{Interface, ObjectRefOwned, Role as AtspiRole};
    use std::sync::{Arc, OnceLock};
    use zbus::names::{OwnedUniqueName, UniqueName};

    struct NoOpActionHandler;
    impl ActionHandler for NoOpActionHandler {
        fn do_action(&mut self, _request: ActionRequest) {}
    }

    struct NoOpCallback;
    impl AdapterCallback for NoOpCallback {
        fn register_interfaces(&self, _: &Adapter, _: FullNodeId, _: atspi::InterfaceSet) {}
        fn unregister_interfaces(&self, _: &Adapter, _: FullNodeId, _: atspi::InterfaceSet) {}
        fn emit_event(&self, _: &Adapter, _: Event) {}
    }

    fn with_children(role: Role, children: &[NodeId]) -> Node {
        let mut node = Node::new(role);
        node.set_children(children.to_vec());
        node
    }

    const BUS_NAME: &str = ":1.0";

    fn root_for(update: TreeUpdate) -> (Adapter, PlatformRoot) {
        let app_context = AppContext::new(None);
        let adapter = Adapter::new(
            &app_context,
            NoOpCallback,
            update,
            false,
            WindowBounds::default(),
            NoOpActionHandler,
        );
        let root = adapter.platform_root();
        (adapter, root)
    }

    fn bus_name() -> OwnedUniqueName {
        OwnedUniqueName::try_from(BUS_NAME).unwrap()
    }

    fn cache(update: TreeUpdate) -> (Adapter, CacheInterface) {
        let (adapter, root) = root_for(update);
        let desktop = Arc::new(OnceLock::new());
        desktop.set(desktop_ref()).unwrap();
        (adapter, CacheInterface::new(bus_name(), root, desktop))
    }

    fn window_with_button() -> TreeUpdate {
        TreeUpdate {
            nodes: vec![
                (NodeId(0), with_children(Role::Window, &[NodeId(1)])),
                (NodeId(1), Node::new(Role::Button)),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        }
    }

    fn root_ref() -> ObjectRefOwned {
        object_ref(
            &UniqueName::from_static_str_unchecked(BUS_NAME),
            ObjectId::Root,
        )
    }

    fn desktop_ref() -> ObjectRefOwned {
        ObjectRefOwned::from_static_str_unchecked(":1.1", "/org/a11y/atspi/accessible/root")
    }

    #[test]
    fn get_items_prepends_application_root() {
        let (_adapter, iface) = cache(window_with_button());
        let items = iface.items().unwrap();
        let [app, _window, _button] = items.as_slice() else {
            panic!("expected application root, window, and button");
        };
        assert_eq!(app.object, root_ref());
        assert_eq!(app.app, root_ref());
        assert_eq!(app.parent, desktop_ref());
        assert_eq!(app.index, -1);
        assert_eq!(app.children, 1);
        assert_eq!(app.role, AtspiRole::Application);
        assert!(app.ifaces.contains(Interface::Application));
    }

    #[test]
    fn top_window_parent_is_application_root() {
        let (_adapter, iface) = cache(window_with_button());
        let items = iface.items().unwrap();
        let [_app, window, _button] = items.as_slice() else {
            panic!("expected application root, window, and button");
        };
        assert_eq!(window.parent, root_ref());
        assert_eq!(window.index, 0);
        assert!(
            window
                .object
                .path_as_str()
                .starts_with("/org/a11y/atspi/accessible/")
        );
        assert_eq!(window.object.name_as_str(), Some(BUS_NAME));
    }

    #[test]
    fn filtered_child_excluded_from_items_and_counts() {
        let mut hidden = Node::new(Role::Button);
        hidden.set_hidden();
        let update = TreeUpdate {
            nodes: vec![
                (
                    NodeId(0),
                    with_children(Role::Window, &[NodeId(1), NodeId(2)]),
                ),
                (NodeId(1), Node::new(Role::Button)),
                (NodeId(2), hidden),
            ],
            tree: Some(TreeInfo::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let (_adapter, iface) = cache(update);
        let items = iface.items().unwrap();
        let [app_root, window, _visible_button] = items.as_slice() else {
            panic!("expected application root, window, and visible button only");
        };
        assert_eq!(app_root.children, 1);
        assert_eq!(window.children, 1);
    }

    #[test]
    fn object_ref_encodes_bus_name_and_node_path() {
        let (_adapter, root) = root_for(window_with_button());
        let (adapter_id, node_id) = root.child_id_at_index(0).unwrap().unwrap();
        let expected_path = ObjectId::Node {
            adapter: adapter_id,
            node: node_id,
        }
        .path();
        let reference = object_ref(
            &UniqueName::from_static_str_unchecked(BUS_NAME),
            ObjectId::Node {
                adapter: adapter_id,
                node: node_id,
            },
        );
        assert_eq!(reference.name_as_str(), Some(BUS_NAME));
        assert_eq!(reference.path_as_str(), expected_path.as_str());
    }

    #[test]
    fn multi_adapter_windows_have_distinct_indices() {
        let app_context = AppContext::new(None);
        let _a0 = Adapter::new(
            &app_context,
            NoOpCallback,
            window_with_button(),
            false,
            WindowBounds::default(),
            NoOpActionHandler,
        );
        let _a1 = Adapter::new(
            &app_context,
            NoOpCallback,
            window_with_button(),
            false,
            WindowBounds::default(),
            NoOpActionHandler,
        );
        let iface = CacheInterface::new(
            bus_name(),
            PlatformRoot::new(&app_context),
            Arc::new(OnceLock::new()),
        );
        let items = iface.items().unwrap();

        let app_root = &items[0];
        assert_eq!(app_root.children, 2);

        let windows = items.iter().filter(|item| item.parent == root_ref());
        let mut window_indices: Vec<i32> = windows.map(|item| item.index).collect();
        window_indices.sort();
        assert_eq!(window_indices, vec![0, 1]);
    }
}
