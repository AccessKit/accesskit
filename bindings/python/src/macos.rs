// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{PythonActionHandler, TreeUpdate};
use accesskit_macos::NSPoint;
use pyo3::{prelude::*, types::PyCapsule};
use std::ffi::c_void;

/// This class must only be used from the main thread.
#[pyclass(module = "accesskit.macos", unsendable)]
pub struct QueuedEvents(Option<accesskit_macos::QueuedEvents>);

#[pymethods]
impl QueuedEvents {
    pub fn raise_events(&mut self) {
        let events = self.0.take().unwrap();
        events.raise();
    }
}

impl From<accesskit_macos::QueuedEvents> for QueuedEvents {
    fn from(events: accesskit_macos::QueuedEvents) -> Self {
        Self(Some(events))
    }
}

/// This class must only be used from the main thread.
#[pyclass(module = "accesskit.macos", unsendable)]
pub struct Adapter(accesskit_macos::Adapter);

#[pymethods]
impl Adapter {
    /// Create a new macOS adapter.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    #[new]
    pub unsafe fn new(
        view: &PyAny,
        initial_state: TreeUpdate,
        is_view_focused: bool,
        handler: Py<PyAny>,
    ) -> Self {
        Self(accesskit_macos::Adapter::new(
            to_void_ptr(view),
            initial_state.into(),
            is_view_focused,
            Box::new(PythonActionHandler(handler)),
        ))
    }

    /// You must call `accesskit.macos.QueuedEvents.raise_events` on the returned value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        self.0.update(update.into()).into()
    }

    /// You must call `accesskit.macos.QueuedEvents.raise_events` on the returned value.
    pub fn update_view_focus_state(&self, is_focused: bool) -> QueuedEvents {
        self.0.update_view_focus_state(is_focused).into()
    }

    pub fn view_children(&self, py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        let ptr: isize = self.0.view_children() as _;
        Ok(PyCapsule::new(py, ptr, None)?.into())
    }

    pub fn focus(&self, py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        let ptr: isize = self.0.focus() as _;
        Ok(PyCapsule::new(py, ptr, None)?.into())
    }

    pub fn hit_test(&self, py: Python<'_>, x: f64, y: f64) -> PyResult<Py<PyCapsule>> {
        let ptr: isize = self.0.hit_test(NSPoint::new(x, y)) as _;
        Ok(PyCapsule::new(py, ptr, None)?.into())
    }
}

/// This class must only be used from the main thread.
#[pyclass(module = "accesskit.macos", unsendable)]
pub struct SubclassingAdapter(accesskit_macos::SubclassingAdapter);

#[pymethods]
impl SubclassingAdapter {
    /// Create an adapter that dynamically subclasses the specified view.
    /// This must be done before the view is shown or focused for
    /// the first time.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    #[new]
    pub unsafe fn new(view: &PyAny, source: Py<PyAny>, handler: Py<PyAny>) -> Self {
        Self(accesskit_macos::SubclassingAdapter::new(
            to_void_ptr(view),
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
            Box::new(PythonActionHandler(handler)),
        ))
    }

    /// Create an adapter that dynamically subclasses the content view
    /// of the specified window.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `window` must be a valid, unreleased pointer to an `NSWindow`.
    ///
    /// # Panics
    ///
    /// This function panics if the specified window doesn't currently have
    /// a content view.
    #[staticmethod]
    pub unsafe fn for_window(window: &PyAny, source: Py<PyAny>, handler: Py<PyAny>) -> Self {
        Self(accesskit_macos::SubclassingAdapter::for_window(
            to_void_ptr(window),
            move || {
                Python::with_gil(|py| {
                    source
                        .call0(py)
                        .unwrap()
                        .extract::<crate::TreeUpdate>(py)
                        .unwrap()
                        .into()
                })
            },
            Box::new(PythonActionHandler(handler)),
        ))
    }

    /// You must call `accesskit.macos.QueuedEvents.raise_events` on the returned value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        self.0.update(update.into()).into()
    }

    /// You must call `accesskit.macos.QueuedEvents.raise_events` on the returned value. It can be `None` if the window is not active.
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

    /// You must call `accesskit.macos.QueuedEvents.raise_events` on the returned value. It can be `None` if the window is not active.
    pub fn update_view_focus_state(&self, is_focused: bool) -> Option<QueuedEvents> {
        self.0.update_view_focus_state(is_focused).map(Into::into)
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
#[pyfunction]
pub unsafe fn add_focus_forwarder_to_window_class(class_name: &str) {
    accesskit_macos::add_focus_forwarder_to_window_class(class_name)
}

fn to_void_ptr(value: &PyAny) -> *mut c_void {
    if let Ok(value) = value.extract::<&PyCapsule>() {
        return value.pointer();
    }
    let value = value.getattr("value").unwrap_or(value);
    value.extract::<isize>().unwrap() as *mut _
}
