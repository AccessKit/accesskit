// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{Point, Rect};
use pyo3::{prelude::*, types::PyList};

#[pyclass(module = "accesskit")]
pub struct NodeClassSet(accesskit::NodeClassSet);

#[pymethods]
impl NodeClassSet {
    #[new]
    pub fn __new__() -> Self {
        Self(accesskit::NodeClassSet::new())
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct Node(accesskit::Node);

impl Node {
    #[inline]
    fn inner(&self) -> &accesskit::Node {
        &self.0
    }
}

impl From<Node> for accesskit::Node {
    fn from(node: Node) -> accesskit::Node {
        node.0
    }
}

#[pymethods]
impl Node {
    #[getter]
    pub fn role(&self) -> accesskit::Role {
        self.inner().role()
    }

    pub fn supports_action(&self, action: accesskit::Action) -> bool {
        self.inner().supports_action(action)
    }
}

#[pyclass(module = "accesskit")]
pub struct NodeBuilder(Option<accesskit::NodeBuilder>);

impl NodeBuilder {
    #[inline]
    fn inner(&self) -> &accesskit::NodeBuilder {
        self.0.as_ref().unwrap()
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut accesskit::NodeBuilder {
        self.0.as_mut().unwrap()
    }
}

#[pymethods]
impl NodeBuilder {
    #[new]
    pub fn new(role: accesskit::Role) -> NodeBuilder {
        Self(Some(accesskit::NodeBuilder::new(role)))
    }

    pub fn build(&mut self, classes: &mut NodeClassSet) -> Node {
        let builder = self.0.take().unwrap();
        Node(builder.build(&mut classes.0))
    }

    #[getter]
    pub fn role(&self) -> accesskit::Role {
        self.inner().role()
    }

    pub fn set_role(&mut self, value: accesskit::Role) {
        self.inner_mut().set_role(value);
    }

    pub fn supports_action(&self, action: accesskit::Action) -> bool {
        self.inner().supports_action(action)
    }

    pub fn add_action(&mut self, action: accesskit::Action) {
        self.inner_mut().add_action(action)
    }

    pub fn remove_action(&mut self, action: accesskit::Action) {
        self.inner_mut().remove_action(action)
    }

    pub fn clear_actions(&mut self) {
        self.inner_mut().clear_actions()
    }
}

pub type NodeId = u64;

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct CustomAction(accesskit::CustomAction);

#[pymethods]
impl CustomAction {
    #[new]
    pub fn new(id: i32, description: &str) -> Self {
        Self(accesskit::CustomAction {
            id,
            description: description.into(),
        })
    }

    #[getter]
    pub fn id(&self) -> i32 {
        self.0.id
    }

    #[setter]
    pub fn set_id(&mut self, id: i32) {
        self.0.id = id;
    }

    #[getter]
    pub fn description(&self) -> &str {
        &self.0.description
    }

    #[setter]
    pub fn set_description(&mut self, description: &str) {
        self.0.description = description.into()
    }
}

impl From<CustomAction> for accesskit::CustomAction {
    fn from(action: CustomAction) -> Self {
        action.0
    }
}

impl From<accesskit::CustomAction> for CustomAction {
    fn from(action: accesskit::CustomAction) -> Self {
        Self(action)
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct TextPosition(accesskit::TextPosition);

#[pymethods]
impl TextPosition {
    #[new]
    pub fn new(node: NodeId, character_index: usize) -> Self {
        Self(accesskit::TextPosition {
            node: node.into(),
            character_index,
        })
    }

    #[getter]
    pub fn node(&self) -> NodeId {
        self.0.node.into()
    }

    #[setter]
    pub fn set_node(&mut self, node: NodeId) {
        self.0.node = node.into();
    }

    #[getter]
    pub fn character_index(&self) -> usize {
        self.0.character_index
    }

    #[setter]
    pub fn set_character_index(&mut self, character_index: usize) {
        self.0.character_index = character_index;
    }
}

impl From<accesskit::TextPosition> for TextPosition {
    fn from(position: accesskit::TextPosition) -> Self {
        Self(position)
    }
}

#[derive(Clone)]
#[pyclass(get_all, set_all, module = "accesskit")]
pub struct TextSelection {
    pub anchor: Py<TextPosition>,
    pub focus: Py<TextPosition>,
}

#[pymethods]
impl TextSelection {
    #[new]
    pub fn new(anchor: Py<TextPosition>, focus: Py<TextPosition>) -> Self {
        Self { anchor, focus }
    }
}

impl From<&accesskit::TextSelection> for TextSelection {
    fn from(selection: &accesskit::TextSelection) -> Self {
        Python::with_gil(|py| Self {
            anchor: Py::new(py, TextPosition::from(selection.anchor)).unwrap(),
            focus: Py::new(py, TextPosition::from(selection.focus)).unwrap(),
        })
    }
}

impl From<TextSelection> for accesskit::TextSelection {
    fn from(selection: TextSelection) -> Self {
        Python::with_gil(|py| accesskit::TextSelection {
            anchor: selection.anchor.as_ref(py).borrow().0,
            focus: selection.focus.as_ref(py).borrow().0,
        })
    }
}

impl From<TextSelection> for Box<accesskit::TextSelection> {
    fn from(selection: TextSelection) -> Self {
        Box::new(selection.into())
    }
}

macro_rules! clearer {
    ($clearer:ident) => {
        #[pymethods]
        impl NodeBuilder {
            pub fn $clearer(&mut self) {
                self.inner_mut().$clearer()
            }
        }
    };
}

macro_rules! getters {
    ($getter:ident, $macro_name:ident, $type:ty) => {
        $macro_name! { Node, $getter, $type }
        $macro_name! { NodeBuilder, $getter, $type }
    };
}

macro_rules! simple_getter {
    ($struct_name:ident, $getter:ident, $type:ty) => {
        #[pymethods]
        impl $struct_name {
            #[getter]
            pub fn $getter(&self) -> $type {
                self.inner().$getter()
            }
        }
    };
}

macro_rules! converting_getter {
    ($struct_name:ident, $getter:ident, $type:ty) => {
        #[pymethods]
        impl $struct_name {
            #[getter]
            pub fn $getter(&self) -> $type {
                self.inner().$getter().into()
            }
        }
    };
}

macro_rules! option_getter {
    ($struct_name:ident, $getter:ident, $type:ty) => {
        #[pymethods]
        impl $struct_name {
            #[getter]
            pub fn $getter(&self) -> $type {
                self.inner().$getter().map(Into::into)
            }
        }
    };
}

macro_rules! simple_setter {
    ($setter:ident, $setter_param:ty) => {
        #[pymethods]
        impl NodeBuilder {
            pub fn $setter(&mut self, value: $setter_param) {
                self.inner_mut().$setter(value);
            }
        }
    };
}

macro_rules! converting_setter {
    ($setter:ident, $setter_param:ty) => {
        #[pymethods]
        impl NodeBuilder {
            pub fn $setter(&mut self, value: $setter_param) {
                self.inner_mut().$setter(value.into());
            }
        }
    };
}

macro_rules! flag_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(getters! { $getter, simple_getter, bool }
        #[pymethods]
        impl NodeBuilder {
            pub fn $setter(&mut self) {
                self.inner_mut().$setter();
            }
        }
        clearer! { $clearer })*
    }
}

macro_rules! property_methods {
    (
        $(($getter:ident, $getter_macro:ident, $getter_result:ty, $setter:ident, $setter_macro:ident, $setter_param:ty, $clearer:ident)),+
    ) => {
        $(getters! { $getter, $getter_macro, $getter_result }
        $setter_macro! { $setter, $setter_param }
        clearer! { $clearer })*
    }
}

macro_rules! vec_property_methods {
    ($(($py_item_type:ty, $accesskit_item_type:ty, $getter:ident, $setter:ident, $pusher:ident, $clearer:ident)),+) => {
        #[pymethods]
        impl Node {
            $(#[getter]
            pub fn $getter(&self, py: Python) -> Py<PyList> {
                let values = self.inner().$getter().iter().cloned().map(<$py_item_type>::from).map(|i| i.into_py(py));
                PyList::new(py, values).into()
            })*
        }
        $(#[pymethods]
        impl NodeBuilder {
            #[getter]
            pub fn $getter(&self, py: Python) -> Py<PyList> {
                let values = self.inner().$getter().iter().cloned().map(<$py_item_type>::from).map(|i| i.into_py(py));
                PyList::new(py, values).into()
            }
            pub fn $setter(&mut self, values: &PyList) {
                let values = values
                    .iter()
                    .map(PyAny::extract::<$py_item_type>)
                    .filter_map(PyResult::ok)
                    .map(<$accesskit_item_type>::from)
                    .collect::<Vec<$accesskit_item_type>>();
                self.inner_mut().$setter(values);
            }
            pub fn $pusher(&mut self, item: $py_item_type) {
                self.inner_mut().$pusher(item.into());
            }
        }
        clearer! { $clearer })*
    }
}

macro_rules! string_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<&str>, $setter, simple_setter, String, $clearer)
        })*
    }
}

macro_rules! node_id_vec_property_methods {
    ($(($getter:ident, $setter:ident, $pusher:ident, $clearer:ident)),+) => {
        $(vec_property_methods! {
            (NodeId, accesskit::NodeId, $getter, $setter, $pusher, $clearer)
        })*
    }
}

macro_rules! node_id_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<NodeId>, $setter, converting_setter, NodeId, $clearer)
        })*
    }
}

macro_rules! f64_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<f64>, $setter, simple_setter, f64, $clearer)
        })*
    }
}

macro_rules! usize_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<usize>, $setter, simple_setter, usize, $clearer)
        })*
    }
}

macro_rules! color_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<u32>, $setter, simple_setter, u32, $clearer)
        })*
    }
}

macro_rules! text_decoration_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<accesskit::TextDecoration>, $setter, converting_setter, accesskit::TextDecoration, $clearer)
        })*
    }
}

macro_rules! length_slice_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, converting_getter, Vec<u8>, $setter, simple_setter, Vec<u8>, $clearer)
        })*
    }
}

macro_rules! coord_slice_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<Vec<f32>>, $setter, simple_setter, Vec<f32>, $clearer)
        })*
    }
}

macro_rules! bool_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<bool>, $setter, simple_setter, bool, $clearer)
        })*
    }
}

macro_rules! unique_enum_property_methods {
    ($(($type:ty, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            ($getter, option_getter, Option<$type>, $setter, simple_setter, $type, $clearer)
        })*
    }
}

flag_methods! {
    (is_hovered, set_hovered, clear_hovered),
    (is_hidden, set_hidden, clear_hidden),
    (is_linked, set_linked, clear_linked),
    (is_multiselectable, set_multiselectable, clear_multiselectable),
    (is_required, set_required, clear_required),
    (is_visited, set_visited, clear_visited),
    (is_busy, set_busy, clear_busy),
    (is_live_atomic, set_live_atomic, clear_live_atomic),
    (is_modal, set_modal, clear_modal),
    (is_touch_transparent, set_touch_transparent, clear_touch_transparent),
    (is_read_only, set_read_only, clear_read_only),
    (is_disabled, set_disabled, clear_disabled),
    (is_bold, set_bold, clear_bold),
    (is_italic, set_italic, clear_italic),
    (clips_children, set_clips_children, clear_clips_children),
    (is_line_breaking_object, set_is_line_breaking_object, clear_is_line_breaking_object),
    (is_page_breaking_object, set_is_page_breaking_object, clear_is_page_breaking_object),
    (is_spelling_error, set_is_spelling_error, clear_is_spelling_error),
    (is_grammar_error, set_is_grammar_error, clear_is_grammar_error),
    (is_search_match, set_is_search_match, clear_is_search_match),
    (is_suggestion, set_is_suggestion, clear_is_suggestion)
}

node_id_vec_property_methods! {
    (children, set_children, push_child, clear_children),
    (controls, set_controls, push_controlled, clear_controls),
    (details, set_details, push_detail, clear_details),
    (described_by, set_described_by, push_described_by, clear_described_by),
    (flow_to, set_flow_to, push_flow_to, clear_flow_to),
    (labelled_by, set_labelled_by, push_labelled_by, clear_labelled_by),
    (radio_group, set_radio_group, push_to_radio_group, clear_radio_group)
}

node_id_property_methods! {
    (active_descendant, set_active_descendant, clear_active_descendant),
    (error_message, set_error_message, clear_error_message),
    (in_page_link_target, set_in_page_link_target, clear_in_page_link_target),
    (member_of, set_member_of, clear_member_of),
    (next_on_line, set_next_on_line, clear_next_on_line),
    (previous_on_line, set_previous_on_line, clear_previous_on_line),
    (popup_for, set_popup_for, clear_popup_for),
    (table_header, set_table_header, clear_table_header),
    (table_row_header, set_table_row_header, clear_table_row_header),
    (table_column_header, set_table_column_header, clear_table_column_header)
}

string_property_methods! {
    (name, set_name, clear_name),
    (description, set_description, clear_description),
    (value, set_value, clear_value),
    (access_key, set_access_key, clear_access_key),
    (class_name, set_class_name, clear_class_name),
    (font_family, set_font_family, clear_font_family),
    (html_tag, set_html_tag, clear_html_tag),
    (inner_html, set_inner_html, clear_inner_html),
    (keyboard_shortcut, set_keyboard_shortcut, clear_keyboard_shortcut),
    (language, set_language, clear_language),
    (placeholder, set_placeholder, clear_placeholder),
    (role_description, set_role_description, clear_role_description),
    (state_description, set_state_description, clear_state_description),
    (tooltip, set_tooltip, clear_tooltip),
    (url, set_url, clear_url)
}

f64_property_methods! {
    (scroll_x, set_scroll_x, clear_scroll_x),
    (scroll_x_min, set_scroll_x_min, clear_scroll_x_min),
    (scroll_x_max, set_scroll_x_max, clear_scroll_x_max),
    (scroll_y, set_scroll_y, clear_scroll_y),
    (scroll_y_min, set_scroll_y_min, clear_scroll_y_min),
    (scroll_y_max, set_scroll_y_max, clear_scroll_y_max),
    (numeric_value, set_numeric_value, clear_numeric_value),
    (min_numeric_value, set_min_numeric_value, clear_min_numeric_value),
    (max_numeric_value, set_max_numeric_value, clear_max_numeric_value),
    (numeric_value_step, set_numeric_value_step, clear_numeric_value_step),
    (numeric_value_jump, set_numeric_value_jump, clear_numeric_value_jump),
    (font_size, set_font_size, clear_font_size),
    (font_weight, set_font_weight, clear_font_weight)
}

usize_property_methods! {
    (table_row_count, set_table_row_count, clear_table_row_count),
    (table_column_count, set_table_column_count, clear_table_column_count),
    (table_row_index, set_table_row_index, clear_table_row_index),
    (table_column_index, set_table_column_index, clear_table_column_index),
    (table_cell_column_index, set_table_cell_column_index, clear_table_cell_column_index),
    (table_cell_column_span, set_table_cell_column_span, clear_table_cell_column_span),
    (table_cell_row_index, set_table_cell_row_index, clear_table_cell_row_index),
    (table_cell_row_span, set_table_cell_row_span, clear_table_cell_row_span),
    (hierarchical_level, set_hierarchical_level, clear_hierarchical_level),
    (size_of_set, set_size_of_set, clear_size_of_set),
    (position_in_set, set_position_in_set, clear_position_in_set)
}

color_property_methods! {
    (color_value, set_color_value, clear_color_value),
    (background_color, set_background_color, clear_background_color),
    (foreground_color, set_foreground_color, clear_foreground_color)
}

text_decoration_property_methods! {
    (overline, set_overline, clear_overline),
    (strikethrough, set_strikethrough, clear_strikethrough),
    (underline, set_underline, clear_underline)
}

length_slice_property_methods! {
    (character_lengths, set_character_lengths, clear_character_lengths),
    (word_lengths, set_word_lengths, clear_word_lengths)
}

coord_slice_property_methods! {
    (character_positions, set_character_positions, clear_character_positions),
    (character_widths, set_character_widths, clear_character_widths)
}

bool_property_methods! {
    (is_expanded, set_expanded, clear_expanded),
    (is_selected, set_selected, clear_selected)
}

unique_enum_property_methods! {
    (accesskit::Invalid, invalid, set_invalid, clear_invalid),
    (accesskit::Checked, checked, set_checked, clear_checked),
    (accesskit::Live, live, set_live, clear_live),
    (accesskit::DefaultActionVerb, default_action_verb, set_default_action_verb, clear_default_action_verb),
    (accesskit::TextDirection, text_direction, set_text_direction, clear_text_direction),
    (accesskit::Orientation, orientation, set_orientation, clear_orientation),
    (accesskit::SortDirection, sort_direction, set_sort_direction, clear_sort_direction),
    (accesskit::AriaCurrent, aria_current, set_aria_current, clear_aria_current),
    (accesskit::AutoComplete, auto_complete, set_auto_complete, clear_auto_complete),
    (accesskit::HasPopup, has_popup, set_has_popup, clear_has_popup),
    (accesskit::ListStyle, list_style, set_list_style, clear_list_style),
    (accesskit::TextAlign, text_align, set_text_align, clear_text_align),
    (accesskit::VerticalOffset, vertical_offset, set_vertical_offset, clear_vertical_offset)
}

property_methods! {
    (transform, option_getter, Option<crate::Affine>, set_transform, simple_setter, crate::Affine, clear_transform),
    (bounds, option_getter, Option<crate::Rect>, set_bounds, converting_setter, crate::Rect, clear_bounds),
    (text_selection, option_getter, Option<TextSelection>, set_text_selection, simple_setter, TextSelection, clear_text_selection)
}

vec_property_methods! {
    (CustomAction, accesskit::CustomAction, custom_actions, set_custom_actions, push_custom_action, clear_custom_actions)
}

#[derive(Clone)]
#[pyclass(module = "accesskit", get_all, set_all)]
pub struct Tree {
    pub root: NodeId,
    pub app_name: Option<String>,
    pub toolkit_name: Option<String>,
    pub toolkit_version: Option<String>,
}

#[pymethods]
impl Tree {
    #[new]
    pub fn new(root: NodeId) -> Self {
        Self {
            root,
            app_name: None,
            toolkit_name: None,
            toolkit_version: None,
        }
    }
}

impl From<Tree> for accesskit::Tree {
    fn from(tree: Tree) -> Self {
        Self {
            root: tree.root.into(),
            app_name: tree.app_name,
            toolkit_name: tree.toolkit_name,
            toolkit_version: tree.toolkit_version,
        }
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit", get_all, set_all)]
pub struct TreeUpdate {
    pub nodes: Py<PyList>,
    pub tree: Option<Py<Tree>>,
    pub focus: NodeId,
}

#[pymethods]
impl TreeUpdate {
    #[new]
    pub fn new(py: Python<'_>, focus: NodeId) -> Self {
        Self {
            nodes: PyList::empty(py).into(),
            tree: None,
            focus,
        }
    }
}

impl From<TreeUpdate> for accesskit::TreeUpdate {
    fn from(update: TreeUpdate) -> Self {
        Python::with_gil(|py| Self {
            nodes: update
                .nodes
                .as_ref(py)
                .iter()
                .map(PyAny::extract::<(NodeId, Node)>)
                .filter_map(Result::ok)
                .map(|(id, node)| (id.into(), node.into()))
                .collect(),
            tree: update.tree.map(|tree| {
                let tree = tree.as_ref(py).borrow();
                accesskit::Tree {
                    root: tree.root.into(),
                    app_name: tree.app_name.clone(),
                    toolkit_name: tree.toolkit_name.clone(),
                    toolkit_version: tree.toolkit_version.clone(),
                }
            }),
            focus: update.focus.into(),
        })
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionDataKind {
    CustomAction,
    Value,
    NumericValue,
    ScrollTargetRect,
    ScrollToPoint,
    SetScrollOffset,
    SetTextSelection,
}

#[pyclass(get_all, module = "accesskit")]
pub struct ActionRequest {
    pub action: accesskit::Action,
    pub target: NodeId,
    pub data: Option<(ActionDataKind, Py<PyAny>)>,
}

impl From<accesskit::ActionRequest> for ActionRequest {
    fn from(request: accesskit::ActionRequest) -> Self {
        Python::with_gil(|py| Self {
            action: request.action,
            target: request.target.into(),
            data: request.data.map(|data| match data {
                accesskit::ActionData::CustomAction(action) => {
                    (ActionDataKind::CustomAction, action.into_py(py))
                }
                accesskit::ActionData::Value(value) => (ActionDataKind::Value, value.into_py(py)),
                accesskit::ActionData::NumericValue(value) => {
                    (ActionDataKind::NumericValue, value.into_py(py))
                }
                accesskit::ActionData::ScrollTargetRect(rect) => (
                    ActionDataKind::ScrollTargetRect,
                    Rect::from(rect).into_py(py),
                ),
                accesskit::ActionData::ScrollToPoint(point) => (
                    ActionDataKind::ScrollToPoint,
                    Point::from(point).into_py(py),
                ),
                accesskit::ActionData::SetScrollOffset(point) => (
                    ActionDataKind::SetScrollOffset,
                    Point::from(point).into_py(py),
                ),
                accesskit::ActionData::SetTextSelection(selection) => (
                    ActionDataKind::SetTextSelection,
                    TextSelection::from(&selection).into_py(py),
                ),
            }),
        })
    }
}

pub struct PythonActionHandler(pub(crate) Py<PyAny>);

impl accesskit::ActionHandler for PythonActionHandler {
    fn do_action(&mut self, request: accesskit::ActionRequest) {
        let request = ActionRequest::from(request);
        Python::with_gil(|py| {
            self.0.call(py, (request,), None).unwrap();
        });
    }
}
