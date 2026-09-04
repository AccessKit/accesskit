// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_atspi_common::PlatformNode;
use zbus::{fdo, interface, names::OwnedUniqueName};

use crate::atspi::{ObjectId, OwnedObjectAddress};

pub(crate) struct TableCellInterface {
    bus_name: OwnedUniqueName,
    node: PlatformNode,
}

impl TableCellInterface {
    pub fn new(bus_name: OwnedUniqueName, node: PlatformNode) -> Self {
        Self { bus_name, node }
    }

    fn map_error(&self) -> impl '_ + FnOnce(accesskit_atspi_common::Error) -> fdo::Error {
        |error| crate::util::map_error_from_node(&self.node, error)
    }
}

#[interface(name = "org.a11y.atspi.TableCell")]
impl TableCellInterface {
    #[zbus(property)]
    fn row_index(&self) -> fdo::Result<i32> {
        let (row, _) = self.node.table_cell_position().map_err(self.map_error())?;
        Ok(row)
    }

    #[zbus(property)]
    fn column_index(&self) -> fdo::Result<i32> {
        let (_, column) = self.node.table_cell_position().map_err(self.map_error())?;
        Ok(column)
    }

    fn get_position(&self) -> fdo::Result<(i32, i32)> {
        self.node.table_cell_position().map_err(self.map_error())
    }

    fn get_row_column_span(&self) -> fdo::Result<(bool, i32, i32, i32, i32)> {
        self.node
            .table_cell_row_column_span()
            .map_err(self.map_error())
    }

    fn get_table(&self) -> fdo::Result<(OwnedObjectAddress,)> {
        let table = self.node.table_cell_table().map_err(self.map_error())?;
        let table = table.map(|node| ObjectId::Node {
            adapter: self.node.adapter_id(),
            node,
        });
        Ok(super::optional_object_address(&self.bus_name, table))
    }
}
