// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    action_handler, tree_update, tree_update_factory, try_ref_from_ptr, BoxCastPtr, CastPtr,
};
use accesskit_macos::{Adapter, NSObject, NSPoint, QueuedEvents, SubclassingAdapter};
use std::{os::raw::c_void, ptr};

#[cfg(not(feature = "cbindgen"))]
pub type NSArray = c_void;

pub struct macos_queued_events {
    _private: [u8; 0],
}

impl CastPtr for macos_queued_events {
    type RustType = QueuedEvents;
}

impl BoxCastPtr for macos_queued_events {}

impl macos_queued_events {
    /// Memory is also freed when calling this function.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_queued_events_raise(events: *mut macos_queued_events) {
        if let Some(events) = macos_queued_events::to_box(events) {
            events.raise();
        }
    }
}

pub struct macos_adapter {
    _private: [u8; 0],
}

impl CastPtr for macos_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for macos_adapter {}

impl macos_adapter {
    /// This function takes ownership of `initial_state` and `handler`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_adapter_new(
        view: *mut c_void,
        initial_state: *mut tree_update,
        handler: *mut action_handler,
    ) -> *mut macos_adapter {
        let initial_state = match tree_update::to_box(initial_state) {
            Some(initial_state) => initial_state,
            None => return ptr::null_mut(),
        };
        let handler = match action_handler::to_box(handler) {
            Some(handler) => handler,
            None => return ptr::null_mut(),
        };
        let adapter = Adapter::new(view, initial_state.into(), handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_free(adapter: *mut macos_adapter) {
        macos_adapter::to_box(adapter);
    }

    /// This function takes ownership of `update`.
    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null in case of error.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_update(
        adapter: *const macos_adapter,
        update: *mut tree_update,
    ) -> *mut macos_queued_events {
        let adapter = try_ref_from_ptr!(adapter);
        if let Some(update) = tree_update::to_box(update) {
            let events = adapter.update(update.into());
            BoxCastPtr::to_mut_ptr(events)
        } else {
            ptr::null_mut()
        }
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_view_children(
        adapter: *const macos_adapter,
    ) -> *mut NSArray {
        let adapter = try_ref_from_ptr!(adapter);
        adapter.view_children() as *mut _
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_focus(
        adapter: *const macos_adapter,
    ) -> *mut NSObject {
        let adapter = try_ref_from_ptr!(adapter);
        adapter.focus()
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_hit_test(
        adapter: *const macos_adapter,
        point: NSPoint,
    ) -> *mut NSObject {
        let adapter = try_ref_from_ptr!(adapter);
        adapter.hit_test(point)
    }
}

pub struct macos_subclassing_adapter {
    _private: [u8; 0],
}

impl CastPtr for macos_subclassing_adapter {
    type RustType = SubclassingAdapter;
}

impl BoxCastPtr for macos_subclassing_adapter {}

impl macos_subclassing_adapter {
    /// This function takes ownership of `handler`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_subclassing_adapter_new(
        view: *mut c_void,
        source: tree_update_factory,
        source_userdata: *mut c_void,
        handler: *mut action_handler,
    ) -> *mut macos_subclassing_adapter {
        let source = match source {
            Some(source) => source,
            None => return ptr::null_mut(),
        };
        let handler = match action_handler::to_box(handler) {
            Some(handler) => handler,
            None => return ptr::null_mut(),
        };
        let adapter = SubclassingAdapter::new(
            view,
            move || tree_update::to_box(source(source_userdata)).unwrap().into(),
            handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_free(
        adapter: *mut macos_subclassing_adapter,
    ) {
        macos_subclassing_adapter::to_box(adapter);
    }

    /// This function takes ownership of `update`.
    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null in case of error.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_update(
        adapter: *const macos_subclassing_adapter,
        update: *mut tree_update,
    ) -> *mut macos_queued_events {
        let adapter = try_ref_from_ptr!(adapter);
        if let Some(update) = tree_update::to_box(update) {
            let events = adapter.update(update.into());
            BoxCastPtr::to_mut_ptr(events)
        } else {
            ptr::null_mut()
        }
    }

    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null in case of error or if the window is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_update_if_active(
        adapter: *const macos_subclassing_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut macos_queued_events {
        let update_factory = match update_factory {
            Some(update_factory) => update_factory,
            None => return ptr::null_mut(),
        };
        let adapter = try_ref_from_ptr!(adapter);
        let events = adapter.update_if_active(|| {
            tree_update::to_box(update_factory(update_factory_userdata))
                .unwrap()
                .into()
        });
        match events {
            Some(events) => BoxCastPtr::to_mut_ptr(events),
            None => ptr::null_mut(),
        }
    }
}
