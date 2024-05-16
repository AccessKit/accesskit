// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use sctk::reexports::{
    calloop::channel::{self, Sender},
    client::{
        backend::{Backend, ObjectId},
        protocol::{__interfaces::WL_SURFACE_INTERFACE, wl_surface::WlSurface},
        Connection, Proxy,
    },
};
use std::{
    collections::HashSet,
    ffi::c_void,
    os::unix::io::AsFd,
    sync::{Arc, Mutex},
};
use wayland_protocols::wp::a11y::v1::client::wp_a11y_updates_v1::WpA11yUpdatesV1;

mod state;
mod worker;

pub struct Adapter {
    request_tx: Sender<worker::Command>,
    update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
    worker_thread: Option<std::thread::JoinHandle<()>>,
}

impl Adapter {
    /// Creates an AccessKit adapter for the specified Wayland display
    /// and surface. The adapter will run on a worker thread with its own
    /// libwayland event queue. All of the handlers will always be called
    /// on that worker thread.
    ///
    /// # Safety
    ///
    /// `display` must be a valid `*mut wl_display` pointer, and
    /// `surface` must be a valid `*mut wl_surface` pointer. Both must remain
    /// valid for as long as the adapter is alive.
    pub unsafe fn new(
        display: *mut c_void,
        surface: *mut c_void,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        deactivation_handler: impl 'static + DeactivationHandler + Send,
    ) -> Self {
        let backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let connection = Connection::from_backend(backend);
        let surface_id =
            unsafe { ObjectId::from_ptr(&WL_SURFACE_INTERFACE, surface.cast()) }.unwrap();
        let surface = WlSurface::from_id(&connection, surface_id).unwrap();
        let (request_tx, request_rx) = channel::channel();
        let update_receivers = Arc::new(Mutex::new(HashSet::new()));
        let worker_thread = worker::spawn(
            connection,
            surface,
            activation_handler,
            action_handler,
            deactivation_handler,
            request_rx,
            Arc::clone(&update_receivers),
        );

        Self {
            request_tx,
            update_receivers,
            worker_thread,
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        use rustix::pipe::{pipe_with, PipeFlags};

        let receivers = self.update_receivers.lock().unwrap();
        if receivers.is_empty() {
            return;
        }
        let update = update_factory();
        let serialized = Arc::new(serde_json::to_vec(&update).unwrap());
        for receiver in receivers.iter() {
            let (read_fd, write_fd) = pipe_with(PipeFlags::CLOEXEC).unwrap();
            self.request_tx
                .send(worker::Command::WriteUpdate(
                    write_fd,
                    Arc::clone(&serialized),
                ))
                .unwrap();
            receiver.send(read_fd.as_fd());
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        let _ = self.request_tx.send(worker::Command::Exit);
        if let Some(worker_thread) = self.worker_thread.take() {
            let _ = worker_thread.join();
        }
    }
}
