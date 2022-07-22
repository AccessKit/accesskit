// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{cell::Cell, ffi::c_void, mem::transmute, ops::Deref};
use windows::{
    core::*,
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
};

use crate::Adapter;

const PROP_NAME: &str = "AccessKitAdapter";

struct SubclassImpl {
    adapter: Adapter,
    prev_wnd_proc: WNDPROC,
    window_destroyed: Cell<bool>,
}

extern "system" fn wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let handle = unsafe { GetPropW(window, PROP_NAME) };
    let impl_ptr = handle.0 as *const SubclassImpl;
    assert!(!impl_ptr.is_null());
    let r#impl = unsafe { &*impl_ptr };
    if message == WM_GETOBJECT {
        if let Some(result) = r#impl.adapter.handle_wm_getobject(wparam, lparam) {
            return result.into();
        }
    }
    if message == WM_NCDESTROY {
        r#impl.window_destroyed.set(true);
    }
    unsafe { CallWindowProcW(r#impl.prev_wnd_proc, window, message, wparam, lparam) }
}

impl SubclassImpl {
    fn new(adapter: Adapter) -> Box<Self> {
        Box::new(Self {
            adapter,
            prev_wnd_proc: None,
            window_destroyed: Cell::new(false),
        })
    }

    fn install(&mut self) {
        let hwnd = self.adapter.window_handle();
        unsafe { SetPropW(hwnd, PROP_NAME, HANDLE(self as *const SubclassImpl as _)) }.unwrap();
        let result =
            unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as *const c_void as _) };
        if result == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        self.prev_wnd_proc = unsafe { transmute::<isize, WNDPROC>(result) };
    }

    fn uninstall(&self) {
        if self.window_destroyed.get() {
            return;
        }
        let hwnd = self.adapter.window_handle();
        let result = unsafe {
            SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                transmute::<WNDPROC, isize>(self.prev_wnd_proc),
            )
        };
        if result == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        unsafe { RemovePropW(hwnd, PROP_NAME) }.unwrap();
    }
}

/// Uses [Win32 subclassing] to handle `WM_GETOBJECT` messages on a window
/// that provides no other way of adding custom message handlers.
///
/// [Win32 subclassing]: https://docs.microsoft.com/en-us/windows/win32/controls/subclassing-overview
#[repr(transparent)]
pub struct SubclassingAdapter(Option<Box<SubclassImpl>>);

impl SubclassingAdapter {
    pub fn new(adapter: Adapter) -> Self {
        let mut r#impl = SubclassImpl::new(adapter);
        r#impl.install();
        Self(Some(r#impl))
    }

    pub fn inner(&self) -> &Adapter {
        &self.0.as_ref().unwrap().adapter
    }

    pub fn into_inner(mut self) -> Adapter {
        let r#impl = self.0.take().unwrap();
        r#impl.uninstall();
        r#impl.adapter
    }
}

impl Deref for SubclassingAdapter {
    type Target = Adapter;

    fn deref(&self) -> &Adapter {
        self.inner()
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        if let Some(r#impl) = self.0.as_ref() {
            r#impl.uninstall();
        }
    }
}
