// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{convert::TryInto, mem::ManuallyDrop};
use windows::{
    core::*,
    Win32::{
        System::{Com::*, Ole::*},
        UI::Accessibility::*,
    },
};

pub(crate) struct VariantFactory(VARENUM, VARIANT_0_0_0);

impl From<VariantFactory> for VARIANT {
    fn from(factory: VariantFactory) -> Self {
        let VariantFactory(vt, value) = factory;
        Self {
            Anonymous: VARIANT_0 {
                Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                    vt: VARENUM(vt.0 as u16),
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    Anonymous: value,
                }),
            },
        }
    }
}

impl VariantFactory {
    pub(crate) fn empty() -> Self {
        // The choice of value field is probably arbitrary, but it seems
        // reasonable to make sure that at least a whole machine word is zero.
        Self(VT_EMPTY, VARIANT_0_0_0 { llVal: 0 })
    }
}

impl From<&str> for VariantFactory {
    fn from(value: &str) -> Self {
        let value: BSTR = value.into();
        value.into()
    }
}

impl From<String> for VariantFactory {
    fn from(value: String) -> Self {
        let value: BSTR = value.into();
        value.into()
    }
}

impl From<BSTR> for VariantFactory {
    fn from(value: BSTR) -> Self {
        Self(
            VT_BSTR,
            VARIANT_0_0_0 {
                bstrVal: ManuallyDrop::new(value),
            },
        )
    }
}

impl From<i32> for VariantFactory {
    fn from(value: i32) -> Self {
        Self(VT_I4, VARIANT_0_0_0 { lVal: value })
    }
}

impl From<f64> for VariantFactory {
    fn from(value: f64) -> Self {
        Self(VT_R8, VARIANT_0_0_0 { dblVal: value })
    }
}

impl From<ToggleState> for VariantFactory {
    fn from(value: ToggleState) -> Self {
        value.0.into()
    }
}

impl From<LiveSetting> for VariantFactory {
    fn from(value: LiveSetting) -> Self {
        value.0.into()
    }
}

const VARIANT_FALSE: i16 = 0i16;
const VARIANT_TRUE: i16 = -1i16;

impl From<bool> for VariantFactory {
    fn from(value: bool) -> Self {
        Self(
            VT_BOOL,
            VARIANT_0_0_0 {
                boolVal: if value { VARIANT_TRUE } else { VARIANT_FALSE },
            },
        )
    }
}

impl<T: Into<VariantFactory>> From<Option<T>> for VariantFactory {
    fn from(value: Option<T>) -> Self {
        value.map_or_else(Self::empty, T::into)
    }
}

fn safe_array_from_primitive_slice<T>(vt: VARENUM, slice: &[T]) -> *mut SAFEARRAY {
    let sa =
        unsafe { SafeArrayCreateVector(VARENUM(vt.0 as u16), 0, slice.len().try_into().unwrap()) };
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

pub(crate) fn safe_array_from_com_slice(slice: &[IUnknown]) -> *mut SAFEARRAY {
    let sa =
        unsafe { SafeArrayCreateVector(VT_UNKNOWN, 0, slice.len().try_into().unwrap()) };
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
        event_id: i32,
    },
    PropertyChanged {
        element: IRawElementProviderSimple,
        property_id: i32,
        old_value: VARIANT,
        new_value: VARIANT,
    },
}
