// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler};
use sctk::reexports::{
    calloop::{
        channel::{self, Channel},
        EventLoop,
    },
    calloop_wayland_source::WaylandSource,
    client::{globals::registry_queue_init, protocol::wl_surface::WlSurface, Connection},
};
use std::{
    collections::HashSet,
    os::unix::io::OwnedFd,
    sync::{Arc, Mutex},
};
use wayland_protocols::wp::a11y::v1::client::wp_a11y_updates_v1::WpA11yUpdatesV1;

use crate::state::State;

pub(crate) fn spawn(
    connection: Connection,
    surface: WlSurface,
    activation_handler: impl 'static + ActivationHandler + Send,
    action_handler: impl 'static + ActionHandler + Send,
    deactivation_handler: impl 'static + DeactivationHandler + Send,
    request_rx: Channel<Command>,
    update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
) -> Option<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("accesskit-adapter".into())
        .spawn(move || {
            worker_impl(
                connection,
                surface,
                activation_handler,
                action_handler,
                deactivation_handler,
                request_rx,
                update_receivers,
            );
        })
        .ok()
}

pub(crate) enum Command {
    WriteUpdate(OwnedFd, Arc<Vec<u8>>),
    Exit,
}

fn worker_impl(
    connection: Connection,
    surface: WlSurface,
    activation_handler: impl 'static + ActivationHandler,
    action_handler: impl 'static + ActionHandler,
    deactivation_handler: impl 'static + DeactivationHandler,
    request_rx: Channel<Command>,
    update_receivers: Arc<Mutex<HashSet<WpA11yUpdatesV1>>>,
) {
    let (globals, event_queue) = match registry_queue_init(&connection) {
        Ok(data) => data,
        Err(_) => return,
    };

    let a11y_manager = match globals.bind(&event_queue.handle(), 1..=1, ()) {
        Ok(a11y_manager) => a11y_manager,
        Err(_) => return,
    };

    let mut event_loop = EventLoop::<State>::try_new().unwrap();
    let loop_handle = event_loop.handle();

    let mut state = State::new(
        &event_queue.handle(),
        loop_handle.clone(),
        surface,
        activation_handler,
        action_handler,
        deactivation_handler,
        update_receivers,
        a11y_manager,
    );

    loop_handle
        .insert_source(request_rx, |event, _, state| {
            if let channel::Event::Msg(event) = event {
                match event {
                    Command::WriteUpdate(fd, serialized) => state.write_update(fd, serialized),
                    Command::Exit => state.exit = true,
                }
            }
        })
        .unwrap();

    WaylandSource::new(connection, event_queue)
        .insert(loop_handle)
        .unwrap();

    loop {
        event_loop.dispatch(None, &mut state).unwrap();

        if state.exit {
            break;
        }
    }
}
