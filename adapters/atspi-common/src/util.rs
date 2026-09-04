// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Point, Rect, Role, ScrollHint};
use accesskit_consumer::{FullNodeId, NodeRef, TextPosition, TextRange};
use atspi_common::{CoordType, Granularity, ScrollType};

use crate::{Error, filters::filter};

#[derive(Clone, Copy, Default, Debug)]
pub struct WindowBounds {
    pub outer: Rect,
    pub inner: Rect,
}

impl WindowBounds {
    pub fn new(outer: Rect, inner: Rect) -> Self {
        Self { outer, inner }
    }

    pub(crate) fn accesskit_point_to_atspi_point(
        &self,
        point: Point,
        parent: Option<NodeRef>,
        coord_type: CoordType,
    ) -> Point {
        let origin = self.origin(parent, coord_type);
        Point::new(origin.x + point.x, origin.y + point.y)
    }

    pub(crate) fn atspi_point_to_accesskit_point(
        &self,
        point: Point,
        parent: Option<NodeRef>,
        coord_type: CoordType,
    ) -> Point {
        let origin = self.origin(parent, coord_type);
        Point::new(point.x - origin.x, point.y - origin.y)
    }

    fn origin(&self, parent: Option<NodeRef>, coord_type: CoordType) -> Point {
        match coord_type {
            CoordType::Screen => self.inner.origin(),
            CoordType::Window => Point::ZERO,
            CoordType::Parent => {
                if let Some(parent) = parent {
                    let parent_origin = parent.bounding_box().unwrap_or_default().origin();
                    Point::new(-parent_origin.x, -parent_origin.y)
                } else {
                    self.inner.origin()
                }
            }
        }
    }
}

pub(crate) fn text_position_from_offset<'a>(
    node: &'a NodeRef,
    offset: i32,
) -> Option<TextPosition<'a>> {
    let index = offset.try_into().ok()?;
    node.text_position_from_global_usv_index(index)
}

pub(crate) fn text_range_from_offset<'a>(
    node: &'a NodeRef,
    offset: i32,
    granularity: Granularity,
) -> Result<TextRange<'a>, Error> {
    let start_offset = text_position_from_offset(node, offset).ok_or(Error::IndexOutOfRange)?;
    let start = match granularity {
        Granularity::Char => start_offset,
        Granularity::Line if start_offset.is_line_start() => start_offset,
        Granularity::Line => start_offset.backward_to_line_start(),
        Granularity::Paragraph if start_offset.is_paragraph_start() => start_offset,
        Granularity::Paragraph => start_offset.backward_to_paragraph_start(),
        Granularity::Sentence => return Err(Error::UnsupportedTextGranularity),
        Granularity::Word if start_offset.is_word_start() => start_offset,
        Granularity::Word => start_offset.backward_to_word_start(),
    };
    let end = match granularity {
        Granularity::Char if start_offset.is_document_end() => start_offset,
        Granularity::Char => start.forward_to_character_end(),
        Granularity::Line => start.forward_to_line_end(),
        Granularity::Paragraph => start.forward_to_paragraph_end(),
        Granularity::Sentence => return Err(Error::UnsupportedTextGranularity),
        Granularity::Word => start.forward_to_word_end(),
    };
    let mut range = start.to_degenerate_range();
    range.set_end(end);
    Ok(range)
}

pub(crate) fn text_range_from_offsets<'a>(
    node: &'a NodeRef,
    start_offset: i32,
    end_offset: i32,
) -> Option<TextRange<'a>> {
    let start = text_position_from_offset(node, start_offset)?;
    let end = if end_offset == -1 {
        node.document_range().end()
    } else {
        text_position_from_offset(node, end_offset)?
    };

    let mut range = start.to_degenerate_range();
    range.set_end(end);
    Some(range)
}

pub(crate) fn text_range_bounds_from_offsets(
    node: &NodeRef,
    start_offset: i32,
    end_offset: i32,
) -> Option<Rect> {
    text_range_from_offsets(node, start_offset, end_offset)?
        .bounding_boxes()
        .into_iter()
        .reduce(|rect1, rect2| rect1.union(rect2))
}

pub(crate) fn atspi_scroll_type_to_scroll_hint(scroll_type: ScrollType) -> Option<ScrollHint> {
    match scroll_type {
        ScrollType::TopLeft => Some(ScrollHint::TopLeft),
        ScrollType::BottomRight => Some(ScrollHint::BottomRight),
        ScrollType::TopEdge => Some(ScrollHint::TopEdge),
        ScrollType::BottomEdge => Some(ScrollHint::BottomEdge),
        ScrollType::LeftEdge => Some(ScrollHint::LeftEdge),
        ScrollType::RightEdge => Some(ScrollHint::RightEdge),
        ScrollType::Anywhere => None,
    }
}

fn is_table_cell_role(role: Role) -> bool {
    matches!(
        role,
        Role::Cell | Role::GridCell | Role::RowHeader | Role::ColumnHeader
    )
}

fn collect_table_rows<'a>(node: &NodeRef<'a>, rows: &mut Vec<NodeRef<'a>>) {
    for child in node.filtered_children(filter) {
        match child.role() {
            Role::Row => rows.push(child),
            Role::RowGroup => collect_table_rows(&child, rows),
            _ => {}
        }
    }
}

fn collect_table_cells<'a>(node: &NodeRef<'a>, cells: &mut Vec<NodeRef<'a>>) {
    for child in node.filtered_children(filter) {
        if is_table_cell_role(child.role()) {
            cells.push(child);
        } else {
            collect_table_cells(&child, cells);
        }
    }
}

/// `grid[row][col]` for a `Table`/`Grid`/`ListGrid` node. Built from
/// `Role::Row` children, the shape every known accesskit consumer produces.
/// Falls back to `row_index`/`column_index` on descendant cells for
/// ARIA-grid-style tables that have no `Row` children at all.
pub(crate) fn table_grid<'a>(table: &NodeRef<'a>) -> Vec<Vec<Option<NodeRef<'a>>>> {
    let mut rows = Vec::new();
    collect_table_rows(table, &mut rows);
    if !rows.is_empty() {
        return rows
            .into_iter()
            .map(|row| {
                row.filtered_children(filter)
                    .filter(|cell| is_table_cell_role(cell.role()))
                    .map(Some)
                    .collect()
            })
            .collect();
    }

    let mut cells = Vec::new();
    collect_table_cells(table, &mut cells);
    let mut grid: Vec<Vec<Option<NodeRef<'a>>>> = Vec::new();
    for cell in cells {
        let (Some(row), Some(column)) = (cell.data().row_index(), cell.data().column_index())
        else {
            continue;
        };
        if grid.len() <= row {
            grid.resize_with(row + 1, Vec::new);
        }
        if grid[row].len() <= column {
            grid[row].resize(column + 1, None);
        }
        grid[row][column] = Some(cell);
    }
    grid
}

pub(crate) fn table_column_count(grid: &[Vec<Option<NodeRef>>]) -> usize {
    grid.iter().map(Vec::len).max().unwrap_or(0)
}

pub(crate) fn table_grid_cell<'a, 'g>(
    grid: &'g [Vec<Option<NodeRef<'a>>>],
    row: usize,
    column: usize,
) -> Option<&'g NodeRef<'a>> {
    grid.get(row)?.get(column)?.as_ref()
}

pub(crate) fn table_row_cells<'a, 'g>(
    grid: &'g [Vec<Option<NodeRef<'a>>>],
    row: usize,
) -> impl Iterator<Item = &'g NodeRef<'a>> {
    grid.get(row)
        .into_iter()
        .flat_map(|row| row.iter().filter_map(Option::as_ref))
}

pub(crate) fn table_column_cells<'a, 'g>(
    grid: &'g [Vec<Option<NodeRef<'a>>>],
    column: usize,
) -> impl Iterator<Item = &'g NodeRef<'a>> {
    grid.iter()
        .filter_map(move |row| row.get(column).and_then(Option::as_ref))
}

fn is_data_cell_role(role: Role) -> bool {
    matches!(role, Role::Cell | Role::GridCell)
}

pub(crate) fn table_row_is_selected(grid: &[Vec<Option<NodeRef>>], row: usize) -> bool {
    let mut any = false;
    for cell in table_row_cells(grid, row).filter(|cell| is_data_cell_role(cell.role())) {
        any = true;
        if cell.is_selected() != Some(true) {
            return false;
        }
    }
    any
}

pub(crate) fn table_column_is_selected(grid: &[Vec<Option<NodeRef>>], column: usize) -> bool {
    let mut any = false;
    for cell in table_column_cells(grid, column).filter(|cell| is_data_cell_role(cell.role())) {
        any = true;
        if cell.is_selected() != Some(true) {
            return false;
        }
    }
    any
}

pub(crate) fn find_table_ancestor<'a>(node: &NodeRef<'a>) -> Option<NodeRef<'a>> {
    let mut current = node.filtered_parent(&filter);
    while let Some(candidate) = current {
        if matches!(candidate.role(), Role::Table | Role::Grid | Role::ListGrid) {
            return Some(candidate);
        }
        current = candidate.filtered_parent(&filter);
    }
    None
}

pub(crate) fn find_cell_position(
    grid: &[Vec<Option<NodeRef>>],
    id: FullNodeId,
) -> Option<(usize, usize)> {
    for (row, cells) in grid.iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            if cell.as_ref().is_some_and(|cell| cell.id() == id) {
                return Some((row, column));
            }
        }
    }
    None
}
