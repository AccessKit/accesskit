// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_schema::{NodeId, TreeUpdate};
use lazy_static::lazy_static;
use windows::{
    runtime::*,
    Win32::{
        Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

use super::Manager;

lazy_static! {
    static ref WIN32_INSTANCE: HINSTANCE = {
        let instance = unsafe { GetModuleHandleW(None) };
        if instance.0 == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        instance
    };

    static ref DEFAULT_CURSOR: HCURSOR = {
        let cursor = unsafe { LoadCursorW(None, IDC_ARROW) };
        if cursor.0 == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        cursor
    };

    static ref WINDOW_CLASS_ATOM: u16 = {
        // The following is a combination of the implementation of
        // IntoParam<PWSTR> and the class registration function from winit.
        let class_name_wsz: Vec<_> = "AccessKitTest"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let wc = WNDCLASSW {
            hCursor: *DEFAULT_CURSOR,
            hInstance: *WIN32_INSTANCE,
            lpszClassName: PWSTR(class_name_wsz.as_ptr() as _),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassW(&wc) };
        if atom == 0 {
            let result: Result<()> = Err(Error::from_win32());
            result.unwrap();
        }
        atom
    };
}

struct WindowState {
    manager: Manager,
    focus: NodeId,
}

unsafe fn get_window_state(window: HWND) -> *mut WindowState {
    GetWindowLongPtrW(window, GWLP_USERDATA) as _
}

fn update_focus(window: HWND, is_window_focused: bool) {
    let window_state = unsafe { &*get_window_state(window) };
    let update = TreeUpdate {
        clear: None,
        nodes: vec![],
        tree: None,
        focus: is_window_focused.then(|| window_state.focus ),
    };
    window_state.manager.update(update);
}

struct WindowCreateParams(TreeUpdate, NodeId);

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message as u32 {
        WM_NCCREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { &mut *(lparam.0 as *mut _) };
            let create_params: Box<WindowCreateParams> =
                unsafe { Box::from_raw(create_struct.lpCreateParams as _) };
            let WindowCreateParams(initial_state, initial_focus) = *create_params;
            let manager = Manager::new(window, initial_state);
            let state = Box::new(WindowState {
                manager,
                focus: initial_focus,
            });
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as _) };
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        WM_PAINT => {
            unsafe { ValidateRect(window, std::ptr::null()) }.unwrap();
            LRESULT(0)
        }
        WM_DESTROY => {
            let ptr = unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, 0) };
            if ptr != 0 {
                let _dropped: Box<WindowState> = unsafe { Box::from_raw(ptr as _) };
            }
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_GETOBJECT => {
            let window_state = unsafe { &*get_window_state(window) };
            window_state.manager.handle_wm_getobject(wparam, lparam)
        }
        WM_SETFOCUS => {
            update_focus(window, true);
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            update_focus(window, false);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

pub(crate) struct Window {
    pub(crate) handle: HWND,
}

impl Window {
    pub(crate) fn new(
        title: &str,
        initial_state: TreeUpdate,
        initial_focus: NodeId,
    ) -> Result<Self> {
        let create_params = Box::new(WindowCreateParams(initial_state, initial_focus));

        let handle = unsafe {
            CreateWindowExW(
                Default::default(),
                PWSTR(*WINDOW_CLASS_ATOM as usize as _),
                title,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                *WIN32_INSTANCE,
                Box::into_raw(create_params) as _,
            )
        };
        if handle.0 == 0 {
            Err(Error::from_win32())?;
        }

        Ok(Self { handle })
    }
}
