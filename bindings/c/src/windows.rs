// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    action_handler, box_from_ptr, opt_struct, ref_from_ptr, tree_update, tree_update_factory,
    tree_update_factory_userdata, BoxCastPtr, CastPtr,
};
use accesskit_windows::*;
use std::{os::raw::c_void, ptr};

pub struct windows_uia_init_marker {
    _private: [u8; 0],
}

impl CastPtr for windows_uia_init_marker {
    type RustType = UiaInitMarker;
}

impl BoxCastPtr for windows_uia_init_marker {}

impl windows_uia_init_marker {
    #[no_mangle]
    pub extern "C" fn accesskit_windows_uia_init_marker_new() -> *mut windows_uia_init_marker {
        let marker = UiaInitMarker::new();
        BoxCastPtr::to_mut_ptr(marker)
    }

    /// You don't need to call this if you use `accesskit_windows_adapter_new`.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_uia_init_marker_free(marker: *mut windows_uia_init_marker) {
        drop(box_from_ptr(marker));
    }
}

pub struct windows_queued_events {
    _private: [u8; 0],
}

impl CastPtr for windows_queued_events {
    type RustType = QueuedEvents;
}

impl BoxCastPtr for windows_queued_events {}

impl windows_queued_events {
    /// Memory is also freed when calling this function.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_queued_events_raise(events: *mut windows_queued_events) {
        let events = box_from_ptr(events);
        events.raise();
    }
}

opt_struct! { opt_lresult, LRESULT }

pub struct windows_adapter {
    _private: [u8; 0],
}

impl CastPtr for windows_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for windows_adapter {}

impl windows_adapter {
    /// This function takes ownership of all pointers passed to it.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_new(
        hwnd: HWND,
        initial_state: *mut tree_update,
        is_window_focused: bool,
        handler: *mut action_handler,
        uia_init_marker: *mut windows_uia_init_marker,
    ) -> *mut windows_adapter {
        let initial_state = box_from_ptr(initial_state);
        let handler = box_from_ptr(handler);
        let uia_init_marker = *box_from_ptr(uia_init_marker);
        let adapter = Adapter::new(
            hwnd,
            *initial_state,
            is_window_focused,
            handler,
            uia_init_marker,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_free(adapter: *mut windows_adapter) {
        drop(box_from_ptr(adapter));
    }

    /// This function takes ownership of `update`.
    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_update(
        adapter: *const windows_adapter,
        update: *mut tree_update,
    ) -> *mut windows_queued_events {
        let adapter = ref_from_ptr(adapter);
        let update = box_from_ptr(update);
        let events = adapter.update(*update);
        BoxCastPtr::to_mut_ptr(events)
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_update_window_focus_state(
        adapter: *const windows_adapter,
        is_focused: bool,
    ) -> *mut windows_queued_events {
        let adapter = ref_from_ptr(adapter);
        let events = adapter.update_window_focus_state(is_focused);
        BoxCastPtr::to_mut_ptr(events)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_handle_wm_getobject(
        adapter: *mut windows_adapter,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> opt_lresult {
        let adapter = ref_from_ptr(adapter);
        let lresult = adapter.handle_wm_getobject(wparam, lparam);
        opt_lresult::from(lresult)
    }
}

pub struct windows_subclassing_adapter {
    _private: [u8; 0],
}

impl CastPtr for windows_subclassing_adapter {
    type RustType = SubclassingAdapter;
}

impl BoxCastPtr for windows_subclassing_adapter {}

impl windows_subclassing_adapter {
    /// This function takes ownership of `handler`.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_new(
        hwnd: HWND,
        source: tree_update_factory,
        source_userdata: *mut c_void,
        handler: *mut action_handler,
    ) -> *mut windows_subclassing_adapter {
        let source = source.unwrap();
        let source_userdata = tree_update_factory_userdata(source_userdata);
        let handler = box_from_ptr(handler);
        let adapter = SubclassingAdapter::new(
            hwnd,
            move || *box_from_ptr(source(source_userdata)),
            handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_free(
        adapter: *mut windows_subclassing_adapter,
    ) {
        drop(box_from_ptr(adapter));
    }

    /// This function takes ownership of `update`.
    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_update(
        adapter: *const windows_subclassing_adapter,
        update: *mut tree_update,
    ) -> *mut windows_queued_events {
        let adapter = ref_from_ptr(adapter);
        let update = box_from_ptr(update);
        let events = adapter.update(*update);
        BoxCastPtr::to_mut_ptr(events)
    }

    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_update_if_active(
        adapter: *const windows_subclassing_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut windows_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = ref_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        match events {
            Some(events) => BoxCastPtr::to_mut_ptr(events),
            None => ptr::null_mut(),
        }
    }
}
