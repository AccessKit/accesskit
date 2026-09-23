// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Action, Role, Toggled};
use accesskit_consumer::{common_filter_with_root_exception, FilterResult, Node};
use ohos_accessibility_binding::{
    AccessibilityError, AccessibleRect, ActionType, ElementInfo, GridInfo, GridItemInfo, RangeInfo,
    Result,
};

use crate::adapter::{NodeIdMap, ROOT_PARENT_ID};

pub(crate) fn filter(node: &Node) -> FilterResult {
    common_filter_with_root_exception(node)
}

pub(crate) fn populate_element(
    node: &Node,
    ids: &mut NodeIdMap,
    accessibility_focus: Option<accesskit_consumer::NodeId>,
    screen_offset: (i32, i32),
    element: &mut ElementInfo<'_>,
) -> Result<()> {
    let element_id = ids.get_or_create(node);
    let parent_id = node
        .filtered_parent(&filter)
        .map(|parent| ids.get_or_create(&parent))
        .unwrap_or(ROOT_PARENT_ID);
    let child_ids = node
        .filtered_children(&filter)
        .map(|child| i64::from(ids.get_or_create(&child)))
        .collect::<Vec<_>>();

    element
        .set_element_id(element_id)?
        .set_parent_id(parent_id)?
        .set_component_type(component_type(node.role()))?
        .set_enabled(!node.is_disabled())?
        .set_visible(!node.is_hidden())?
        .set_focusable(node.is_focusable(&filter))?
        .set_focused(node.is_focused())?
        .set_accessibility_focused(accessibility_focus == Some(node.id()))?
        .set_selected(node.is_selected().unwrap_or(false))?
        .set_clickable(node.is_clickable(&filter))?
        .set_long_clickable(false)?
        .set_scrollable(is_scrollable(node))?
        .set_editable(node.is_text_input() && !node.is_read_only())?
        .set_password(node.role() == Role::PasswordInput)?
        .set_accessibility_level(if node.is_root() { "no" } else { "yes" })?;

    // ArkUI rejects a zero child count with BAD_PARAMETER. Leaf nodes already
    // have an empty child list by default, so only call the setter when there
    // are IDs to copy.
    if !child_ids.is_empty() {
        element.set_child_node_ids(&child_ids)?;
    }

    if let Some(contents) = node.value() {
        element.set_contents(&contents)?;
    }
    if let Some(label) = node.label() {
        element.set_accessibility_text(&label)?;
    }
    if let Some(description) = node.description() {
        element.set_accessibility_description(&description)?;
    }
    if let Some(hint) = node.placeholder() {
        element.set_hint_text(hint)?.set_is_hint(true)?;
    }
    #[cfg(feature = "api-24")]
    if let Some(identifier) = node.author_id() {
        element.set_component_identifier(identifier)?;
    }
    if let Some(bounds) = node.bounding_box() {
        element.set_screen_rect(AccessibleRect {
            left: coordinate_to_i32(bounds.x0).saturating_add(screen_offset.0),
            top: coordinate_to_i32(bounds.y0).saturating_add(screen_offset.1),
            right: coordinate_to_i32(bounds.x1).saturating_add(screen_offset.0),
            bottom: coordinate_to_i32(bounds.y1).saturating_add(screen_offset.1),
        })?;
    }

    if let Some(toggled) = node.toggled() {
        element
            .set_checkable(true)?
            .set_checked(toggled != Toggled::False)?;
    }

    if let (Some(min), Some(max), Some(current)) = (
        node.min_numeric_value(),
        node.max_numeric_value(),
        node.numeric_value(),
    ) {
        element.set_range_info(RangeInfo { min, max, current })?;
    }

    let data = node.data();
    if data.row_count().is_some() || data.column_count().is_some() {
        element.set_grid_info(GridInfo {
            rows: usize_to_i32(data.row_count().unwrap_or(0))?,
            columns: usize_to_i32(data.column_count().unwrap_or(0))?,
            selection_mode: i32::from(data.is_multiselectable()),
        })?;
    }
    if data.row_index().is_some() || data.column_index().is_some() {
        element.set_grid_item_info(GridItemInfo {
            heading: matches!(
                node.role(),
                Role::RowHeader | Role::ColumnHeader | Role::Heading
            ),
            selected: node.is_selected().unwrap_or(false),
            row_index: usize_to_i32(data.row_index().unwrap_or(0))?,
            column_index: usize_to_i32(data.column_index().unwrap_or(0))?,
            row_span: usize_to_i32(data.row_span().unwrap_or(1))?,
            column_span: usize_to_i32(data.column_span().unwrap_or(1))?,
        })?;
    }

    if let Some(selection) = node.text_selection() {
        element
            .set_selected_text_start(usize_to_i32(selection.start().to_global_utf16_index())?)?
            .set_selected_text_end(usize_to_i32(selection.end().to_global_utf16_index())?)?;
    }
    if let Some(position) = node.position_in_set() {
        element.set_current_item_index(usize_to_i32(position)?)?;
    }
    if let Some(count) = node.size_of_set() {
        element.set_item_count(usize_to_i32(count)?)?;
    }

    let actions = supported_actions(node);
    if !actions.is_empty() {
        element.set_operation_actions(&actions)?;
    }

    Ok(())
}

fn supported_actions(node: &Node) -> Vec<(ActionType, &'static str)> {
    let mut actions = vec![
        (
            ActionType::GainAccessibilityFocus,
            "gain accessibility focus",
        ),
        (
            ActionType::ClearAccessibilityFocus,
            "clear accessibility focus",
        ),
    ];
    if node.supports_action(Action::Click, &filter) {
        actions.push((ActionType::Click, "click"));
    }
    if node.supports_action(Action::ScrollDown, &filter)
        || node.supports_action(Action::ScrollRight, &filter)
    {
        actions.push((ActionType::ScrollForward, "scroll forward"));
    }
    if node.supports_action(Action::ScrollUp, &filter)
        || node.supports_action(Action::ScrollLeft, &filter)
    {
        actions.push((ActionType::ScrollBackward, "scroll backward"));
    }
    if node.supports_action(Action::SetTextSelection, &filter) {
        actions.push((ActionType::SelectText, "select text"));
        actions.push((ActionType::SetCursorPosition, "set cursor position"));
    }
    if node.supports_action(Action::SetValue, &filter) {
        actions.push((ActionType::SetText, "set text"));
    }
    actions
}

fn is_scrollable(node: &Node) -> bool {
    [
        Action::ScrollDown,
        Action::ScrollLeft,
        Action::ScrollRight,
        Action::ScrollUp,
    ]
    .into_iter()
    .any(|action| node.supports_action(action, &filter))
}

fn coordinate_to_i32(value: f64) -> i32 {
    if value.is_nan() {
        0
    } else {
        value.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
    }
}

pub(crate) fn usize_to_i32(value: usize) -> Result<i32> {
    i32::try_from(value).map_err(|_| AccessibilityError::BadParameter)
}

fn component_type(role: Role) -> &'static str {
    match role {
        Role::Button | Role::DefaultButton | Role::PdfActionableHighlight => "button",
        Role::CheckBox => "checkbox",
        Role::RadioButton => "radio",
        Role::Switch => "switch",
        Role::TextInput
        | Role::MultilineTextInput
        | Role::SearchInput
        | Role::EmailInput
        | Role::NumberInput
        | Role::PasswordInput
        | Role::PhoneNumberInput
        | Role::UrlInput => "textInput",
        Role::Label | Role::TextRun => "text",
        Role::Image | Role::SvgRoot => "image",
        Role::Link => "link",
        Role::List | Role::ListBox => "list",
        Role::ListItem | Role::ListBoxOption => "listItem",
        Role::Grid | Role::Table | Role::TreeGrid => "grid",
        Role::Row => "row",
        Role::Cell | Role::LayoutTableCell => "cell",
        Role::Slider => "slider",
        Role::ProgressIndicator | Role::Meter => "progress",
        Role::ScrollView | Role::ScrollBar => "scroll",
        Role::Dialog | Role::AlertDialog => "dialog",
        Role::Menu | Role::MenuBar | Role::MenuListPopup => "menu",
        Role::MenuItem | Role::MenuItemCheckBox | Role::MenuItemRadio => "menuItem",
        Role::TabList => "tabs",
        Role::Tab => "tab",
        Role::Window | Role::RootWebArea | Role::Document => "root",
        _ => "container",
    }
}
