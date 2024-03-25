// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::Tree;
use once_cell::unsync::Lazy;
use sctk::{
    data_device_manager::{ReadPipe, WritePipe},
    delegate_registry,
    reexports::{
        calloop::{LoopHandle, PostAction},
        client::{
            event_created_child, globals::GlobalList, protocol::wl_surface::WlSurface, Connection,
            Dispatch, QueueHandle,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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

type LazyTree = Lazy<Tree, Box<dyn FnOnce() -> Tree>>;

pub(crate) struct State {
    registry_state: RegistryState,
    loop_handle: LoopHandle<'static, Self>,
    pub(crate) exit: bool,
    surface: WlSurface,
    a11y_surface: WpA11ySurfaceV1,
    tree: LazyTree,
    action_handler: Box<dyn ActionHandler + Send>,
    update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
}

impl State {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        globals: &GlobalList,
        qh: &QueueHandle<Self>,
        loop_handle: LoopHandle<'static, Self>,
        surface: WlSurface,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: Box<dyn ActionHandler + Send>,
        update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
        a11y_manager: WpA11yManagerV1,
    ) -> Self {
        let a11y_surface = a11y_manager.get_a11y_surface(&surface, qh, ());
        a11y_manager.destroy();
        let tree: LazyTree = Lazy::new(Box::new(move || Tree::new(source(), true)));

        Self {
            registry_state: RegistryState::new(globals),
            loop_handle,
            exit: false,
            surface,
            a11y_surface,
            tree,
            action_handler,
            update_receivers,
        }
    }

    pub(crate) fn update_tree(&mut self, update: TreeUpdate) {
        if let Some(tree) = Lazy::get_mut(&mut self.tree) {
            tree.update(update);
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
        let tree = Lazy::force(&self.tree);
        let update = tree.state().serialize();
        let serialized = Arc::new(serde_json::to_vec(&update).unwrap());
        let (read_fd, write_fd) = pipe_with(PipeFlags::CLOEXEC).unwrap();
        self.write_update(write_fd, serialized);
        receiver.send(read_fd.as_fd());
        self.surface.commit();
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
                            let request = match serde_json::from_slice(&content) {
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
}

impl ProvidesRegistryState for State {
    registry_handlers![];

    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

delegate_registry!(State);

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
            state.update_receivers.lock().unwrap().remove(receiver);
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
