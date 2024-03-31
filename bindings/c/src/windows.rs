// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    box_from_ptr, mut_from_ptr, opt_struct, tree_update_factory, tree_update_factory_userdata,
    ActionHandlerCallback, ActivationHandlerCallback, BoxCastPtr, CastPtr, FfiActionHandler,
    FfiActivationHandler,
};
use accesskit_windows::*;
use std::os::raw::c_void;

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
    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_new(
        hwnd: HWND,
        is_window_focused: bool,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut windows_adapter {
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = Adapter::new(hwnd, is_window_focused, action_handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_free(adapter: *mut windows_adapter) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_update_if_active(
        adapter: *mut windows_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut windows_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_update_window_focus_state(
        adapter: *mut windows_adapter,
        is_focused: bool,
    ) -> *mut windows_queued_events {
        let adapter = mut_from_ptr(adapter);
        let events = adapter.update_window_focus_state(is_focused);
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_windows_adapter_handle_wm_getobject(
        adapter: *mut windows_adapter,
        wparam: WPARAM,
        lparam: LPARAM,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
    ) -> opt_lresult {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let lresult = adapter.handle_wm_getobject(wparam, lparam, &mut activation_handler);
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
    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_new(
        hwnd: HWND,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut windows_subclassing_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = SubclassingAdapter::new(hwnd, activation_handler, action_handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_free(
        adapter: *mut windows_subclassing_adapter,
    ) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_windows_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_windows_subclassing_adapter_update_if_active(
        adapter: *mut windows_subclassing_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut windows_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }
}
