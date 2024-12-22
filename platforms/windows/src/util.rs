// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::TreeState;
use std::{
    fmt::{self, Write},
    sync::{Arc, Weak},
};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, Ole::*, Variant::*},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

use crate::window_handle::WindowHandle;

pub(crate) struct WideString<'a>(&'a mut Vec<u16>);

impl<'a> WideString<'a> {
    pub(crate) fn new(buffer: &'a mut Vec<u16>) -> Self {
        buffer.clear();
        Self(buffer)
    }
}

impl Write for WideString<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend(s.encode_utf16());
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0.extend_from_slice(c.encode_utf16(&mut [0; 2]));
        Ok(())
    }
}

impl PartialEq for WideString<'_> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl From<WideString<'_>> for BSTR {
    fn from(value: WideString) -> Self {
        Self::from_wide(value.0).unwrap()
    }
}

pub(crate) struct Variant(VARIANT);

impl From<Variant> for VARIANT {
    fn from(variant: Variant) -> Self {
        variant.0
    }
}

impl Variant {
    pub(crate) fn empty() -> Self {
        Self(VARIANT::default())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<BSTR> for Variant {
    fn from(value: BSTR) -> Self {
        Self(value.into())
    }
}

impl From<WideString<'_>> for Variant {
    fn from(value: WideString) -> Self {
        BSTR::from(value).into()
    }
}

pub(crate) struct StrWrapper<'a> {
    value: &'a str,
    buffer: &'a mut Vec<u16>,
}

impl<'a> StrWrapper<'a> {
    pub(crate) fn new(value: &'a str, buffer: &'a mut Vec<u16>) -> Self {
        Self { value, buffer }
    }
}

impl PartialEq for StrWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl From<StrWrapper<'_>> for Variant {
    fn from(w: StrWrapper) -> Self {
        let mut result = WideString::new(w.buffer);
        result.write_str(w.value).unwrap();
        result.into()
    }
}

impl From<IUnknown> for Variant {
    fn from(value: IUnknown) -> Self {
        Self(value.into())
    }
}

impl From<i32> for Variant {
    fn from(value: i32) -> Self {
        Self(value.into())
    }
}

impl From<f64> for Variant {
    fn from(value: f64) -> Self {
        Self(value.into())
    }
}

impl From<ToggleState> for Variant {
    fn from(value: ToggleState) -> Self {
        Self(value.0.into())
    }
}

impl From<LiveSetting> for Variant {
    fn from(value: LiveSetting) -> Self {
        Self(value.0.into())
    }
}

impl From<CaretPosition> for Variant {
    fn from(value: CaretPosition) -> Self {
        Self(value.0.into())
    }
}

impl From<UIA_CONTROLTYPE_ID> for Variant {
    fn from(value: UIA_CONTROLTYPE_ID) -> Self {
        Self(value.0.into())
    }
}

impl From<OrientationType> for Variant {
    fn from(value: OrientationType) -> Self {
        Self(value.0.into())
    }
}

impl From<bool> for Variant {
    fn from(value: bool) -> Self {
        Self(value.into())
    }
}

impl<T: Into<Variant>> From<Option<T>> for Variant {
    fn from(value: Option<T>) -> Self {
        value.map_or_else(Self::empty, T::into)
    }
}

fn safe_array_from_primitive_slice<T>(vt: VARENUM, slice: &[T]) -> *mut SAFEARRAY {
    let sa = unsafe { SafeArrayCreateVector(VARENUM(vt.0), 0, slice.len().try_into().unwrap()) };
    if sa.is_null() {
        panic!("SAFEARRAY allocation failed");
    }
    for (i, item) in slice.iter().enumerate() {
        let i: i32 = i.try_into().unwrap();
        unsafe { SafeArrayPutElement(&*sa, &i, (item as *const T) as *const _) }.unwrap();
    }
    sa
}

pub(crate) fn safe_array_from_i32_slice(slice: &[i32]) -> *mut SAFEARRAY {
    safe_array_from_primitive_slice(VT_I4, slice)
}

pub(crate) fn safe_array_from_f64_slice(slice: &[f64]) -> *mut SAFEARRAY {
    safe_array_from_primitive_slice(VT_R8, slice)
}

pub(crate) fn safe_array_from_com_slice(slice: &[IUnknown]) -> *mut SAFEARRAY {
    let sa = unsafe { SafeArrayCreateVector(VT_UNKNOWN, 0, slice.len().try_into().unwrap()) };
    if sa.is_null() {
        panic!("SAFEARRAY allocation failed");
    }
    for (i, item) in slice.iter().enumerate() {
        let i: i32 = i.try_into().unwrap();
        unsafe { SafeArrayPutElement(&*sa, &i, std::mem::transmute_copy(item)) }.unwrap();
    }
    sa
}

pub(crate) enum QueuedEvent {
    Simple {
        element: IRawElementProviderSimple,
        event_id: UIA_EVENT_ID,
    },
    PropertyChanged {
        element: IRawElementProviderSimple,
        property_id: UIA_PROPERTY_ID,
        old_value: VARIANT,
        new_value: VARIANT,
    },
}

pub(crate) fn not_implemented() -> Error {
    E_NOTIMPL.into()
}

pub(crate) fn invalid_arg() -> Error {
    E_INVALIDARG.into()
}

pub(crate) fn required_param<T>(param: Option<&T>) -> Result<&T> {
    param.map_or_else(|| Err(invalid_arg()), Ok)
}

pub(crate) fn element_not_available() -> Error {
    HRESULT(UIA_E_ELEMENTNOTAVAILABLE as _).into()
}

pub(crate) fn invalid_operation() -> Error {
    HRESULT(UIA_E_INVALIDOPERATION as _).into()
}

pub(crate) fn client_top_left(hwnd: WindowHandle) -> Point {
    let mut result = POINT::default();
    // If ClientToScreen fails, that means the window is gone.
    // That's an unexpected condition, so we should fail loudly.
    unsafe { ClientToScreen(hwnd.0, &mut result) }.unwrap();
    Point::new(result.x.into(), result.y.into())
}

pub(crate) fn window_title(hwnd: WindowHandle, buffer: &mut Vec<u16>) -> Option<BSTR> {
    // The following is an old hack to get the window caption without ever
    // sending messages to the window itself, even if the window is in
    // the same process but possibly a separate thread. This prevents
    // possible hangs and sluggishness. This hack has been proven to work
    // over nearly 20 years on every version of Windows back to XP.
    let result = unsafe { DefWindowProcW(hwnd.0, WM_GETTEXTLENGTH, WPARAM(0), LPARAM(0)) };
    if result.0 <= 0 {
        return None;
    }
    let capacity = (result.0 as usize) + 1; // make room for the null
    buffer.clear();
    buffer.reserve(capacity);
    let result = unsafe {
        DefWindowProcW(
            hwnd.0,
            WM_GETTEXT,
            WPARAM(capacity),
            LPARAM(buffer.as_mut_ptr() as _),
        )
    };
    if result.0 <= 0 {
        return None;
    }
    let len = result.0 as usize;
    unsafe { buffer.set_len(len) };
    Some(BSTR::from_wide(buffer).unwrap())
}

pub(crate) fn toolkit_description<'a>(
    state: &TreeState,
    buffer: &'a mut Vec<u16>,
) -> Option<WideString<'a>> {
    state.toolkit_name().map(|name| {
        let mut result = WideString::new(buffer);
        result.write_str(name).unwrap();
        if let Some(version) = state.toolkit_version() {
            result.write_char(' ').unwrap();
            result.write_str(version).unwrap();
        }
        result
    })
}

pub(crate) fn upgrade<T>(weak: &Weak<T>) -> Result<Arc<T>> {
    if let Some(strong) = weak.upgrade() {
        Ok(strong)
    } else {
        Err(element_not_available())
    }
}
