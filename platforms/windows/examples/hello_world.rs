// Based on the create_window sample in windows-samples-rs.

use std::{cell::Cell, num::NonZeroU64};

use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use lazy_static::lazy_static;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};

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

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

fn make_button(id: NodeId, name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(id, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = make_button(BUTTON_1_ID, "Button 1");
    let button_2 = make_button(BUTTON_2_ID, "Button 2");
    TreeUpdate {
        clear: None,
        nodes: vec![root, button_1, button_2],
        tree: Some(Tree::new(
            TreeId("test".into()),
            WINDOW_ID,
            StringEncoding::Utf8,
        )),
        focus: None,
    }
}

struct WindowState {
    manager: accesskit_windows::Manager,
    focus: Cell<NodeId>,
}

unsafe fn get_window_state(window: HWND) -> *const WindowState {
    GetWindowLongPtrW(window, GWLP_USERDATA) as _
}

fn update_focus(window: HWND, is_window_focused: bool) {
    let window_state = unsafe { &*get_window_state(window) };
    let update = TreeUpdate {
        clear: None,
        nodes: vec![],
        tree: None,
        focus: is_window_focused.then(|| window_state.focus.get()),
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
            let manager = accesskit_windows::Manager::new(window, initial_state);
            let state = Box::new(WindowState {
                manager,
                focus: Cell::new(initial_focus),
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
            let window_state = unsafe { get_window_state(window) };
            if window_state.is_null() {
                // We need to be prepared to gracefully handle WM_GETOBJECT
                // while the window is being destroyed; this can happen if
                // the thread is using a COM STA.
                return unsafe { DefWindowProcW(window, message, wparam, lparam) };
            }
            let window_state = unsafe { &*window_state };
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
        WM_KEYDOWN => match VIRTUAL_KEY(wparam.0 as u16) {
            VK_TAB => {
                let window_state = unsafe { &*get_window_state(window) };
                window_state
                    .focus
                    .set(if window_state.focus.get() == BUTTON_1_ID {
                        BUTTON_2_ID
                    } else {
                        BUTTON_1_ID
                    });
                update_focus(window, true);
                LRESULT(0)
            }
            VK_SPACE => {
                let window_state = unsafe { &*get_window_state(window) };
                // This is a pretty hacky way of updating a node.
                // A real GUI framework would have a consistent way
                // of building a node from underlying data.
                let focus = window_state.focus.get();
                let node = if focus == BUTTON_1_ID {
                    make_button(BUTTON_1_ID, "You pressed button 1")
                } else {
                    make_button(BUTTON_2_ID, "You pressed button 2")
                };
                let update = TreeUpdate {
                    clear: None,
                    nodes: vec![node],
                    tree: None,
                    focus: Some(focus),
                };
                window_state.manager.update(update);
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        },
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

fn main() -> Result<()> {
    let window = create_window(WINDOW_TITLE, get_initial_state(), INITIAL_FOCUS)?;
    unsafe { ShowWindow(window, SW_SHOW) };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, HWND(0), 0, 0) }.into() {
        unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }

    Ok(())
}
