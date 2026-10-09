// Based on the create_window sample in windows-samples-rs.

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};
use accesskit_windows::Adapter;
use example_common::{Key, KeyEvent, KeyState, Modifiers, Renderer, UiState, WINDOW_TITLE};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, Win32WindowHandle, WindowHandle,
};
use std::{
    cell::RefCell,
    num::NonZeroIsize,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::*,
};

static WINDOW_CLASS_ATOM: LazyLock<u16> = LazyLock::new(|| {
    let class_name = w!("AccessKitTest");

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
        panic!("{}", Error::from_thread());
    }
    atom
});

const ACTION_REQUEST_MSG: u32 = WM_USER;

const ANNOUNCEMENT_TIMER_ID: usize = 1;

struct Ui(UiState);

impl ActivationHandler for Ui {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.0.build_tree_update())
    }
}

impl Deref for Ui {
    type Target = UiState;

    fn deref(&self) -> &UiState {
        &self.0
    }
}

impl DerefMut for Ui {
    fn deref_mut(&mut self) -> &mut UiState {
        &mut self.0
    }
}

#[derive(Clone)]
struct RenderTarget(HWND);

impl HasWindowHandle for RenderTarget {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        let hwnd = NonZeroIsize::new(self.0.0 as isize).unwrap();
        // SAFETY: The window outlives this target, which the window itself
        // owns, and it belongs to the thread that draws to it.
        Ok(unsafe { WindowHandle::borrow_raw(Win32WindowHandle::new(hwnd).into()) })
    }
}

impl HasDisplayHandle for RenderTarget {
    fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::windows())
    }
}

struct WindowState {
    adapter: RefCell<Adapter>,
    ui: RefCell<Ui>,
    renderer: RefCell<Renderer<RenderTarget>>,
}

impl WindowState {
    fn update_accessibility_tree(&self) {
        let mut adapter = self.adapter.borrow_mut();
        let mut ui = self.ui.borrow_mut();
        if let Some(events) = adapter.update_if_active(|| ui.build_tree_update()) {
            drop(ui);
            drop(adapter);
            events.raise();
        }
    }

    fn after_input(&self, window: HWND) {
        self.update_accessibility_tree();
        let Some(delay) = self.ui.borrow().time_until_announcement() else {
            return;
        };
        let timer = unsafe {
            SetTimer(
                Some(window),
                ANNOUNCEMENT_TIMER_ID,
                delay.as_millis() as u32,
                None,
            )
        };
        if timer == 0 {
            panic!("{}", Error::from_thread());
        }
    }

    fn flush_announcement(&self, window: HWND) {
        let _ = unsafe { KillTimer(Some(window), ANNOUNCEMENT_TIMER_ID) };
        if self.ui.borrow_mut().flush_announcement() {
            self.update_accessibility_tree();
        }
    }
}

fn modifiers() -> Modifiers {
    Modifiers {
        shift: unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0,
    }
}

fn translate_key(key: VIRTUAL_KEY) -> Option<Key> {
    match key {
        VK_RETURN => Some(Key::Enter),
        VK_SPACE => Some(Key::Space),
        VK_TAB => Some(Key::Tab),
        _ => None,
    }
}

unsafe fn get_window_state(window: HWND) -> *const WindowState {
    unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) as _ }
}

fn update_window_focus_state(window: HWND, is_focused: bool) {
    let state = unsafe { &*get_window_state(window) };
    let mut adapter = state.adapter.borrow_mut();
    if let Some(events) = adapter.update_window_focus_state(is_focused) {
        drop(adapter);
        events.raise();
    }
}

struct SimpleActionHandler {
    window: HWND,
}

unsafe impl Send for SimpleActionHandler {}
unsafe impl Sync for SimpleActionHandler {}

impl ActionHandler for SimpleActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let request = Box::into_raw(Box::new(request));
        unsafe {
            PostMessageW(
                Some(self.window),
                ACTION_REQUEST_MSG,
                WPARAM(0),
                LPARAM(request as _),
            )
        }
        .unwrap();
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let adapter = Adapter::new(window, false, SimpleActionHandler { window });
            let state = Box::new(WindowState {
                adapter: RefCell::new(adapter),
                ui: RefCell::new(Ui(UiState::new())),
                renderer: RefCell::new(Renderer::new(RenderTarget(window))),
            });
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as _) };
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        WM_PAINT => {
            let state = unsafe { &*get_window_state(window) };
            let mut rect = RECT::default();
            unsafe { GetClientRect(window, &mut rect) }.unwrap();
            state.renderer.borrow_mut().draw(
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            );
            unsafe { ValidateRect(Some(window), None) }.unwrap();
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
            let mut ui = state.ui.borrow_mut();
            let result = adapter.handle_wm_getobject(wparam, lparam, &mut *ui);
            drop(ui);
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
        WM_KEYDOWN | WM_KEYUP => {
            let Some(key) = translate_key(VIRTUAL_KEY(wparam.0 as u16)) else {
                return unsafe { DefWindowProcW(window, message, wparam, lparam) };
            };
            let key_state = if message == WM_KEYDOWN {
                KeyState::Pressed
            } else {
                KeyState::Released
            };
            let state = unsafe { &*get_window_state(window) };
            state.ui.borrow_mut().handle_key(KeyEvent {
                key,
                state: key_state,
                modifiers: modifiers(),
            });
            state.after_input(window);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 != ANNOUNCEMENT_TIMER_ID {
                return unsafe { DefWindowProcW(window, message, wparam, lparam) };
            }
            let state = unsafe { &*get_window_state(window) };
            state.flush_announcement(window);
            LRESULT(0)
        }
        ACTION_REQUEST_MSG => {
            // SAFETY: The action handler boxed this request and posted it
            // here, and nothing else handles this message.
            let request = unsafe { Box::from_raw(lparam.0 as *mut ActionRequest) };
            let state = unsafe { &*get_window_state(window) };
            state.ui.borrow_mut().do_action(&request);
            state.after_input(window);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn create_window(title: &str) -> Result<HWND> {
    let module = HINSTANCE::from(unsafe { GetModuleHandleW(None)? });

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
            Some(module),
            None,
        )?
    };
    if window.is_invalid() {
        return Err(Error::from_thread());
    }

    Ok(window)
}

fn main() -> Result<()> {
    example_common::print_instructions();

    let window = create_window(WINDOW_TITLE)?;
    let _ = unsafe { ShowWindow(window, SW_SHOW) };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.into() {
        let _ = unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }

    Ok(())
}
