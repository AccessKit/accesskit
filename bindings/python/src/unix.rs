// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    PythonActionHandler, PythonActivationHandler, PythonDeactivationHandler, Rect, TreeUpdate,
};
use pyo3::prelude::*;

#[pyclass(module = "accesskit.unix")]
pub struct Adapter(accesskit_unix::Adapter);

#[pymethods]
impl Adapter {
    /// Create a new Unix adapter.
    ///
    /// All of the handlers will always be called from another thread.
    #[new]
    pub fn new(
        activation_handler: Py<PyAny>,
        action_handler: Py<PyAny>,
        deactivation_handler: Py<PyAny>,
    ) -> Self {
        Self(accesskit_unix::Adapter::new(
            PythonActivationHandler(activation_handler),
            PythonActionHandler(action_handler),
            PythonDeactivationHandler(deactivation_handler),
        ))
    }

    pub fn set_root_window_bounds(&mut self, outer: Rect, inner: Rect) {
        self.0.set_root_window_bounds(outer.into(), inner.into());
    }

    pub fn update_if_active(&mut self, py: Python<'_>, update_factory: Py<PyAny>) {
        self.0.update_if_active(|| {
            let update = update_factory.call0(py).unwrap();
            update.extract::<TreeUpdate>(py).unwrap().into()
        });
    }

    pub fn update_window_focus_state(&mut self, is_focused: bool) {
        self.0.update_window_focus_state(is_focused);
    }
}
