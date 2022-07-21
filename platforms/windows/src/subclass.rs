// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{cell::Cell, ffi::c_void, mem::transmute};
use windows::{
    core::*,
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
};

use crate::Adapter;

const PROP_NAME: &str = "AccessKitAdapter";

struct SubclassData<'a> {
    adapter: &'a Adapter,
    prev_wnd_proc: WNDPROC,
    window_destroyed: Cell<bool>,
}

extern "system" fn wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let handle = unsafe { GetPropW(window, PROP_NAME) };
    let data_ptr = handle.0 as *const SubclassData;
    assert!(!data_ptr.is_null());
    let data = unsafe { &*data_ptr };
    if message == WM_GETOBJECT {
        if let Some(result) = data.adapter.handle_wm_getobject(wparam, lparam) {
            return result.into();
        }
    }
    if message == WM_NCDESTROY {
        data.window_destroyed.set(true);
    }
    unsafe { CallWindowProcW(data.prev_wnd_proc, window, message, wparam, lparam) }
}

/// Uses [Win32 subclassing] to handle `WM_GETOBJECT` messages on a window
/// that provides no other way of adding custom message handlers.
///
/// [Win32 subclassing]: https://docs.microsoft.com/en-us/windows/win32/controls/subclassing-overview
#[repr(transparent)]
pub struct WindowSubclass<'a>(Box<SubclassData<'a>>);

impl<'a> WindowSubclass<'a> {
    pub fn new(adapter: &'a Adapter) -> Self {
        let hwnd = adapter.window_handle();
        let mut data = Box::new(SubclassData {
            adapter,
            prev_wnd_proc: None,
            window_destroyed: Cell::new(false),
        });
        unsafe { SetPropW(hwnd, PROP_NAME, HANDLE(&*data as *const SubclassData as _)) }.unwrap();
        let result =
            unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as *const c_void as _) };
        if result == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        data.prev_wnd_proc = unsafe { transmute::<isize, WNDPROC>(result) };
        Self(data)
    }
}

impl Drop for WindowSubclass<'_> {
    fn drop(&mut self) {
        if !self.0.window_destroyed.get() {
            let hwnd = self.0.adapter.window_handle();
            let result = unsafe {
                SetWindowLongPtrW(
                    hwnd,
                    GWLP_WNDPROC,
                    transmute::<WNDPROC, isize>(self.0.prev_wnd_proc),
                )
            };
            if result == 0 {
                let result: Result<()> = Err(Error::from_win32());
                result.unwrap();
            }
            unsafe { RemovePropW(hwnd, PROP_NAME) }.unwrap();
        }
    }
}
