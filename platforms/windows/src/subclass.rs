// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use once_cell::unsync::Lazy;
use std::{cell::Cell, ffi::c_void, mem::transmute};
use windows::{
    core::*,
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
};

use crate::{Adapter, QueuedEvents, UiaInitMarker};

// Work around a difference between the SetWindowLongPtrW API definition
// in windows-rs on 32-bit and 64-bit Windows.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
type LongPtr = isize;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
type LongPtr = i32;

const PROP_NAME: PCWSTR = w!("AccessKitAdapter");

type LazyAdapter = Lazy<Adapter, Box<dyn FnOnce() -> Adapter>>;

struct SubclassImpl {
    hwnd: HWND,
    adapter: LazyAdapter,
    prev_wnd_proc: WNDPROC,
    window_destroyed: Cell<bool>,
}

extern "system" fn wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let handle = unsafe { GetPropW(window, PROP_NAME) };
    let impl_ptr = handle.0 as *const SubclassImpl;
    assert!(!impl_ptr.is_null());
    let r#impl = unsafe { &*impl_ptr };
    if message == WM_GETOBJECT {
        let adapter = Lazy::force(&r#impl.adapter);
        if let Some(result) = adapter.handle_wm_getobject(wparam, lparam) {
            return result.into();
        }
    }
    if message == WM_NCDESTROY {
        r#impl.window_destroyed.set(true);
    }
    unsafe { CallWindowProcW(r#impl.prev_wnd_proc, window, message, wparam, lparam) }
}

impl SubclassImpl {
    /// Creates a new Windows platform adapter using window subclassing.
    ///
    /// The action handler may or may not be called on the thread that owns
    /// the window.
    fn new(hwnd: HWND, adapter: LazyAdapter) -> Box<Self> {
        Box::new(Self {
            hwnd,
            adapter,
            prev_wnd_proc: None,
            window_destroyed: Cell::new(false),
        })
    }

    fn install(&mut self) {
        unsafe {
            SetPropW(
                self.hwnd,
                PROP_NAME,
                HANDLE(self as *const SubclassImpl as _),
            )
        }
        .unwrap();
        let result =
            unsafe { SetWindowLongPtrW(self.hwnd, GWLP_WNDPROC, wnd_proc as *const c_void as _) };
        if result == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        self.prev_wnd_proc = unsafe { transmute::<LongPtr, WNDPROC>(result) };
    }

    fn uninstall(&self) {
        if self.window_destroyed.get() {
            return;
        }
        let result = unsafe {
            SetWindowLongPtrW(
                self.hwnd,
                GWLP_WNDPROC,
                transmute::<WNDPROC, LongPtr>(self.prev_wnd_proc),
            )
        };
        if result == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        unsafe { RemovePropW(self.hwnd, PROP_NAME) }.unwrap();
    }
}

/// Uses [Win32 subclassing] to handle `WM_GETOBJECT` messages on a window
/// that provides no other way of adding custom message handlers.
///
/// [Win32 subclassing]: https://docs.microsoft.com/en-us/windows/win32/controls/subclassing-overview
pub struct SubclassingAdapter(Box<SubclassImpl>);

impl SubclassingAdapter {
    pub fn new(
        hwnd: HWND,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
    ) -> Self {
        let uia_init_marker = UiaInitMarker::new();
        let adapter: LazyAdapter = Lazy::new(Box::new(move || {
            Adapter::new(hwnd, source(), action_handler, uia_init_marker)
        }));
        let mut r#impl = SubclassImpl::new(hwnd, adapter);
        r#impl.install();
        Self(r#impl)
    }

    /// Initialize the tree if it hasn't been initialized already, then apply
    /// the provided update.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let adapter = Lazy::force(&self.0.adapter);
        adapter.update(update)
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update_if_active(
        &self,
        update_factory: impl FnOnce() -> TreeUpdate,
    ) -> Option<QueuedEvents> {
        Lazy::get(&self.0.adapter).map(|adapter| adapter.update(update_factory()))
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        self.0.uninstall();
    }
}
