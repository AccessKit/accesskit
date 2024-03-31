// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    LocalPythonActivationHandler, PythonActionHandler, PythonActivationHandler, TreeUpdate,
};
use accesskit_windows::{HWND, LPARAM, WPARAM};
use pyo3::prelude::*;

#[pyclass(module = "accesskit.windows")]
pub struct QueuedEvents(Option<accesskit_windows::QueuedEvents>);

#[pymethods]
impl QueuedEvents {
    pub fn raise_events(&mut self) {
        let events = self.0.take().unwrap();
        events.raise();
    }
}

impl From<accesskit_windows::QueuedEvents> for QueuedEvents {
    fn from(events: accesskit_windows::QueuedEvents) -> Self {
        Self(Some(events))
    }
}

#[pyclass(module = "accesskit.windows")]
pub struct Adapter(accesskit_windows::Adapter);

#[pymethods]
impl Adapter {
    /// Creates a new Windows platform adapter.
    ///
    /// The action handler may or may not be called on the thread that owns
    /// the window.
    #[new]
    pub fn new(hwnd: &PyAny, is_window_focused: bool, action_handler: Py<PyAny>) -> Self {
        Self(accesskit_windows::Adapter::new(
            HWND(cast::<isize>(hwnd)),
            is_window_focused,
            PythonActionHandler(action_handler),
        ))
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value. It can be `None` if the window is not active.
    pub fn update_if_active(
        &mut self,
        py: Python<'_>,
        update_factory: Py<PyAny>,
    ) -> Option<QueuedEvents> {
        self.0
            .update_if_active(|| {
                let update = update_factory.call0(py).unwrap();
                update.extract::<TreeUpdate>(py).unwrap().into()
            })
            .map(Into::into)
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value.
    pub fn update_window_focus_state(&mut self, is_focused: bool) -> Option<QueuedEvents> {
        self.0.update_window_focus_state(is_focused).map(Into::into)
    }

    pub fn handle_wm_getobject(
        &mut self,
        py: Python<'_>,
        wparam: &PyAny,
        lparam: &PyAny,
        activation_handler: Py<PyAny>,
    ) -> Option<isize> {
        let mut activation_handler = LocalPythonActivationHandler {
            py,
            handler: activation_handler,
        };
        self.0
            .handle_wm_getobject(
                WPARAM(cast::<usize>(wparam)),
                LPARAM(cast::<isize>(lparam)),
                &mut activation_handler,
            )
            .map(|lresult| lresult.into().0)
    }
}

#[pyclass(module = "accesskit.windows", unsendable)]
pub struct SubclassingAdapter(accesskit_windows::SubclassingAdapter);

#[pymethods]
impl SubclassingAdapter {
    /// Creates a new Windows platform adapter using window subclassing.
    /// This must be done before the window is shown or focused
    /// for the first time.
    ///
    /// The action handler may or may not be called on the thread that owns
    /// the window.
    #[new]
    pub fn new(hwnd: &PyAny, activation_handler: Py<PyAny>, action_handler: Py<PyAny>) -> Self {
        Self(accesskit_windows::SubclassingAdapter::new(
            HWND(cast::<isize>(hwnd)),
            PythonActivationHandler(activation_handler),
            PythonActionHandler(action_handler),
        ))
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value. It can be `None` if the window is not active.
    pub fn update_if_active(
        &mut self,
        py: Python<'_>,
        update_factory: Py<PyAny>,
    ) -> Option<QueuedEvents> {
        self.0
            .update_if_active(|| {
                let update = update_factory.call0(py).unwrap();
                update.extract::<TreeUpdate>(py).unwrap().into()
            })
            .map(Into::into)
    }
}

fn cast<'a, D: FromPyObject<'a>>(value: &'a PyAny) -> D {
    let value = value.getattr("value").unwrap_or(value);
    value.extract().unwrap()
}
