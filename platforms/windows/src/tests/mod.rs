// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_schema::{NodeId, TreeUpdate};
use lazy_static::lazy_static;
use parking_lot::{const_mutex, Condvar, Mutex};
use std::{sync::Arc, time::Duration};
use windows as Windows;
use windows::{
    runtime::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::{Com::*, LibraryLoader::GetModuleHandleW},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

use super::Manager;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

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
        focus: is_window_focused.then(|| window_state.focus),
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
        WM_SETFOCUS | WM_EXITMENULOOP | WM_EXITSIZEMOVE => {
            update_focus(window, true);
            LRESULT(0)
        }
        WM_KILLFOCUS | WM_ENTERMENULOOP | WM_ENTERSIZEMOVE => {
            update_focus(window, false);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn create_window(title: &str, initial_state: TreeUpdate, initial_focus: NodeId) -> Result<HWND> {
    let create_params = Box::new(WindowCreateParams(initial_state, initial_focus));

    let window = unsafe {
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
    if window.0 == 0 {
        return Err(Error::from_win32());
    }

    Ok(window)
}

pub(crate) struct Scope {
    pub(crate) uia: IUIAutomation,
    pub(crate) window: HWND,
}

impl Scope {
    pub(crate) fn show_and_focus_window(&self) {
        unsafe { ShowWindow(self.window, SW_SHOW) };
        unsafe { SetForegroundWindow(self.window) };
    }
}

// It's not safe to run these UI-related tests concurrently.
static MUTEX: Mutex<()> = const_mutex(());

pub(crate) fn scope<F>(
    window_title: &str,
    initial_state: TreeUpdate,
    initial_focus: NodeId,
    f: F,
) -> Result<()>
where
    F: FnOnce(&Scope) -> Result<()>,
{
    let _lock_guard = MUTEX.lock();

    unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED) }.unwrap();
    let _com_guard = scopeguard::guard((), |_| unsafe { CoUninitialize() });

    let uia: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }?;

    let window_mutex: Mutex<Option<HWND>> = Mutex::new(None);
    let window_cv = Condvar::new();

    crossbeam_utils::thread::scope(|thread_scope| {
        thread_scope.spawn(|_| {
            unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED) }.unwrap();
            let _com_guard = scopeguard::guard((), |_| unsafe { CoUninitialize() });

            let window = create_window(window_title, initial_state, initial_focus).unwrap();

            {
                let mut state = window_mutex.lock();
                *state = Some(window);
                window_cv.notify_one();
            }

            let mut message = MSG::default();
            while unsafe { GetMessageW(&mut message, HWND(0), 0, 0) }.into() {
                unsafe { TranslateMessage(&message) };
                unsafe { DispatchMessageW(&message) };
            }
        });

        let window = {
            let mut state = window_mutex.lock();
            if state.is_none() {
                window_cv.wait(&mut state);
            }
            state.take().unwrap()
        };

        let _window_guard = scopeguard::guard((), |_| {
            unsafe { PostMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0)) }.unwrap()
        });

        let s = Scope { uia, window };
        f(&s)
    })
    .unwrap()
}

pub(crate) struct ReceivedFocusEvent {
    mutex: Mutex<Option<IUIAutomationElement>>,
    cv: Condvar,
}

impl ReceivedFocusEvent {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            mutex: Mutex::new(None),
            cv: Condvar::new(),
        })
    }

    pub(crate) fn wait<F>(&self, f: F) -> IUIAutomationElement
    where
        F: Fn(&IUIAutomationElement) -> bool,
    {
        let mut received = self.mutex.lock();
        loop {
            if let Some(element) = received.take() {
                if f(&element) {
                    return element;
                }
            }
            let result = self.cv.wait_for(&mut received, DEFAULT_TIMEOUT);
            assert!(!result.timed_out());
        }
    }

    fn put(&self, element: IUIAutomationElement) {
        let mut received = self.mutex.lock();
        *received = Some(element);
        self.cv.notify_one();
    }
}

#[implement(Windows::Win32::UI::Accessibility::IUIAutomationFocusChangedEventHandler)]
pub(crate) struct FocusEventHandler {
    received: Arc<ReceivedFocusEvent>,
}

#[allow(non_snake_case)]
impl FocusEventHandler {
    #[allow(clippy::new_ret_no_self)] // it does return self, but wrapped
    pub(crate) fn new() -> (
        IUIAutomationFocusChangedEventHandler,
        Arc<ReceivedFocusEvent>,
    ) {
        let received = ReceivedFocusEvent::new();
        (
            Self {
                received: received.clone(),
            }
            .into(),
            received,
        )
    }

    fn HandleFocusChangedEvent(&self, sender: &Option<IUIAutomationElement>) -> Result<()> {
        self.received.put(sender.as_ref().unwrap().clone());
        Ok(())
    }
}

mod simple;
