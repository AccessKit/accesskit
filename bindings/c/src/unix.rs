// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    action_handler, box_from_ptr, ref_from_ptr, tree_update_factory, tree_update_factory_userdata,
    BoxCastPtr, CastPtr,
};
use accesskit::Rect;
use accesskit_unix::Adapter;
use std::os::raw::c_void;

pub struct unix_adapter {
    _private: [u8; 0],
}

impl CastPtr for unix_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for unix_adapter {}

impl unix_adapter {
    /// This function will take ownership of the pointer returned by `source`, which can't be null.
    ///
    /// `source` can be called from any thread.
    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_new(
        source: tree_update_factory,
        source_userdata: *mut c_void,
        handler: *mut action_handler,
    ) -> *mut unix_adapter {
        let source = source.unwrap();
        let source_userdata = tree_update_factory_userdata(source_userdata);
        let handler = box_from_ptr(handler);
        let adapter = Adapter::new(move || *box_from_ptr(source(source_userdata)), handler);
        BoxCastPtr::to_mut_ptr(adapter)
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
    pub extern "C" fn accesskit_unix_adapter_update_if_active(
        adapter: *const unix_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = ref_from_ptr(adapter);
        adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
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
