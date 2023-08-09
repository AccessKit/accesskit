// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{PythonActionHandler, Rect, TreeUpdate};
use pyo3::prelude::*;

#[pyclass(module = "accesskit.unix")]
pub struct Adapter(accesskit_unix::Adapter);

#[pymethods]
impl Adapter {
    #[staticmethod]
    pub fn create(
        source: Py<PyAny>,
        is_window_focused: bool,
        action_handler: Py<PyAny>,
    ) -> Option<Self> {
        accesskit_unix::Adapter::new(
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
            is_window_focused,
            Box::new(PythonActionHandler(action_handler)),
        )
        .map(Self)
    }

    pub fn set_root_window_bounds(&mut self, outer: Rect, inner: Rect) {
        self.0.set_root_window_bounds(outer.into(), inner.into());
    }

    pub fn update(&self, update: TreeUpdate) {
        self.0.update(update.into());
    }

    pub fn update_window_focus_state(&self, is_focused: bool) {
        self.0.update_window_focus_state(is_focused);
    }
}
