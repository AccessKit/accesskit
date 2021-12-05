// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::interfaces::Accessible;
use std::collections::HashMap;
use zbus::{SignalContext, Result, dbus_interface};
use zvariant::Value;

pub struct FocusEventsInterface;

impl FocusEventsInterface {
    pub async fn focused(&self, ctxt: &SignalContext<'_>) -> Result<()> {
        FocusEventsInterface::focus(ctxt, "", 0, 0, 0i32.into(), HashMap::new()).await
    }
}

#[dbus_interface(name = "org.a11y.atspi.Event.Focus")]
impl FocusEventsInterface {
    #[dbus_interface(signal)]
    async fn focus(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;
}

pub enum ObjectEvent {
    Activated,
    Deactivated,
    FocusGained,
    FocusLost,
}

pub struct ObjectEventsInterface;

impl ObjectEventsInterface {
    pub async fn emit(&self, event: ObjectEvent, ctxt: &SignalContext<'_>) -> Result<()> {
        let properties = HashMap::new();
        match event {
            ObjectEvent::Activated =>
                ObjectEventsInterface::state_changed(ctxt, "active", 1, 0, 0i32.into(), properties).await,
            ObjectEvent::Deactivated =>
                ObjectEventsInterface::state_changed(ctxt, "active", 0, 0, 0i32.into(), properties).await,
            ObjectEvent::FocusGained =>
                ObjectEventsInterface::state_changed(ctxt, "focused", 1, 0, 0i32.into(), properties).await,
            ObjectEvent::FocusLost =>
                ObjectEventsInterface::state_changed(ctxt, "focused", 0, 0, 0i32.into(), properties).await
        }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Event.Object")]
impl ObjectEventsInterface {
    #[dbus_interface(signal)]
    async fn state_changed(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;
}

pub enum WindowEvent {
    Activated,
    Closed,
    Created,
    Deactivated,
    Destroyed
}

pub struct WindowEventsInterface<T>(pub T);

impl<T: Accessible> WindowEventsInterface<T> {
    pub async fn emit(&self, event: WindowEvent, ctxt: &SignalContext<'_>) -> Result<()> {
        let name = self.0.name().into();
        let properties = HashMap::new();
        match event {
            WindowEvent::Activated =>
                WindowEventsInterface::<T>::activate(ctxt, "", 0, 0, name, properties).await,
            WindowEvent::Closed =>
                WindowEventsInterface::<T>::close(ctxt, "", 0, 0, name, properties).await,
            WindowEvent::Created =>
                WindowEventsInterface::<T>::create(ctxt, "", 0, 0, name, properties).await,
            WindowEvent::Deactivated =>
                WindowEventsInterface::<T>::deactivate(ctxt, "", 0, 0, name, properties).await,
            WindowEvent::Destroyed =>
                WindowEventsInterface::<T>::destroy(ctxt, "", 0, 0, name, properties).await
        }
    }
}

#[dbus_interface(name = "org.a11y.atspi.Event.Window")]
impl<T: Accessible> WindowEventsInterface<T> {
    #[dbus_interface(signal)]
    async fn activate(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;

    #[dbus_interface(signal)]
    async fn close(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;

    #[dbus_interface(signal)]
    async fn create(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;

    #[dbus_interface(signal)]
    async fn deactivate(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;

    #[dbus_interface(signal)]
    async fn destroy(
        ctxt: &SignalContext<'_>,
        minor: &str,
        detail1: i32,
        detail2: i32,
        any_data: Value<'_>,
        properties: HashMap<String, Value<'_>>
    ) -> Result<()>;
}
