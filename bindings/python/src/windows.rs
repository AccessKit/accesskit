// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{PythonActionHandler, TreeUpdate};
use accesskit_windows::{HWND, LPARAM, WPARAM};
use pyo3::prelude::*;

#[derive(Clone)]
#[pyclass(module = "accesskit.windows")]
pub struct UiaInitMarker(accesskit_windows::UiaInitMarker);

#[pymethods]
impl UiaInitMarker {
    #[new]
    pub fn __new__() -> Self {
        Self(accesskit_windows::UiaInitMarker::new())
    }
}

impl From<UiaInitMarker> for accesskit_windows::UiaInitMarker {
    fn from(marker: UiaInitMarker) -> Self {
        marker.0
    }
}

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
    pub fn new(
        hwnd: &PyAny,
        initial_state: TreeUpdate,
        is_window_focused: bool,
        action_handler: Py<PyAny>,
        uia_init_marker: UiaInitMarker,
    ) -> Self {
        Self(accesskit_windows::Adapter::new(
            HWND(cast::<isize>(hwnd)),
            initial_state.into(),
            is_window_focused,
            Box::new(PythonActionHandler(action_handler)),
            uia_init_marker.into(),
        ))
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        self.0.update(update.into()).into()
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value.
    pub fn update_window_focus_state(&self, is_focused: bool) -> QueuedEvents {
        self.0.update_window_focus_state(is_focused).into()
    }

    pub fn handle_wm_getobject(&self, wparam: &PyAny, lparam: &PyAny) -> Option<isize> {
        self.0
            .handle_wm_getobject(WPARAM(cast::<usize>(wparam)), LPARAM(cast::<isize>(lparam)))
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
    pub fn new(hwnd: &PyAny, source: Py<PyAny>, action_handler: Py<PyAny>) -> Self {
        Self(accesskit_windows::SubclassingAdapter::new(
            HWND(cast::<isize>(hwnd)),
            move || {
                Python::with_gil(|py| {
                    source
                        .call0(py)
                        .unwrap()
                        .extract::<TreeUpdate>(py)
                        .unwrap()
                        .into()
                })
            },
            Box::new(PythonActionHandler(action_handler)),
        ))
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        self.0.update(update.into()).into()
    }

    /// You must call `accesskit.windows.QueuedEvents.raise_events` on the returned value. It can be `None` if the window is not active.
    pub fn update_if_active(
        &self,
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
