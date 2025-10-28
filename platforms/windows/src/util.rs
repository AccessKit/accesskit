// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::Point;
use accesskit_consumer::TreeState;
use std::{
    fmt::{self, Write},
    mem::ManuallyDrop,
    sync::{Arc, Weak},
};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Globalization::*,
        Graphics::Gdi::*,
        System::{Com::*, Ole::*, Variant::*},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

use crate::window_handle::WindowHandle;

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct WideString(Vec<u16>);

impl Write for WideString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend(s.encode_utf16());
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0.extend_from_slice(c.encode_utf16(&mut [0; 2]));
        Ok(())
    }
}

impl From<WideString> for BSTR {
    fn from(value: WideString) -> Self {
        Self::from_wide(&value.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LocaleName<'a>(pub(crate) &'a str);

impl From<LocaleName<'_>> for Variant {
    fn from(value: LocaleName) -> Self {
        let lcid = unsafe { LocaleNameToLCID(&HSTRING::from(value.0), LOCALE_ALLOW_NEUTRAL_NAMES) };
        (lcid != 0).then_some(lcid as i32).into()
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

impl From<WideString> for Variant {
    fn from(value: WideString) -> Self {
        BSTR::from(value).into()
    }
}

impl From<&str> for Variant {
    fn from(value: &str) -> Self {
        let mut result = WideString::default();
        result.write_str(value).unwrap();
        result.into()
    }
}

impl From<String> for Variant {
    fn from(value: String) -> Self {
        value.as_str().into()
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

impl From<Vec<IUnknown>> for Variant {
    fn from(value: Vec<IUnknown>) -> Self {
        if value.is_empty() {
            Variant::empty()
        } else {
            let parray = safe_array_from_com_slice(&value);
            Self(VARIANT {
                Anonymous: VARIANT_0 {
                    Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                        vt: VT_ARRAY | VT_UNKNOWN,
                        wReserved1: 0,
                        wReserved2: 0,
                        wReserved3: 0,
                        Anonymous: VARIANT_0_0_0 { parray },
                    }),
                },
            })
        }
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

pub(crate) fn required_param<'a, T: Interface>(param: &'a Ref<T>) -> Result<&'a T> {
    param.ok().map_err(|_| invalid_arg())
}

pub(crate) fn element_not_available() -> Error {
    HRESULT(UIA_E_ELEMENTNOTAVAILABLE as _).into()
}

pub(crate) fn element_not_enabled() -> Error {
    HRESULT(UIA_E_ELEMENTNOTENABLED as _).into()
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

pub(crate) fn window_title(hwnd: WindowHandle) -> Option<BSTR> {
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
    let mut buffer = Vec::<u16>::with_capacity(capacity);
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
    Some(BSTR::from_wide(&buffer))
}

pub(crate) fn toolkit_description(state: &TreeState) -> Option<WideString> {
    state.toolkit_name().map(|name| {
        let mut result = WideString::default();
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
