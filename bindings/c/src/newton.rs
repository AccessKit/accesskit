// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    box_from_ptr, mut_from_ptr, tree_update_factory, tree_update_factory_userdata,
    ActionHandlerCallback, ActivationHandlerCallback, BoxCastPtr, CastPtr,
    DeactivationHandlerCallback, FfiActionHandler, FfiActivationHandler, FfiDeactivationHandler,
};
use accesskit_newton::Adapter;
use std::os::raw::c_void;

pub struct newton_adapter {
    _private: [u8; 0],
}

impl CastPtr for newton_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for newton_adapter {}

impl newton_adapter {
    /// All of the handlers will always be called from another thread.
    ///
    /// # Safety
    ///
    /// `display` must be a valid `wl_display *` pointer, and
    /// `surface` must be a valid `wl_surface *` pointer. Both must remain
    /// valid for as long as the adapter is alive.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_newton_adapter_new(
        display: *mut c_void,
        surface: *mut c_void,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
        deactivation_handler: DeactivationHandlerCallback,
        deactivation_handler_userdata: *mut c_void,
    ) -> *mut newton_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let deactivation_handler =
            FfiDeactivationHandler::new(deactivation_handler, deactivation_handler_userdata);
        let adapter = Adapter::new(
            display,
            surface,
            activation_handler,
            action_handler,
            deactivation_handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_newton_adapter_free(adapter: *mut newton_adapter) {
        drop(box_from_ptr(adapter));
    }

    #[no_mangle]
    pub extern "C" fn accesskit_newton_adapter_update_if_active(
        adapter: *mut newton_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
    }
}
