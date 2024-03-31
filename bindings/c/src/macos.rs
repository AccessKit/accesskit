// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    box_from_ptr, mut_from_ptr, tree_update_factory, tree_update_factory_userdata,
    ActionHandlerCallback, ActivationHandlerCallback, BoxCastPtr, CastPtr, FfiActionHandler,
    FfiActivationHandler,
};
use accesskit_macos::{
    add_focus_forwarder_to_window_class, Adapter, NSPoint, QueuedEvents, SubclassingAdapter,
};
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
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
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_adapter_new(
        view: *mut c_void,
        is_view_focused: bool,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut macos_adapter {
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = Adapter::new(view, is_view_focused, action_handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_free(adapter: *mut macos_adapter) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_update_if_active(
        adapter: *mut macos_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut macos_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_update_view_focus_state(
        adapter: *mut macos_adapter,
        is_focused: bool,
    ) -> *mut macos_queued_events {
        let adapter = mut_from_ptr(adapter);
        let events = adapter.update_view_focus_state(is_focused);
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// Returns a pointer to an `NSArray`. Ownership of the pointer is not transferred.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_view_children(
        adapter: *mut macos_adapter,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
    ) -> *mut c_void {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        adapter.view_children(&mut activation_handler) as *mut _
    }

    /// Returns a pointer to an `NSObject`. Ownership of the pointer is not transferred.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_focus(
        adapter: *mut macos_adapter,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
    ) -> *mut c_void {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        adapter.focus(&mut activation_handler) as *mut _
    }

    /// Returns a pointer to an `NSObject`. Ownership of the pointer is not transferred.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_adapter_hit_test(
        adapter: *mut macos_adapter,
        x: f64,
        y: f64,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
    ) -> *mut c_void {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        adapter.hit_test(NSPoint::new(x, y), &mut activation_handler) as *mut _
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
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_macos_subclassing_adapter_new(
        view: *mut c_void,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut macos_subclassing_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = SubclassingAdapter::new(view, activation_handler, action_handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

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
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut macos_subclassing_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = SubclassingAdapter::for_window(window, activation_handler, action_handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_free(
        adapter: *mut macos_subclassing_adapter,
    ) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_update_if_active(
        adapter: *mut macos_subclassing_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut macos_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// Update the tree state based on whether the window is focused.
    ///
    /// You must call `accesskit_macos_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_macos_subclassing_adapter_update_view_focus_state(
        adapter: *mut macos_subclassing_adapter,
        is_focused: bool,
    ) -> *mut macos_queued_events {
        let adapter = mut_from_ptr(adapter);
        let events = adapter.update_view_focus_state(is_focused);
        BoxCastPtr::to_nullable_mut_ptr(events)
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
