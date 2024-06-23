// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler};
use sctk::{
    data_device_manager::{ReadPipe, WritePipe},
    reexports::{
        calloop::{LoopHandle, PostAction},
        client::{
            event_created_child,
            globals::GlobalListContents,
            protocol::{
                wl_registry::{Event as RegistryEvent, WlRegistry},
                wl_surface::WlSurface,
            },
            Connection, Dispatch, QueueHandle,
        },
    },
};
use std::{
    collections::HashSet,
    io::{ErrorKind, Read, Write},
    os::unix::io::{AsFd, AsRawFd, OwnedFd, RawFd},
    sync::{Arc, Mutex},
};
use wayland_protocols::wp::a11y::v1::client::{
    wp_a11y_manager_v1::{Event as ManagerEvent, WpA11yManagerV1},
    wp_a11y_surface_v1::{Event as SurfaceEvent, WpA11ySurfaceV1, EVT_UPDATES_WANTED_OPCODE},
    wp_a11y_updates_v1::{Event as UpdatesEvent, WpA11yUpdatesV1},
};

pub(crate) struct State {
    loop_handle: LoopHandle<'static, Self>,
    pub(crate) exit: bool,
    surface: WlSurface,
    a11y_surface: WpA11ySurfaceV1,
    activation_handler: Box<dyn ActivationHandler>,
    action_handler: Box<dyn ActionHandler>,
    deactivation_handler: Box<dyn DeactivationHandler>,
    update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
}

impl State {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        qh: &QueueHandle<Self>,
        loop_handle: LoopHandle<'static, Self>,
        surface: WlSurface,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
        deactivation_handler: impl 'static + DeactivationHandler,
        update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
        a11y_manager: WpA11yManagerV1,
    ) -> Self {
        let a11y_surface = a11y_manager.get_a11y_surface(&surface, qh, ());
        a11y_manager.destroy();

        Self {
            loop_handle,
            exit: false,
            surface,
            a11y_surface,
            activation_handler: Box::new(activation_handler),
            action_handler: Box::new(action_handler),
            deactivation_handler: Box::new(deactivation_handler),
            update_receivers,
        }
    }

    pub(crate) fn write_update(&self, fd: OwnedFd, serialized: Arc<Vec<u8>>) {
        let write_pipe = WritePipe::from(fd);
        unsafe {
            if set_non_blocking(write_pipe.as_raw_fd()).is_err() {
                return;
            }
        }
        let mut written = 0;
        let _ = self
            .loop_handle
            .insert_source(write_pipe, move |_, file, _| {
                let file = unsafe { file.get_mut() };
                loop {
                    match file.write(&serialized[written..]) {
                        Ok(n) if written + n == serialized.len() => {
                            written += n;
                            break PostAction::Remove;
                        }
                        Ok(n) => written += n,
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            break PostAction::Continue
                        }
                        Err(_) => break PostAction::Remove,
                    }
                }
            });
    }

    fn handle_new_update_receiver(&mut self, receiver: WpA11yUpdatesV1) {
        use rustix::pipe::{pipe_with, PipeFlags};

        let mut receivers = self.update_receivers.lock().unwrap();
        if let Some(update) = self.activation_handler.request_initial_tree() {
            let serialized = crate::serialize_tree_update(&update);
            let (read_fd, write_fd) = pipe_with(PipeFlags::CLOEXEC).unwrap();
            self.write_update(write_fd, serialized);
            receiver.send(read_fd.as_fd());
            self.surface.commit();
        }
        receivers.insert(receiver);
    }

    fn handle_action_request(&mut self, fd: OwnedFd) {
        let read_pipe = ReadPipe::from(fd);
        unsafe {
            if set_non_blocking(read_pipe.as_raw_fd()).is_err() {
                return;
            }
        }
        let mut reader_buffer = [0; 4096];
        let mut content = Vec::new();
        let _ = self
            .loop_handle
            .insert_source(read_pipe, move |_, file, state| {
                let file = unsafe { file.get_mut() };
                loop {
                    match file.read(&mut reader_buffer) {
                        Ok(0) => {
                            let request = match postcard::from_bytes(&content) {
                                Ok(request) => request,
                                Err(_) => {
                                    break PostAction::Remove;
                                }
                            };
                            state.action_handler.do_action(request);
                            break PostAction::Remove;
                        }
                        Ok(n) => content.extend_from_slice(&reader_buffer[..n]),
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            break PostAction::Continue
                        }
                        Err(_) => {
                            break PostAction::Remove;
                        }
                    };
                }
            });
    }

    fn handle_deactivated_update_receiver(&mut self, receiver: &WpA11yUpdatesV1) {
        let mut receivers = self.update_receivers.lock().unwrap();
        receivers.remove(receiver);
        if receivers.is_empty() {
            drop(receivers);
            self.deactivation_handler.deactivate_accessibility();
        }
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: RegistryEvent,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        // We don't need to handle registry events; we already got our global.
    }
}

impl Dispatch<WpA11yManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &WpA11yManagerV1,
        _: ManagerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        // No events for this interface.
    }
}

impl Dispatch<WpA11ySurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &WpA11ySurfaceV1,
        event: SurfaceEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        match event {
            SurfaceEvent::UpdatesWanted { receiver } => {
                state.handle_new_update_receiver(receiver);
            }
            SurfaceEvent::ActionRequest { fd } => {
                state.handle_action_request(fd);
            }
            _ => (),
        }
    }

    event_created_child!(State, WpA11ySurfaceV1, [
        EVT_UPDATES_WANTED_OPCODE => (WpA11yUpdatesV1, ())
    ]);
}

impl Dispatch<WpA11yUpdatesV1, ()> for State {
    fn event(
        state: &mut Self,
        receiver: &WpA11yUpdatesV1,
        event: UpdatesEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        if let UpdatesEvent::Unwanted = event {
            state.handle_deactivated_update_receiver(receiver);
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.a11y_surface.destroy();
    }
}

unsafe fn set_non_blocking(raw_fd: RawFd) -> std::io::Result<()> {
    let flags = libc::fcntl(raw_fd, libc::F_GETFL);

    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }

    let result = libc::fcntl(raw_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    if result < 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}
