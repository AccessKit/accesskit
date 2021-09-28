// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::mem::ManuallyDrop;

use accesskit_windows_bindings::Windows::Win32::{Foundation::*, System::OleAutomation::*};

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
