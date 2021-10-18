// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_windows_bindings::Windows::Win32::{Foundation::*, System::OleAutomation::*};
use std::{convert::TryInto, mem::ManuallyDrop};

pub(crate) const fn variant(vt: VARENUM, value: VARIANT_0_0_0) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: vt.0 as u16,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: value,
            }),
        },
    }
}

pub(crate) const fn empty_variant() -> VARIANT {
    // The choice of value field is probably arbitrary, but it seems reasonable
    // to make sure that at least a whole machine word is zero.
    variant(VT_EMPTY, VARIANT_0_0_0 { llVal: 0 })
}

pub(crate) const fn variant_from_bstr(value: BSTR) -> VARIANT {
    variant(
        VT_BSTR,
        VARIANT_0_0_0 {
            bstrVal: ManuallyDrop::new(value),
        },
    )
}

pub(crate) const fn variant_from_i32(value: i32) -> VARIANT {
    variant(VT_I4, VARIANT_0_0_0 { lVal: value })
}

const VARIANT_FALSE: i16 = 0i16;
const VARIANT_TRUE: i16 = -1i16;

pub(crate) const fn variant_from_bool(value: bool) -> VARIANT {
    variant(
        VT_BOOL,
        VARIANT_0_0_0 {
            boolVal: if value { VARIANT_TRUE } else { VARIANT_FALSE },
        },
    )
}

fn safe_array_from_slice<T>(vt: VARENUM, slice: &[T]) -> *mut SAFEARRAY {
    let sa = unsafe { SafeArrayCreateVector(vt.0 as u16, 0, slice.len().try_into().unwrap()) };
    if sa.is_null() {
        panic!("SAFEARRAY allocation failed");
    }
    for (i, item) in slice.iter().enumerate() {
        let i: i32 = i.try_into().unwrap();
        unsafe { SafeArrayPutElement(sa, &i, (item as *const T) as *const _) }.unwrap();
    }
    sa
}

pub(crate) fn safe_array_from_i32_slice(slice: &[i32]) -> *mut SAFEARRAY {
    safe_array_from_slice(VT_I4, slice)
}
