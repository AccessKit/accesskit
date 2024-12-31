// Based on the create_window sample in windows-samples-rs.

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};
use accesskit_demo_lib::{Key, WindowState as AccessKitWindowState};
use accesskit_windows::Adapter;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};

static WINDOW_CLASS_ATOM: Lazy<u16> = Lazy::new(|| {
    let class_name = w!("AccessKitDemo");

    let wc = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap(),
        hInstance: unsafe { GetModuleHandleW(None) }.unwrap().into(),
        lpszClassName: class_name,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassW(&wc) };
    if atom == 0 {
        panic!("{}", Error::from_win32());
    }
    atom
});

const ACTION_REQUEST_MSG: u32 = WM_USER;

struct InnerWindowState(AccessKitWindowState);

impl ActivationHandler for InnerWindowState {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.0.build_initial_tree())
    }
}

struct WindowState {
    adapter: RefCell<Adapter>,
    inner_state: RefCell<InnerWindowState>,
}

impl WindowState {
    fn key_pressed(&self, key: Key) {
        self.inner_state.borrow_mut().0.key_pressed(key);
    }

    fn do_action(&self, request: ActionRequest) {
        self.inner_state.borrow_mut().0.do_action(&request);
    }

    fn update_accesskit_if_active(&self) {
        let mut adapter = self.adapter.borrow_mut();
        let inner_state = self.inner_state.borrow();
        if let Some(events) = adapter.update_if_active(|| inner_state.0.build_tree()) {
            drop(adapter);
            drop(inner_state);
            events.raise();
        }
    }
}

unsafe fn get_window_state(window: HWND) -> *const WindowState {
    GetWindowLongPtrW(window, GWLP_USERDATA) as _
}

fn update_window_focus_state(window: HWND, is_focused: bool) {
    let state = unsafe { &*get_window_state(window) };
    let mut adapter = state.adapter.borrow_mut();
    if let Some(events) = adapter.update_window_focus_state(is_focused) {
        drop(adapter);
        events.raise();
    }
}

struct WindowCreateParams(AccessKitWindowState);

struct SimpleActionHandler {
    window: HWND,
}

unsafe impl Send for SimpleActionHandler {}
unsafe impl Sync for SimpleActionHandler {}

impl ActionHandler for SimpleActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let request = Box::new(request);
        unsafe {
            PostMessageW(
                self.window,
                ACTION_REQUEST_MSG,
                WPARAM(0),
                LPARAM(Box::into_raw(request) as _),
            )
        }
        .unwrap();
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { &mut *(lparam.0 as *mut _) };
            let create_params: Box<WindowCreateParams> =
                unsafe { Box::from_raw(create_struct.lpCreateParams as _) };
            let WindowCreateParams(accesskit_state) = *create_params;
            let inner_state = RefCell::new(InnerWindowState(accesskit_state));
            let adapter = Adapter::new(window, false, SimpleActionHandler { window });
            let state = Box::new(WindowState {
                adapter: RefCell::new(adapter),
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
                drop(unsafe { Box::<WindowState>::from_raw(ptr as _) });
            }
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_GETOBJECT => {
            let state_ptr = unsafe { get_window_state(window) };
            if state_ptr.is_null() {
                // We need to be prepared to gracefully handle WM_GETOBJECT
                // while the window is being destroyed; this can happen if
                // the thread is using a COM STA.
                return unsafe { DefWindowProcW(window, message, wparam, lparam) };
            }
            let state = unsafe { &*state_ptr };
            let mut adapter = state.adapter.borrow_mut();
            let mut inner_state = state.inner_state.borrow_mut();
            let result = adapter.handle_wm_getobject(wparam, lparam, &mut *inner_state);
            drop(inner_state);
            drop(adapter);
            result.map_or_else(
                || unsafe { DefWindowProcW(window, message, wparam, lparam) },
                |result| result.into(),
            )
        }
        WM_SETFOCUS | WM_EXITMENULOOP | WM_EXITSIZEMOVE => {
            update_window_focus_state(window, true);
            LRESULT(0)
        }
        WM_KILLFOCUS | WM_ENTERMENULOOP | WM_ENTERSIZEMOVE => {
            update_window_focus_state(window, false);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = match VIRTUAL_KEY(wparam.0 as u16) {
                VK_LEFT => Some(Key::Left),
                VK_RIGHT => Some(Key::Right),
                VK_SPACE => Some(Key::Space),
                VK_TAB => Some(Key::Tab),
                _ => None,
            };
            if let Some(key) = key {
                let state = unsafe { &*get_window_state(window) };
                state.key_pressed(key);
                state.update_accesskit_if_active();
            }
            LRESULT(0)
        }
        ACTION_REQUEST_MSG => {
            let request = unsafe { Box::from_raw(lparam.0 as *mut ActionRequest) };
            let state = unsafe { &*get_window_state(window) };
            state.do_action(*request);
            state.update_accesskit_if_active();
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn create_window(accesskit_state: AccessKitWindowState) -> Result<HWND> {
    let title = HSTRING::from(accesskit_state.title());
    let create_params = Box::new(WindowCreateParams(accesskit_state));

    let window = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(*WINDOW_CLASS_ATOM as usize as _),
            &title,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            GetModuleHandleW(None).unwrap(),
            Some(Box::into_raw(create_params) as _),
        )?
    };
    if window.is_invalid() {
        return Err(Error::from_win32());
    }

    Ok(window)
}

fn main() -> Result<()> {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!("- [Space] 'presses' the button, adding static text in a live region announcing that it was pressed.");
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");

    let accesskit_state = AccessKitWindowState::default();
    let window = create_window(accesskit_state)?;
    let _ = unsafe { ShowWindow(window, SW_SHOW) };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, HWND::default(), 0, 0) }.into() {
        let _ = unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }

    Ok(())
}
