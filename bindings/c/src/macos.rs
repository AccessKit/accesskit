// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    action_handler, box_from_ptr, ref_from_ptr, tree_update, tree_update_factory, BoxCastPtr,
    CastPtr,
};
use accesskit_macos::{
    add_focus_forwarder_to_window_class, Adapter, NSPoint, QueuedEvents, SubclassingAdapter,
};
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
    ptr,
};

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
        let events = box_from_ptr(events);
        events.raise();
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
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_adapter_new(
        view: *mut c_void,
        initial_state: *mut tree_update,
        handler: *mut action_handler,
    ) -> *mut macos_adapter {
        let initial_state = box_from_ptr(initial_state);
        let handler = box_from_ptr(handler);
        let adapter = Adapter::new(view, *initial_state, handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_free(adapter: *mut macos_adapter) {
        drop(box_from_ptr(adapter));
    }

    /// This function takes ownership of `update`.
    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null in case of error.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_update(
        adapter: *const macos_adapter,
        update: *mut tree_update,
    ) -> *mut macos_queued_events {
        let adapter = ref_from_ptr(adapter);
        let update = box_from_ptr(update);
        let events = adapter.update(*update);
        BoxCastPtr::to_mut_ptr(events)
    }

    /// Returns a pointer to an `NSArray`. Ownership of the pointer is not transfered.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_view_children(
        adapter: *const macos_adapter,
    ) -> *mut c_void {
        let adapter = ref_from_ptr(adapter);
        adapter.view_children() as *mut _
    }

    /// Returns a pointer to an `NSObject`. Ownership of the pointer is not transfered.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_focus(adapter: *const macos_adapter) -> *mut c_void {
        let adapter = ref_from_ptr(adapter);
        adapter.focus() as *mut _
    }

    /// Returns a pointer to an `NSObject`. Ownership of the pointer is not transfered.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_hit_test(
        adapter: *const macos_adapter,
        x: f64,
        y: f64,
    ) -> *mut c_void {
        let adapter = ref_from_ptr(adapter);
        adapter.hit_test(NSPoint::new(x, y)) as *mut _
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
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_subclassing_adapter_new(
        view: *mut c_void,
        source: tree_update_factory,
        source_userdata: *mut c_void,
        handler: *mut action_handler,
    ) -> *mut macos_subclassing_adapter {
        let source = source.unwrap();
        let handler = box_from_ptr(handler);
        let adapter = SubclassingAdapter::new(
            view,
            move || *box_from_ptr(source(source_userdata)),
            handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    /// This function takes ownership of `handler`.
    ///
    /// # Safety
    ///
    /// `window` must be a valid, unreleased pointer to an `NSWindow`.
    ///
    /// # Panics
    ///
    /// This function panics if the specified window doesn't currently have
    /// a content view.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_subclassing_adapter_for_window(
        window: *mut c_void,
        source: tree_update_factory,
        source_userdata: *mut c_void,
        handler: *mut action_handler,
    ) -> *mut macos_subclassing_adapter {
        let source = source.unwrap();
        let handler = box_from_ptr(handler);
        let adapter = SubclassingAdapter::for_window(
            window,
            move || *box_from_ptr(source(source_userdata)),
            handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_free(
        adapter: *mut macos_subclassing_adapter,
    ) {
        drop(box_from_ptr(adapter));
    }

    /// This function takes ownership of `update`.
    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null in case of error.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_update(
        adapter: *const macos_subclassing_adapter,
        update: *mut tree_update,
    ) -> *mut macos_queued_events {
        let adapter = ref_from_ptr(adapter);
        let update = box_from_ptr(update);
        let events = adapter.update(*update);
        BoxCastPtr::to_mut_ptr(events)
    }

    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null in case of error or if the window is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_update_if_active(
        adapter: *const macos_subclassing_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut macos_queued_events {
        let update_factory = update_factory.unwrap();
        let adapter = ref_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        match events {
            Some(events) => BoxCastPtr::to_mut_ptr(events),
            None => ptr::null_mut(),
        }
    }
}

/// Modifies the specified class, which must be a subclass of `NSWindow`,
/// to include an `accessibilityFocusedUIElement` method that calls
/// the corresponding method on the window's content view. This is needed
/// for windowing libraries such as SDL that place the keyboard focus
/// directly on the window rather than the content view.
///
/// # Safety
///
/// This function is declared unsafe because the caller must ensure that the
/// code for this library is never unloaded from the application process,
/// since it's not possible to reverse this operation. It's safest
/// if this library is statically linked into the application's main executable.
/// Also, this function assumes that the specified class is a subclass
/// of `NSWindow`.
#[no_mangle]
pub unsafe extern "C" fn accesskit_macos_add_focus_forwarder_to_window_class(
    class_name: *const c_char,
) {
    let class_name = unsafe { CStr::from_ptr(class_name).to_string_lossy() };
    add_focus_forwarder_to_window_class(&class_name);
}
