// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use pyo3::prelude::*;

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct Affine(accesskit::Affine);

#[pymethods]
impl Affine {
    #[new]
    pub fn new(c: [f64; 6]) -> Affine {
        accesskit::Affine::new(c).into()
    }

    #[getter]
    pub fn coeffs(&self) -> [f64; 6] {
        self.0.as_coeffs()
    }
}

impl From<Affine> for Box<accesskit::Affine> {
    fn from(value: Affine) -> Self {
        Box::new(value.0)
    }
}

impl From<accesskit::Affine> for Affine {
    fn from(value: accesskit::Affine) -> Self {
        Self(value)
    }
}

impl From<&accesskit::Affine> for Affine {
    fn from(value: &accesskit::Affine) -> Self {
        Self(*value)
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct Point(accesskit::Point);

#[pymethods]
impl Point {
    #[new]
    pub fn new(x: f64, y: f64) -> Self {
        accesskit::Point::new(x, y).into()
    }

    #[getter]
    pub fn get_x(&self) -> f64 {
        self.0.x
    }

    #[setter]
    pub fn set_x(&mut self, value: f64) {
        self.0.x = value
    }

    #[getter]
    pub fn get_y(&self) -> f64 {
        self.0.y
    }

    #[setter]
    pub fn set_y(&mut self, value: f64) {
        self.0.y = value
    }
}

impl From<Point> for accesskit::Point {
    fn from(value: Point) -> Self {
        value.0
    }
}

impl From<accesskit::Point> for Point {
    fn from(value: accesskit::Point) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct Rect(accesskit::Rect);

#[pymethods]
impl Rect {
    #[new]
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
        accesskit::Rect::new(x0, y0, x1, y1).into()
    }

    #[staticmethod]
    pub fn from_points(p0: Point, p1: Point) -> Self {
        let p0 = accesskit::Point::from(p0);
        let p1 = accesskit::Point::from(p1);
        accesskit::Rect::from_points(p0, p1).into()
    }

    #[staticmethod]
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        let origin = accesskit::Point::from(origin);
        let size = accesskit::Size::from(size);
        accesskit::Rect::from_origin_size(origin, size).into()
    }

    #[getter]
    pub fn get_x0(&self) -> f64 {
        self.0.x0
    }

    #[setter]
    pub fn set_x0(&mut self, value: f64) {
        self.0.x0 = value
    }

    #[getter]
    pub fn get_y0(&self) -> f64 {
        self.0.y0
    }

    #[setter]
    pub fn set_y0(&mut self, value: f64) {
        self.0.y0 = value
    }

    #[getter]
    pub fn get_x1(&self) -> f64 {
        self.0.x1
    }

    #[setter]
    pub fn set_x1(&mut self, value: f64) {
        self.0.x1 = value
    }

    #[getter]
    pub fn get_y1(&self) -> f64 {
        self.0.y1
    }

    #[setter]
    pub fn set_y1(&mut self, value: f64) {
        self.0.y1 = value
    }
}

impl From<Rect> for accesskit::Rect {
    fn from(value: Rect) -> Self {
        value.0
    }
}

impl From<accesskit::Rect> for Rect {
    fn from(value: accesskit::Rect) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct Size(accesskit::Size);

#[pymethods]
impl Size {
    #[new]
    pub fn new(width: f64, height: f64) -> Self {
        accesskit::Size::new(width, height).into()
    }

    #[getter]
    pub fn get_width(&self) -> f64 {
        self.0.width
    }

    #[setter]
    pub fn set_width(&mut self, value: f64) {
        self.0.width = value
    }

    #[getter]
    pub fn get_height(&self) -> f64 {
        self.0.height
    }

    #[setter]
    pub fn set_height(&mut self, value: f64) {
        self.0.height = value
    }
}

impl From<Size> for accesskit::Size {
    fn from(value: Size) -> Self {
        value.0
    }
}

impl From<accesskit::Size> for Size {
    fn from(value: accesskit::Size) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
#[pyclass(module = "accesskit")]
pub struct Vec2(accesskit::Vec2);

#[pymethods]
impl Vec2 {
    #[new]
    pub fn new(x: f64, y: f64) -> Vec2 {
        accesskit::Vec2::new(x, y).into()
    }

    #[getter]
    pub fn get_x(&self) -> f64 {
        self.0.x
    }

    #[setter]
    pub fn set_x(&mut self, value: f64) {
        self.0.x = value
    }

    #[getter]
    pub fn get_y(&self) -> f64 {
        self.0.y
    }

    #[setter]
    pub fn set_y(&mut self, value: f64) {
        self.0.y = value
    }
}

impl From<Vec2> for accesskit::Vec2 {
    fn from(value: Vec2) -> Self {
        value.0
    }
}

impl From<accesskit::Vec2> for Vec2 {
    fn from(value: accesskit::Vec2) -> Self {
        Self(value)
    }
}
