// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    action_handler, box_from_ptr, ref_from_ptr, tree_update, tree_update_factory, BoxCastPtr,
    CastPtr,
};
use accesskit::Rect;
use accesskit_unix::Adapter;
use std::{os::raw::c_void, ptr};

pub struct unix_adapter {
    _private: [u8; 0],
}

impl CastPtr for unix_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for unix_adapter {}

impl unix_adapter {
    /// This function will take ownership of the pointer returned by `initial_state`, which can't be null.
    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_new(
        initial_state: tree_update_factory,
        initial_state_userdata: *mut c_void,
        is_window_focused: bool,
        handler: *mut action_handler,
    ) -> *mut unix_adapter {
        let initial_state = initial_state.unwrap();
        let handler = box_from_ptr(handler);
        let adapter = Adapter::new(
            move || *box_from_ptr(initial_state(initial_state_userdata)),
            is_window_focused,
            handler,
        );
        adapter.map_or_else(ptr::null_mut, BoxCastPtr::to_mut_ptr)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_free(adapter: *mut unix_adapter) {
        drop(box_from_ptr(adapter));
    }

    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_set_root_window_bounds(
        adapter: *const unix_adapter,
        outer: Rect,
        inner: Rect,
    ) {
        let adapter = ref_from_ptr(adapter);
        adapter.set_root_window_bounds(outer, inner);
    }

    /// This function takes ownership of `update`.
    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_update(
        adapter: *const unix_adapter,
        update: *mut tree_update,
    ) {
        let adapter = ref_from_ptr(adapter);
        let update = box_from_ptr(update);
        adapter.update(*update);
    }

    /// Update the tree state based on whether the window is focused.
    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_update_window_focus_state(
        adapter: *const unix_adapter,
        is_focused: bool,
    ) {
        let adapter = ref_from_ptr(adapter);
        adapter.update_window_focus_state(is_focused);
    }
}
