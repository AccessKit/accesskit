// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::PlatformNode;
use zbus::{fdo, interface, names::OwnedUniqueName};

use crate::atspi::{ObjectId, OwnedObjectAddress};

pub(crate) struct TableInterface {
    bus_name: OwnedUniqueName,
    node: PlatformNode,
}

impl TableInterface {
    pub fn new(bus_name: OwnedUniqueName, node: PlatformNode) -> Self {
        Self { bus_name, node }
    }

    fn map_error(&self) -> impl '_ + FnOnce(accesskit_atspi_common::Error) -> fdo::Error {
        |error| crate::util::map_error_from_node(&self.node, error)
    }

    fn object_address(&self, id: Option<accesskit_atspi_common::FullNodeId>) -> OwnedObjectAddress {
        let id = id.map(|node| ObjectId::Node {
            adapter: self.node.adapter_id(),
            node,
        });
        super::optional_object_address(&self.bus_name, id).0
    }
}

#[interface(name = "org.a11y.atspi.Table")]
impl TableInterface {
    #[zbus(property)]
    fn n_rows(&self) -> fdo::Result<i32> {
        self.node.table_row_count().map_err(self.map_error())
    }

    #[zbus(property)]
    fn n_columns(&self) -> fdo::Result<i32> {
        self.node.table_column_count().map_err(self.map_error())
    }

    #[zbus(property)]
    fn caption(&self) -> fdo::Result<(OwnedObjectAddress,)> {
        let caption = self.node.table_caption().map_err(self.map_error())?;
        Ok((self.object_address(caption),))
    }

    #[zbus(property)]
    fn summary(&self) -> fdo::Result<String> {
        self.node.table_summary().map_err(self.map_error())
    }

    #[zbus(property)]
    fn n_selected_rows(&self) -> fdo::Result<i32> {
        self.node.table_n_selected_rows().map_err(self.map_error())
    }

    #[zbus(property)]
    fn n_selected_columns(&self) -> fdo::Result<i32> {
        self.node
            .table_n_selected_columns()
            .map_err(self.map_error())
    }

    fn get_accessible_at(&self, row: i32, column: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        let cell = self
            .node
            .table_accessible_at(row, column)
            .map_err(self.map_error())?;
        Ok((self.object_address(cell),))
    }

    fn get_index_at(&self, row: i32, column: i32) -> fdo::Result<i32> {
        self.node
            .table_index_at(row, column)
            .map_err(self.map_error())
    }

    fn get_row_at_index(&self, index: i32) -> fdo::Result<i32> {
        self.node
            .table_row_at_index(index)
            .map_err(self.map_error())
    }

    fn get_column_at_index(&self, index: i32) -> fdo::Result<i32> {
        self.node
            .table_column_at_index(index)
            .map_err(self.map_error())
    }

    fn get_row_description(&self, row: i32) -> fdo::Result<String> {
        self.node
            .table_row_description(row)
            .map_err(self.map_error())
    }

    fn get_column_description(&self, column: i32) -> fdo::Result<String> {
        self.node
            .table_column_description(column)
            .map_err(self.map_error())
    }

    fn get_row_extent_at(&self, row: i32, column: i32) -> fdo::Result<i32> {
        self.node
            .table_row_extent_at(row, column)
            .map_err(self.map_error())
    }

    fn get_column_extent_at(&self, row: i32, column: i32) -> fdo::Result<i32> {
        self.node
            .table_column_extent_at(row, column)
            .map_err(self.map_error())
    }

    fn get_row_header(&self, row: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        let header = self.node.table_row_header(row).map_err(self.map_error())?;
        Ok((self.object_address(header),))
    }

    fn get_column_header(&self, column: i32) -> fdo::Result<(OwnedObjectAddress,)> {
        let header = self
            .node
            .table_column_header(column)
            .map_err(self.map_error())?;
        Ok((self.object_address(header),))
    }

    fn get_selected_rows(&self) -> fdo::Result<Vec<i32>> {
        self.node.table_selected_rows().map_err(self.map_error())
    }

    fn get_selected_columns(&self) -> fdo::Result<Vec<i32>> {
        self.node.table_selected_columns().map_err(self.map_error())
    }

    fn is_row_selected(&self, row: i32) -> fdo::Result<bool> {
        self.node
            .table_is_row_selected(row)
            .map_err(self.map_error())
    }

    fn is_column_selected(&self, column: i32) -> fdo::Result<bool> {
        self.node
            .table_is_column_selected(column)
            .map_err(self.map_error())
    }

    fn is_selected(&self, row: i32, column: i32) -> fdo::Result<bool> {
        self.node
            .table_is_selected(row, column)
            .map_err(self.map_error())
    }

    fn add_row_selection(&self, row: i32) -> fdo::Result<bool> {
        self.node
            .table_add_row_selection(row)
            .map_err(self.map_error())
    }

    fn add_column_selection(&self, column: i32) -> fdo::Result<bool> {
        self.node
            .table_add_column_selection(column)
            .map_err(self.map_error())
    }

    fn remove_row_selection(&self, row: i32) -> fdo::Result<bool> {
        self.node
            .table_remove_row_selection(row)
            .map_err(self.map_error())
    }

    fn remove_column_selection(&self, column: i32) -> fdo::Result<bool> {
        self.node
            .table_remove_column_selection(column)
            .map_err(self.map_error())
    }

    fn get_row_column_extents_at_index(
        &self,
        index: i32,
    ) -> fdo::Result<(bool, i32, i32, i32, i32, bool)> {
        self.node
            .table_row_column_extents_at_index(index)
            .map_err(self.map_error())
    }
}

#[cfg(test)]
mod tests {
    use super::TableInterface;
    use crate::atspi::ObjectId;
    use accesskit::{
        ActionHandler, ActionRequest, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate,
    };
    use accesskit_atspi_common::{
        Adapter, AdapterCallback, AppContext, Event, FullNodeId, PlatformNode, WindowBounds,
    };
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

    fn bus_name() -> OwnedUniqueName {
        OwnedUniqueName::try_from(BUS_NAME).unwrap()
    }

    fn table_iface() -> (Adapter, TableInterface) {
        const WINDOW: NodeId = NodeId(0);
        const TABLE: NodeId = NodeId(1);
        const HEADER_ROW: NodeId = NodeId(2);
        const NAME_HEADER: NodeId = NodeId(3);
        const STATUS_HEADER: NodeId = NodeId(4);
        const DATA_ROW: NodeId = NodeId(5);
        const CELL_A: NodeId = NodeId(6);
        const CELL_DONE: NodeId = NodeId(7);

        let mut name_header = Node::new(Role::ColumnHeader);
        name_header.set_label("Name");
        let mut status_header = Node::new(Role::ColumnHeader);
        status_header.set_label("Status");
        let mut cell_a = Node::new(Role::Cell);
        cell_a.set_label("a");
        cell_a.set_selected(false);
        let mut cell_done = Node::new(Role::Cell);
        cell_done.set_label("done");
        cell_done.set_selected(true);

        let update = TreeUpdate {
            nodes: vec![
                (WINDOW, with_children(Role::Window, &[TABLE])),
                (TABLE, with_children(Role::Table, &[HEADER_ROW, DATA_ROW])),
                (
                    HEADER_ROW,
                    with_children(Role::Row, &[NAME_HEADER, STATUS_HEADER]),
                ),
                (NAME_HEADER, name_header),
                (STATUS_HEADER, status_header),
                (DATA_ROW, with_children(Role::Row, &[CELL_A, CELL_DONE])),
                (CELL_A, cell_a),
                (CELL_DONE, cell_done),
            ],
            tree: Some(TreeInfo::new(WINDOW)),
            tree_id: TreeId::ROOT,
            focus: WINDOW,
        };
        let app_context = AppContext::new(None);
        let adapter = Adapter::new(
            &app_context,
            NoOpCallback,
            update,
            false,
            WindowBounds::default(),
            NoOpActionHandler,
        );
        let table_id = adapter
            .platform_node(adapter.root_id())
            .child_at_index(0)
            .unwrap()
            .unwrap();
        let table = TableInterface::new(bus_name(), adapter.platform_node(table_id));
        (adapter, table)
    }

    fn cell_address(node: &PlatformNode, id: FullNodeId) -> crate::atspi::OwnedObjectAddress {
        ObjectId::Node {
            adapter: node.adapter_id(),
            node: id,
        }
        .to_address(&UniqueName::from_static_str_unchecked(BUS_NAME))
    }

    #[test]
    fn n_rows_and_n_columns_match_the_tree() {
        let (_adapter, table) = table_iface();
        assert_eq!(table.n_rows(), Ok(2));
        assert_eq!(table.n_columns(), Ok(2));
    }

    #[test]
    fn get_accessible_at_returns_the_real_cell_address() {
        let (adapter, table) = table_iface();
        let data_row = adapter
            .platform_node(adapter.root_id())
            .child_at_index(0)
            .unwrap()
            .unwrap();
        let data_row = adapter
            .platform_node(data_row)
            .child_at_index(1)
            .unwrap()
            .unwrap();
        let cell_done = adapter
            .platform_node(data_row)
            .child_at_index(1)
            .unwrap()
            .unwrap();
        let (address,) = table.get_accessible_at(1, 1).unwrap();
        assert_eq!(address, cell_address(&table.node, cell_done));
    }

    #[test]
    fn is_selected_reflects_live_cell_state() {
        let (_adapter, table) = table_iface();
        assert_eq!(table.is_selected(1, 1), Ok(true));
        assert_eq!(table.is_selected(1, 0), Ok(false));
    }
}
