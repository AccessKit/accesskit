// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{box_from_ptr, mut_from_ptr, opt_struct, ref_from_ptr, BoxCastPtr, CastPtr};
use accesskit::*;
use paste::paste;
use std::{
    ffi::{CStr, CString},
    mem,
    num::NonZeroU128,
    os::raw::{c_char, c_void},
    ptr, slice,
};

pub struct node_class_set {
    _private: [u8; 0],
}

impl CastPtr for node_class_set {
    type RustType = NodeClassSet;
}

impl BoxCastPtr for node_class_set {}

impl node_class_set {
    #[no_mangle]
    pub extern "C" fn accesskit_node_class_set_new() -> *mut node_class_set {
        let set = NodeClassSet::new();
        BoxCastPtr::to_mut_ptr(set)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_class_set_free(set: *mut node_class_set) {
        drop(box_from_ptr(set));
    }
}

pub struct node {
    _private: [u8; 0],
}

impl CastPtr for node {
    type RustType = Node;
}

impl BoxCastPtr for node {}

impl node {
    #[no_mangle]
    pub extern "C" fn accesskit_node_free(node: *mut node) {
        drop(box_from_ptr(node));
    }
}

pub struct node_builder {
    _private: [u8; 0],
}

impl CastPtr for node_builder {
    type RustType = NodeBuilder;
}

impl BoxCastPtr for node_builder {}

macro_rules! clearer {
    ($clearer:ident) => {
        paste! {
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $clearer>](builder: *mut node_builder) {
                    let builder = mut_from_ptr(builder);
                    builder.$clearer()
                }
            }
        }
    };
}

macro_rules! flag_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        paste! {
            impl node {
                $(#[no_mangle]
                pub extern "C" fn [<accesskit_node_ $getter>](node: *const node) -> bool {
                    let node = ref_from_ptr(node);
                    node.$getter()
                })*
            }
            $(impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $getter>](builder: *const node_builder) -> bool {
                    let builder = ref_from_ptr(builder);
                    builder.$getter()
                }
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $setter>](builder: *mut node_builder) {
                    let builder = mut_from_ptr(builder);
                    builder.$setter()
                }
            }
            clearer! { $clearer })*
        }
    }
}

macro_rules! array_setter {
    ($setter:ident, $ffi_type:ty, $rust_type:ty) => {
        paste! {
            impl node_builder {
                /// Caller is responsible for freeing `values`.
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $setter>](builder: *mut node_builder, length: usize, values: *const $ffi_type) {
                    let builder = mut_from_ptr(builder);
                    let values = unsafe {
                        slice::from_raw_parts(values, length)
                            .iter()
                            .cloned()
                            .map(From::from)
                            .collect::<Vec<$rust_type>>()
                    };
                    builder.$setter(values);
                }
            }
        }
    }
}

macro_rules! property_getters {
    ($getter:ident, *const $getter_result:tt) => {
        paste! {
            impl node {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_ $getter>](node: *const node) -> *const $getter_result {
                    let node = ref_from_ptr(node);
                    match node.$getter() {
                        Some(value) => value as *const _,
                        None => ptr::null(),
                    }
                }
            }
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $getter>](builder: *const node_builder) -> *const $getter_result {
                    let builder = ref_from_ptr(builder);
                    match builder.$getter() {
                        Some(value) => value as *const _,
                        None => ptr::null(),
                    }
                }
            }
        }
    };
    ($getter:ident, *mut $getter_result:tt) => {
        paste! {
            impl node {
                /// Caller is responsible for freeing the returned value.
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_ $getter>](node: *const node) -> *mut $getter_result {
                    let node = ref_from_ptr(node);
                    BoxCastPtr::to_mut_ptr(node.$getter().into())
                }
            }
            impl node_builder {
                /// Caller is responsible for freeing the returned value.
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $getter>](builder: *const node_builder) -> *const $getter_result {
                    let builder = ref_from_ptr(builder);
                    BoxCastPtr::to_mut_ptr(builder.$getter().into())
                }
            }
        }
    };
    ($getter:ident, $getter_result:tt) => {
        paste! {
            impl node {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_ $getter>](node: *const node) -> $getter_result {
                    let node = ref_from_ptr(node);
                    node.$getter().into()
                }
            }
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $getter>](builder: *const node_builder) -> $getter_result {
                    let builder = ref_from_ptr(builder);
                    builder.$getter().into()
                }
            }
        }
    }
}

macro_rules! simple_property_methods {
    ($getter:ident, $getter_result:tt, $setter:ident, $setter_param:tt, $clearer:ident) => {
        paste! {
            property_getters! { $getter, $getter_result }
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $setter>](builder: *mut node_builder, value: $setter_param) {
                    let builder = mut_from_ptr(builder);
                    builder.$setter(value.into());
                }
            }
            clearer! { $clearer }
        }
    };
    ($getter:ident, *const $getter_result:tt, $setter:ident, $setter_param:tt, $clearer:ident) => {
        paste! {
            property_getters! { $getter, *const $getter_result }
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $setter>](builder: *mut node_builder, value: $setter_param) {
                    let builder = mut_from_ptr(builder);
                    builder.$setter(Box::new(value));
                }
            }
            clearer! { $clearer }
        }
    };
    ($getter:ident, $getter_result:tt, $setter:ident, *const $setter_param:tt, $clearer:ident) => {
        property_getters! { $getter, $getter_result }
        array_setter! { $setter, $setter_param, $setter_param }
        clearer! { $clearer }
    }
}

macro_rules! slice_struct {
    ($struct_name:ident, $rust_type:ty, $ffi_type:ty) => {
        #[repr(C)]
        pub struct $struct_name {
            pub length: usize,
            pub values: *const $ffi_type,
        }
        impl From<&[$rust_type]> for $struct_name {
            fn from(values: &[$rust_type]) -> Self {
                Self {
                    length: values.len(),
                    values: values.as_ptr() as *const _,
                }
            }
        }
        impl From<$struct_name> for Vec<$rust_type> {
            fn from(values: $struct_name) -> Self {
                unsafe {
                    slice::from_raw_parts(values.values as *mut $rust_type, values.length).to_vec()
                }
            }
        }
    };
}

macro_rules! array_struct {
    ($struct_name:ident, $rust_type:ty, $ffi_type:ty) => {
        #[repr(C)]
        pub struct $struct_name {
            pub length: usize,
            pub values: *mut $ffi_type,
        }
        impl CastPtr for $struct_name {
            type RustType = $struct_name;
        }
        impl BoxCastPtr for $struct_name {}
        paste! {
            impl $struct_name {
                #[no_mangle]
                pub extern "C" fn [<accesskit_ $struct_name _free>](value: *mut $struct_name) {
                    let array = box_from_ptr(value);
                    unsafe { Vec::from_raw_parts(array.values, array.length, array.length) };
                    drop(array);
                }
            }
        }
        impl From<&[$rust_type]> for $struct_name {
            fn from(values: &[$rust_type]) -> Self {
                let length = values.len();
                let mut ffi_values = values.iter().map(From::from).collect::<Vec<$ffi_type>>();
                let array = Self {
                    length,
                    values: ffi_values.as_mut_ptr(),
                };
                mem::forget(ffi_values);
                array
            }
        }
    };
}

macro_rules! vec_property_methods {
    ($(($item_type:ty, $getter:ident, *mut $getter_result:ty, $setter:ident, $setter_param:ty, $pusher:ident, $clearer:ident)),+) => {
        paste! {
            $(property_getters! { $getter, *mut $getter_result }
            array_setter! { $setter, $setter_param, $item_type }
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $pusher>](builder: *mut node_builder, item: $setter_param) {
                    let builder = mut_from_ptr(builder);
                    builder.$pusher(item.into());
                }
            }
            clearer! { $clearer })*
        }
    };
    ($(($item_type:ty, $getter:ident, $getter_result:ty, $setter:ident, $setter_param:ty, $pusher:ident, $clearer:ident)),+) => {
        paste! {
            $(property_getters! { $getter, $getter_result }
            array_setter! { $setter, $setter_param, $item_type }
            impl node_builder {
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $pusher>](builder: *mut node_builder, item: $setter_param) {
                    let builder = mut_from_ptr(builder);
                    builder.$pusher(item.into());
                }
            }
            clearer! { $clearer })*
        }
    }
}

/// Call `accesskit_node_id_new` to create this struct.
///
/// If you have to manually populate this, ensure it is not filled up only with zeroes.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct node_id([u8; 16]);

impl From<NodeId> for node_id {
    fn from(id: NodeId) -> Self {
        Self(id.0.get().to_le_bytes())
    }
}

impl From<node_id> for NodeId {
    fn from(id: node_id) -> Self {
        Self(unsafe { NonZeroU128::new_unchecked(u128::from_le_bytes(id.0)) })
    }
}

slice_struct! { node_ids, NodeId, node_id }

macro_rules! node_id_vec_property_methods {
    ($(($getter:ident, $setter:ident, $pusher:ident, $clearer:ident)),+) => {
        $(vec_property_methods! {
            (NodeId, $getter, node_ids, $setter, node_id, $pusher, $clearer)
        })*
    }
}

macro_rules! node_id_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_struct! { opt_node_id, node_id }
        $(simple_property_methods! {
            $getter, opt_node_id, $setter, node_id, $clearer
        })*
    }
}

macro_rules! string_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        paste! {
            impl node {
                /// Caller must call `accesskit_string_free` with the return value.
                $(#[no_mangle]
                pub extern "C" fn [<accesskit_node_ $getter>](node: *const node) -> *mut c_char {
                    let node = ref_from_ptr(node);
                    match node.$getter() {
                        Some(value) => CString::new(value).unwrap().into_raw(),
                        None => ptr::null_mut()
                    }
                })*
            }
            $(impl node_builder {
                /// Caller must call `accesskit_string_free` with the return value.
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $getter>](builder: *const node_builder) -> *mut c_char {
                    let builder = ref_from_ptr(builder);
                    match builder.$getter() {
                        Some(value) => CString::new(value).unwrap().into_raw(),
                        None => ptr::null_mut()
                    }
                }
                /// Caller is responsible for freeing the memory pointed by `value`.
                #[no_mangle]
                pub extern "C" fn [<accesskit_node_builder_ $setter>](builder: *mut node_builder, value: *const c_char) {
                    let builder = mut_from_ptr(builder);
                    let value = unsafe { CStr::from_ptr(value) };
                    builder.$setter(value.to_string_lossy());
                }
            }
            clearer! { $clearer })*
        }
    }
}

macro_rules! f64_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_struct! { opt_double, f64 }
        $(simple_property_methods! {
            $getter, opt_double, $setter, f64, $clearer
        })*
    }
}

macro_rules! usize_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_struct! { opt_index, usize }
        $(simple_property_methods! {
            $getter, opt_index, $setter, usize, $clearer
        })*
    }
}

macro_rules! color_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_struct! { opt_color, u32 }
        $(simple_property_methods! {
            $getter, opt_color, $setter, u32, $clearer
        })*
    }
}

macro_rules! text_decoration_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_struct! { opt_text_decoration, TextDecoration }
        $(simple_property_methods! {
            $getter, opt_text_decoration, $setter, TextDecoration, $clearer
        })*
    }
}

macro_rules! opt_slice_struct {
    ($struct_name:ident, $rust_type:ty, $ffi_type:ty) => {
        #[repr(C)]
        pub struct $struct_name {
            pub has_value: bool,
            pub length: usize,
            pub values: *const $ffi_type,
        }
        impl From<Option<&[$rust_type]>> for $struct_name {
            fn from(value: Option<&[$rust_type]>) -> $struct_name {
                match value {
                    Some(value) => $struct_name {
                        has_value: true,
                        length: value.len(),
                        values: value.as_ptr() as *const $ffi_type,
                    },
                    None => $struct_name::default(),
                }
            }
        }
        impl Default for $struct_name {
            fn default() -> $struct_name {
                $struct_name {
                    has_value: false,
                    length: 0,
                    values: ptr::null(),
                }
            }
        }
    };
}

macro_rules! length_slice_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        slice_struct! { lengths, u8, u8 }
        $(simple_property_methods! {
            $getter, lengths, $setter, *const u8, $clearer
        })*
    }
}

macro_rules! coord_slice_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_slice_struct! { opt_coords, f32, f32 }
        $(simple_property_methods! {
            $getter, opt_coords, $setter, *const f32, $clearer
        })*
    }
}

macro_rules! bool_property_methods {
    ($(($getter:ident, $setter:ident, $clearer:ident)),+) => {
        opt_struct! { opt_bool, bool }
        $(simple_property_methods! {
            $getter, opt_bool, $setter, bool, $clearer
        })*
    }
}

macro_rules! unique_enum_property_methods {
    ($(($prop_type:ty, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(paste! {
            opt_struct! { [<opt_ $prop_type >], $prop_type }
            simple_property_methods! {
                $getter, [<opt_ $prop_type >], $setter, $prop_type, $clearer
            }
        })*
    }
}

property_getters! { role, Role }
impl node_builder {
    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_set_role(builder: *mut node_builder, value: Role) {
        let builder = mut_from_ptr(builder);
        builder.set_role(value);
    }
}

impl node {
    #[no_mangle]
    pub extern "C" fn accesskit_node_supports_action(node: *const node, action: Action) -> bool {
        let node = ref_from_ptr(node);
        node.supports_action(action)
    }
}

impl node_builder {
    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_supports_action(
        builder: *const node_builder,
        action: Action,
    ) -> bool {
        let builder = ref_from_ptr(builder);
        builder.supports_action(action)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_add_action(
        builder: *mut node_builder,
        action: Action,
    ) {
        let builder = mut_from_ptr(builder);
        builder.add_action(action);
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_remove_action(
        builder: *mut node_builder,
        action: Action,
    ) {
        let builder = mut_from_ptr(builder);
        builder.remove_action(action);
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_clear_actions(builder: *mut node_builder) {
        let builder = mut_from_ptr(builder);
        builder.clear_actions();
    }
}

flag_methods! {
    (is_autofill_available, set_autofill_available, clear_autofill_available),
    (is_default, set_default, clear_default),
    (is_editable, set_editable, clear_editable),
    (is_hovered, set_hovered, clear_hovered),
    (is_hidden, set_hidden, clear_hidden),
    (is_linked, set_linked, clear_linked),
    (is_multiline, set_multiline, clear_multiline),
    (is_multiselectable, set_multiselectable, clear_multiselectable),
    (is_protected, set_protected, clear_protected),
    (is_required, set_required, clear_required),
    (is_visited, set_visited, clear_visited),
    (is_busy, set_busy, clear_busy),
    (is_live_atomic, set_live_atomic, clear_live_atomic),
    (is_modal, set_modal, clear_modal),
    (is_scrollable, set_scrollable, clear_scrollable),
    (is_selected_from_focus, set_selected_from_focus, clear_selected_from_focus),
    (is_touch_pass_through, set_touch_pass_through, clear_touch_pass_through),
    (is_read_only, set_read_only, clear_read_only),
    (is_disabled, set_disabled, clear_disabled),
    (is_bold, set_bold, clear_bold),
    (is_italic, set_italic, clear_italic),
    (canvas_has_fallback, set_canvas_has_fallback, clear_canvas_has_fallback),
    (clips_children, set_clips_children, clear_clips_children),
    (is_line_breaking_object, set_is_line_breaking_object, clear_is_line_breaking_object),
    (is_page_breaking_object, set_is_page_breaking_object, clear_is_page_breaking_object),
    (is_spelling_error, set_is_spelling_error, clear_is_spelling_error),
    (is_grammar_error, set_is_grammar_error, clear_is_grammar_error),
    (is_search_match, set_is_search_match, clear_is_search_match),
    (is_suggestion, set_is_suggestion, clear_is_suggestion),
    (is_nonatomic_text_field_root, set_is_nonatomic_text_field_root, clear_is_nonatomic_text_field_root)
}

node_id_vec_property_methods! {
    (children, set_children, push_child, clear_children),
    (indirect_children, set_indirect_children, push_indirect_child, clear_indirect_children),
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
    (table_column_header, set_table_column_header, clear_table_column_header),
    (next_focus, set_next_focus, clear_next_focus),
    (previous_focus, set_previous_focus, clear_previous_focus)
}

/// Only call this function with a string that originated from AccessKit.
#[no_mangle]
pub extern "C" fn accesskit_string_free(string: *mut c_char) {
    assert!(!string.is_null());
    drop(unsafe { CString::from_raw(string) });
}

string_property_methods! {
    (name, set_name, clear_name),
    (description, set_description, clear_description),
    (value, set_value, clear_value),
    (access_key, set_access_key, clear_access_key),
    (auto_complete, set_auto_complete, clear_auto_complete),
    (checked_state_description, set_checked_state_description, clear_checked_state_description),
    (class_name, set_class_name, clear_class_name),
    (css_display, set_css_display, clear_css_display),
    (font_family, set_font_family, clear_font_family),
    (html_tag, set_html_tag, clear_html_tag),
    (inner_html, set_inner_html, clear_inner_html),
    (input_type, set_input_type, clear_input_type),
    (key_shortcuts, set_key_shortcuts, clear_key_shortcuts),
    (language, set_language, clear_language),
    (live_relevant, set_live_relevant, clear_live_relevant),
    (placeholder, set_placeholder, clear_placeholder),
    (aria_role, set_aria_role, clear_aria_role),
    (role_description, set_role_description, clear_role_description),
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
    (font_weight, set_font_weight, clear_font_weight),
    (text_indent, set_text_indent, clear_text_indent)
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
    (NameFrom, name_from, set_name_from, clear_name_from),
    (DescriptionFrom, description_from, set_description_from, clear_description_from),
    (Invalid, invalid, set_invalid, clear_invalid),
    (CheckedState, checked_state, set_checked_state, clear_checked_state),
    (Live, live, set_live, clear_live),
    (DefaultActionVerb, default_action_verb, set_default_action_verb, clear_default_action_verb),
    (TextDirection, text_direction, set_text_direction, clear_text_direction),
    (Orientation, orientation, set_orientation, clear_orientation),
    (SortDirection, sort_direction, set_sort_direction, clear_sort_direction),
    (AriaCurrent, aria_current, set_aria_current, clear_aria_current),
    (HasPopup, has_popup, set_has_popup, clear_has_popup),
    (ListStyle, list_style, set_list_style, clear_list_style),
    (TextAlign, text_align, set_text_align, clear_text_align),
    (VerticalOffset, vertical_offset, set_vertical_offset, clear_vertical_offset)
}

simple_property_methods! {
    transform, *const Affine, set_transform, Affine, clear_transform
}
opt_struct! { opt_rect, Rect }
simple_property_methods! {
    bounds, opt_rect, set_bounds, Rect, clear_bounds
}

#[repr(C)]
pub struct text_position {
    pub node: node_id,
    pub character_index: usize,
}

impl From<text_position> for TextPosition {
    fn from(position: text_position) -> Self {
        Self {
            node: position.node.into(),
            character_index: position.character_index,
        }
    }
}

impl From<TextPosition> for text_position {
    fn from(position: TextPosition) -> Self {
        Self {
            node: position.node.into(),
            character_index: position.character_index,
        }
    }
}

#[repr(C)]
pub struct text_selection {
    pub anchor: text_position,
    pub focus: text_position,
}

impl From<text_selection> for TextSelection {
    fn from(selection: text_selection) -> Self {
        Self {
            anchor: selection.anchor.into(),
            focus: selection.focus.into(),
        }
    }
}

impl From<TextSelection> for text_selection {
    fn from(selection: TextSelection) -> Self {
        Self {
            anchor: selection.anchor.into(),
            focus: selection.focus.into(),
        }
    }
}

impl From<&TextSelection> for text_selection {
    fn from(selection: &TextSelection) -> Self {
        Self {
            anchor: selection.anchor.into(),
            focus: selection.focus.into(),
        }
    }
}

opt_struct! { opt_text_selection, text_selection }
property_getters! { text_selection, opt_text_selection }
impl node_builder {
    #[no_mangle]
    pub extern "C" fn accesskit_builder_set_text_selection(
        builder: *mut node_builder,
        value: text_selection,
    ) {
        let builder = mut_from_ptr(builder);
        builder.set_text_selection(Box::new(value.into()));
    }
}
clearer! { clear_text_selection }

/// Use `accesskit_custom_action_new` to create this struct. Do not reallocate `description`.
///
/// When you get this struct, you are responsible for freeing `description`.
#[derive(Clone)]
#[repr(C)]
pub struct custom_action {
    pub id: i32,
    pub description: *mut c_char,
}

impl custom_action {
    #[no_mangle]
    pub extern "C" fn accesskit_custom_action_new(
        id: i32,
        description: *const c_char,
    ) -> custom_action {
        let description = CString::new(String::from(
            unsafe { CStr::from_ptr(description) }.to_string_lossy(),
        ))
        .unwrap();
        Self {
            id,
            description: description.into_raw(),
        }
    }
}

impl Drop for custom_action {
    fn drop(&mut self) {
        accesskit_string_free(self.description);
    }
}

impl From<custom_action> for CustomAction {
    fn from(action: custom_action) -> Self {
        Self {
            id: action.id,
            description: unsafe { CStr::from_ptr(action.description).to_string_lossy().into() },
        }
    }
}

impl From<&custom_action> for CustomAction {
    fn from(action: &custom_action) -> Self {
        Self {
            id: action.id,
            description: unsafe { CStr::from_ptr(action.description).to_string_lossy().into() },
        }
    }
}

impl From<&CustomAction> for custom_action {
    fn from(action: &CustomAction) -> Self {
        Self {
            id: action.id,
            description: CString::new(&*action.description).unwrap().into_raw(),
        }
    }
}

array_struct! { custom_actions, CustomAction, custom_action }

vec_property_methods! {
    (CustomAction, custom_actions, *mut custom_actions, set_custom_actions, custom_action, push_custom_action, clear_custom_actions)
}

impl node_id {
    #[no_mangle]
    pub extern "C" fn accesskit_node_id_new(id: u64) -> opt_node_id {
        NonZeroU128::new(id as u128).map(NodeId).into()
    }
}

impl node_builder {
    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_new(role: Role) -> *mut node_builder {
        let builder = NodeBuilder::new(role);
        BoxCastPtr::to_mut_ptr(builder)
    }

    /// Converts an `accesskit_node_builder` to an `accesskit_node`, freeing the memory in the process.
    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_build(
        builder: *mut node_builder,
        classes: *mut node_class_set,
    ) -> *mut node {
        let builder = box_from_ptr(builder);
        let classes = mut_from_ptr(classes);
        let node = builder.build(classes);
        BoxCastPtr::to_mut_ptr(node)
    }

    /// Only call this function if you have to abort the building of a node.
    ///
    /// If you called `accesskit_node_builder_build`, don't call this function.
    #[no_mangle]
    pub extern "C" fn accesskit_node_builder_free(builder: *mut node_builder) {
        drop(box_from_ptr(builder));
    }
}

#[repr(C)]
pub struct tree {
    pub root: node_id,
    pub root_scroller: opt_node_id,
}

impl tree {
    #[no_mangle]
    pub extern "C" fn accesskit_tree_new(root: node_id) -> tree {
        tree {
            root,
            root_scroller: opt_node_id::default(),
        }
    }
}

opt_struct! { opt_tree, tree }

impl From<tree> for Tree {
    fn from(tree: tree) -> Self {
        Self {
            root: tree.root.into(),
            root_scroller: tree.root_scroller.into(),
        }
    }
}

/// Use `accesskit_tree_update_new` to create this struct. Do not reallocate `ids` and `nodes`.
#[repr(C)]
pub struct tree_update {
    pub node_count: usize,
    pub ids: *mut node_id,
    pub nodes: *mut *mut node,
    pub tree: opt_tree,
    pub focus: opt_node_id,
}

impl CastPtr for tree_update {
    type RustType = tree_update;
}

impl BoxCastPtr for tree_update {}

impl tree_update {
    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_new(node_count: usize) -> *mut tree_update {
        let mut ids = Vec::with_capacity(node_count);
        let mut nodes = Vec::with_capacity(node_count);
        let update = tree_update {
            node_count,
            ids: ids.as_mut_ptr(),
            nodes: nodes.as_mut_ptr(),
            tree: opt_tree::default(),
            focus: opt_node_id::default(),
        };
        mem::forget(ids);
        mem::forget(nodes);
        BoxCastPtr::to_mut_ptr(update)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_free(update: *mut tree_update) {
        drop(TreeUpdate::from(box_from_ptr(update)));
    }
}

impl From<Box<tree_update>> for TreeUpdate {
    fn from(update: Box<tree_update>) -> Self {
        let ids = unsafe {
            Vec::from_raw_parts(
                update.ids as *mut NodeId,
                update.node_count,
                update.node_count,
            )
        };
        let nodes =
            unsafe { Vec::from_raw_parts(update.nodes, update.node_count, update.node_count) };
        Self {
            nodes: ids
                .into_iter()
                .zip(nodes.into_iter().map(box_from_ptr))
                .map(|(id, node)| (id, *node))
                .collect::<Vec<(NodeId, Node)>>(),
            tree: update.tree.into(),
            focus: update.focus.into(),
        }
    }
}

#[repr(C)]
pub enum action_data {
    CustomAction(i32),
    Value(*mut c_char),
    NumericValue(f64),
    ScrollTargetRect(Rect),
    ScrollToPoint(Point),
    SetScrollOffset(Point),
    SetTextSelection(text_selection),
}

impl Drop for action_data {
    fn drop(&mut self) {
        if let Self::Value(value) = *self {
            accesskit_string_free(value);
        }
    }
}

opt_struct! { opt_action_data, action_data }

impl From<ActionData> for action_data {
    fn from(data: ActionData) -> Self {
        match data {
            ActionData::CustomAction(action) => Self::CustomAction(action),
            ActionData::Value(value) => Self::Value(CString::new(&*value).unwrap().into_raw()),
            ActionData::NumericValue(value) => Self::NumericValue(value),
            ActionData::ScrollTargetRect(rect) => Self::ScrollTargetRect(rect),
            ActionData::ScrollToPoint(point) => Self::ScrollToPoint(point),
            ActionData::SetScrollOffset(offset) => Self::SetScrollOffset(offset),
            ActionData::SetTextSelection(selection) => Self::SetTextSelection(selection.into()),
        }
    }
}

#[repr(C)]
pub struct action_request {
    pub action: Action,
    pub target: node_id,
    pub data: opt_action_data,
}

impl From<ActionRequest> for action_request {
    fn from(request: ActionRequest) -> action_request {
        Self {
            action: request.action,
            target: request.target.into(),
            data: request.data.into(),
        }
    }
}

pub type ActionHandlerCallback =
    Option<extern "C" fn(request: *const action_request, userdata: *mut c_void)>;

struct FfiActionHandlerUserdata(*mut c_void);

unsafe impl Send for FfiActionHandlerUserdata {}
unsafe impl Sync for FfiActionHandlerUserdata {}

pub(crate) struct FfiActionHandler {
    callback: ActionHandlerCallback,
    userdata: FfiActionHandlerUserdata,
}

pub struct action_handler {
    _private: [u8; 0],
}

impl CastPtr for action_handler {
    type RustType = FfiActionHandler;
}

impl BoxCastPtr for action_handler {}

impl action_handler {
    #[no_mangle]
    pub extern "C" fn accesskit_action_handler_new(
        callback: ActionHandlerCallback,
        userdata: *mut c_void,
    ) -> *mut action_handler {
        let userdata = FfiActionHandlerUserdata(userdata);
        let handler = FfiActionHandler { callback, userdata };
        BoxCastPtr::to_mut_ptr(handler)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_action_handler_free(handler: *mut action_handler) {
        drop(box_from_ptr(handler));
    }
}

impl ActionHandler for FfiActionHandler {
    fn do_action(&self, request: ActionRequest) {
        if let Some(callback) = self.callback {
            let request = request.into();
            callback(&request, self.userdata.0);
        }
    }
}

/// This function can't return a null pointer. Ownership of the returned value will be transfered to the caller.
#[cfg(any(target_os = "windows", target_os = "macos", feature = "cbindgen"))]
pub type tree_update_factory = Option<extern "C" fn(*mut c_void) -> *mut tree_update>;
