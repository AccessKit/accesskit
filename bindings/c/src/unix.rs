// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    action_handler, ffi_panic_boundary, tree_update, try_ref_from_ptr, BoxCastPtr, CastPtr,
};
use accesskit::Rect;
use accesskit_unix::Adapter;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
    ptr,
};

pub struct unix_adapter {
    _private: [u8; 0],
}

impl CastPtr for unix_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for unix_adapter {}

impl unix_adapter {
    /// Caller is responsible for freeing `app_name`, `toolkit_name` and `toolkit_version`.
    /// This function will take ownership of the pointer returned by `initial_state`, which can't be null.
    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_new(
        app_name: *const c_char,
        toolkit_name: *const c_char,
        toolkit_version: *const c_char,
        initial_state: Option<extern "C" fn(*mut c_void) -> *mut tree_update>,
        initial_state_userdata: *mut c_void,
        handler: *mut action_handler,
    ) -> *mut unix_adapter {
        let app_name = unsafe { CStr::from_ptr(app_name).to_string_lossy().into() };
        let toolkit_name = unsafe { CStr::from_ptr(toolkit_name).to_string_lossy().into() };
        let toolkit_version = unsafe { CStr::from_ptr(toolkit_version).to_string_lossy().into() };
        let initial_state = match initial_state {
            Some(initial_state) => initial_state,
            None => return ptr::null_mut(),
        };
        let handler = match action_handler::to_box(handler) {
            Some(handler) => handler,
            None => return ptr::null_mut(),
        };
        ffi_panic_boundary! {
            let adapter = Adapter::new(
                app_name,
                toolkit_name,
                toolkit_version,
                move || {
                    tree_update::to_box(initial_state(initial_state_userdata))
                        .unwrap()
                        .into()
                },
                handler,
            );
            adapter.map_or_else(ptr::null_mut, BoxCastPtr::to_mut_ptr)
        }
    }

    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_free(adapter: *mut unix_adapter) {
        unix_adapter::to_box(adapter);
    }

    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_set_root_window_bounds(
        adapter: *const unix_adapter,
        outer: Rect,
        inner: Rect,
    ) {
        let adapter = try_ref_from_ptr!(adapter);
        adapter.set_root_window_bounds(outer, inner);
    }

    /// This function takes ownership of `update`.
    #[no_mangle]
    pub extern "C" fn accesskit_unix_adapter_update(
        adapter: *const unix_adapter,
        update: *mut tree_update,
    ) -> bool {
        ffi_panic_boundary! {
            let adapter = try_ref_from_ptr!(adapter);
            if let Some(update) = tree_update::to_box(update) {
                adapter.update(update.into());
                true
            } else {
                false
            }
        }
    }
}
