// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, NodeId, TreeUpdate};
use once_cell::{sync::Lazy as SyncLazy, unsync::Lazy};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
use windows as Windows;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::{Com::*, LibraryLoader::GetModuleHandleW},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

use super::{Adapter, UiaInitMarker};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

static WINDOW_CLASS_ATOM: SyncLazy<u16> = SyncLazy::new(|| {
    let class_name = w!("AccessKitTest");

    let wc = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap(),
        hInstance: unsafe { GetModuleHandleW(None) }.unwrap(),
        lpszClassName: class_name.into(),
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
});

struct InnerWindowState {
    focus: NodeId,
    is_window_focused: bool,
}

type LazyAdapter = Lazy<Adapter, Box<dyn FnOnce() -> Adapter>>;

struct WindowState {
    adapter: LazyAdapter,
    inner_state: Rc<RefCell<InnerWindowState>>,
}

unsafe fn get_window_state(window: HWND) -> *const WindowState {
    GetWindowLongPtrW(window, GWLP_USERDATA) as _
}

fn update_focus(window: HWND, is_window_focused: bool) {
    let window_state = unsafe { &*get_window_state(window) };
    let mut inner_state = window_state.inner_state.borrow_mut();
    inner_state.is_window_focused = is_window_focused;
    let focus = inner_state.focus;
    drop(inner_state);
    if let Some(adapter) = Lazy::get(&window_state.adapter) {
        let events = adapter.update(TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: is_window_focused.then_some(focus),
        });
        events.raise();
    }
}

struct WindowCreateParams(TreeUpdate, NodeId, Box<dyn ActionHandler + Send + Sync>);

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { &mut *(lparam.0 as *mut _) };
            let create_params: Box<WindowCreateParams> =
                unsafe { Box::from_raw(create_struct.lpCreateParams as _) };
            let WindowCreateParams(initial_state, initial_focus, action_handler) = *create_params;
            let inner_state = Rc::new(RefCell::new(InnerWindowState {
                focus: initial_focus,
                is_window_focused: false,
            }));
            let inner_state_for_tree_init = inner_state.clone();
            let uia_init_marker = UiaInitMarker::new();
            let state = Box::new(WindowState {
                adapter: Lazy::new(Box::new(move || {
                    let mut initial_tree = initial_state;
                    let state = inner_state_for_tree_init.borrow();
                    initial_tree.focus = state.is_window_focused.then_some(state.focus);
                    Adapter::new(window, initial_tree, action_handler, uia_init_marker)
                })),
                inner_state,
            });
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as _) };
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        WM_PAINT => {
            unsafe { ValidateRect(window, None) }.unwrap();
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
            let window_state = unsafe { get_window_state(window) };
            if window_state.is_null() {
                // We need to be prepared to gracefully handle WM_GETOBJECT
                // while the window is being destroyed; this can happen if
                // the thread is using a COM STA.
                return unsafe { DefWindowProcW(window, message, wparam, lparam) };
            }
            let window_state = unsafe { &*window_state };
            let adapter = Lazy::force(&window_state.adapter);
            let result = adapter.handle_wm_getobject(wparam, lparam);
            result.map_or_else(
                || unsafe { DefWindowProcW(window, message, wparam, lparam) },
                |result| result.into(),
            )
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

fn create_window(
    title: &str,
    initial_state: TreeUpdate,
    initial_focus: NodeId,
    action_handler: Box<dyn ActionHandler + Send + Sync>,
) -> Result<HWND> {
    let create_params = Box::new(WindowCreateParams(
        initial_state,
        initial_focus,
        action_handler,
    ));

    let window = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(*WINDOW_CLASS_ATOM as usize as _),
            &HSTRING::from(title),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            GetModuleHandleW(None).unwrap(),
            Some(Box::into_raw(create_params) as _),
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
static MUTEX: Mutex<()> = Mutex::new(());

pub(crate) fn scope<F>(
    window_title: &str,
    initial_state: TreeUpdate,
    initial_focus: NodeId,
    action_handler: Box<dyn ActionHandler + Send + Sync>,
    f: F,
) -> Result<()>
where
    F: FnOnce(&Scope) -> Result<()>,
{
    let _lock_guard = MUTEX.lock().unwrap();

    let window_mutex: Mutex<Option<HWND>> = Mutex::new(None);
    let window_cv = Condvar::new();

    thread::scope(|thread_scope| {
        thread_scope.spawn(|| {
            // We explicitly don't want to initialize COM on the provider thread,
            // because we want to make sure that the provider side of UIA works
            // even if COM is never initialized on the provider thread
            // (as is the case if the window is never shown), or if COM is
            // initialized after the window is shown (as is the case,
            // at least on some Windows 10 machines, due to IME support).

            let window =
                create_window(window_title, initial_state, initial_focus, action_handler).unwrap();

            {
                let mut state = window_mutex.lock().unwrap();
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
            let state = window_mutex.lock().unwrap();
            let mut state = if state.is_none() {
                window_cv.wait(state).unwrap()
            } else {
                state
            };
            state.take().unwrap()
        };

        let _window_guard = scopeguard::guard((), |_| {
            unsafe { PostMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0)) }.unwrap()
        });

        // We must initialize COM before creating the UIA client. The MTA option
        // is cleaner by far, especially when we want to wait for a UIA event
        // handler to be called, and there's no reason not to use it here.
        // Note that we don't initialize COM this way on the provider thread,
        // as explained above. It's also important that we let the provider
        // thread do its forced initialization of UIA, in an environment
        // where COM has not been initialized, before we create the UIA client,
        // which also triggers UIA initialization, in a thread where COM
        // _has_ been initialized. This way, we ensure that the provider side
        // of UIA works even if it is set up in an environment where COM
        // has not been initialized, and that this sequence of events
        // doesn't prevent the UIA client from working.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.unwrap();
        let _com_guard = scopeguard::guard((), |_| unsafe { CoUninitialize() });

        let uia: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }?;

        let s = Scope { uia, window };
        f(&s)
    })
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
        let mut received = self.mutex.lock().unwrap();
        loop {
            if let Some(element) = received.take() {
                if f(&element) {
                    return element;
                }
            }
            let (lock, result) = self.cv.wait_timeout(received, DEFAULT_TIMEOUT).unwrap();
            assert!(!result.timed_out());
            received = lock;
        }
    }

    fn put(&self, element: IUIAutomationElement) {
        let mut received = self.mutex.lock().unwrap();
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
}

impl IUIAutomationFocusChangedEventHandler_Impl for FocusEventHandler {
    fn HandleFocusChangedEvent(&self, sender: Option<&IUIAutomationElement>) -> Result<()> {
        self.received.put(sender.unwrap().clone());
        Ok(())
    }
}

mod simple;
mod subclassed;
