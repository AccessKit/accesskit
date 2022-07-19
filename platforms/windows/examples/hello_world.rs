// Based on the create_window sample in windows-samples-rs.

use std::{cell::RefCell, convert::TryInto, mem::drop, num::NonZeroU128, rc::Rc};

use accesskit::kurbo::Rect;
use accesskit::{
    Action, ActionHandler, ActionRequest, DefaultActionVerb, Node, NodeId, Role, StringEncoding,
    Tree, TreeId, TreeUpdate,
};
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
        instance.unwrap()
    };

    static ref DEFAULT_CURSOR: HCURSOR = {
        let cursor = unsafe { LoadCursorW(None, IDC_ARROW) };
        cursor.unwrap()
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
            lpszClassName: PCWSTR(class_name_wsz.as_ptr() as _),
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

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

const BUTTON_1_RECT: Rect = Rect {
    x0: 20.0,
    y0: 20.0,
    x1: 100.0,
    y1: 60.0,
};

const BUTTON_2_RECT: Rect = Rect {
    x0: 20.0,
    y0: 60.0,
    x1: 100.0,
    y1: 100.0,
};

const SET_FOCUS_MSG: u32 = WM_USER;
const DO_DEFAULT_ACTION_MSG: u32 = WM_USER + 1;

fn make_button(id: NodeId, name: &str) -> Node {
    let rect = match id {
        BUTTON_1_ID => BUTTON_1_RECT,
        BUTTON_2_ID => BUTTON_2_RECT,
        _ => unreachable!(),
    };

    Node {
        bounds: Some(rect),
        name: Some(name.into()),
        focusable: true,
        default_action_verb: Some(DefaultActionVerb::Click),
        ..Node::new(id, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: vec![BUTTON_1_ID, BUTTON_2_ID],
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = make_button(BUTTON_1_ID, "Button 1");
    let button_2 = make_button(BUTTON_2_ID, "Button 2");
    TreeUpdate {
        nodes: vec![root, button_1, button_2],
        tree: Some(Tree::new(
            TreeId("test".into()),
            WINDOW_ID,
            StringEncoding::Utf8,
        )),
        focus: None,
    }
}

struct InnerWindowState {
    focus: NodeId,
    is_window_focused: bool,
}

struct WindowState {
    adapter: accesskit_windows::Adapter,
    inner_state: Rc<RefCell<InnerWindowState>>,
}

impl WindowState {
    fn press_button(&self, id: NodeId) {
        // This is a pretty hacky way of updating a node.
        // A real GUI framework would have a consistent way
        // of building a node from underlying data.
        // Also, this update isn't as lazy as it could be;
        // we force the AccessKit tree to be initialized.
        // This is expedient in this case, because that tree
        // is the only place where the state of the buttons
        // is stored. It's not a problem because we're really
        // only concerned with testing lazy updates in the context
        // of focus changes.
        let inner_state = self.inner_state.borrow();
        let is_window_focused = inner_state.is_window_focused;
        let focus = inner_state.focus;
        drop(inner_state);
        let name = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        let node = make_button(id, name);
        let update = TreeUpdate {
            nodes: vec![node],
            tree: None,
            focus: is_window_focused.then(|| focus),
        };
        let events = self.adapter.update(update);
        events.raise();
    }
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
    let events = window_state.adapter.update_if_active(|| TreeUpdate {
        nodes: vec![],
        tree: None,
        focus: is_window_focused.then(|| focus),
    });
    events.raise();
}

struct WindowCreateParams(TreeUpdate, NodeId);

pub struct SimpleActionHandler {
    window: HWND,
}

impl ActionHandler for SimpleActionHandler {
    fn do_action(&self, request: ActionRequest) {
        match request.action {
            Action::Focus => {
                unsafe {
                    PostMessageW(
                        self.window,
                        SET_FOCUS_MSG,
                        WPARAM(0),
                        LPARAM(request.target.0.get().try_into().unwrap()),
                    )
                };
            }
            Action::Default => {
                unsafe {
                    PostMessageW(
                        self.window,
                        DO_DEFAULT_ACTION_MSG,
                        WPARAM(0),
                        LPARAM(request.target.0.get().try_into().unwrap()),
                    )
                };
            }
            _ => (),
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message as u32 {
        WM_NCCREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { &mut *(lparam.0 as *mut _) };
            let create_params: Box<WindowCreateParams> =
                unsafe { Box::from_raw(create_struct.lpCreateParams as _) };
            let WindowCreateParams(initial_state, initial_focus) = *create_params;
            let inner_state = Rc::new(RefCell::new(InnerWindowState {
                focus: initial_focus,
                is_window_focused: false,
            }));
            let inner_state_for_tree_init = inner_state.clone();
            let state = Box::new(WindowState {
                adapter: accesskit_windows::Adapter::new(
                    window,
                    Box::new(move || {
                        let mut result = initial_state;
                        let state = inner_state_for_tree_init.borrow();
                        result.focus = state.is_window_focused.then(|| state.focus);
                        result
                    }),
                    Box::new(SimpleActionHandler { window }),
                ),
                inner_state,
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
            let result = window_state.adapter.handle_wm_getobject(wparam, lparam);
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
        WM_KEYDOWN => match VIRTUAL_KEY(wparam.0 as u16) {
            VK_TAB => {
                let window_state = unsafe { &*get_window_state(window) };
                let mut inner_state = window_state.inner_state.borrow_mut();
                inner_state.focus = if inner_state.focus == BUTTON_1_ID {
                    BUTTON_2_ID
                } else {
                    BUTTON_1_ID
                };
                drop(inner_state);
                update_focus(window, true);
                LRESULT(0)
            }
            VK_SPACE => {
                let window_state = unsafe { &*get_window_state(window) };
                let id = window_state.inner_state.borrow().focus;
                window_state.press_button(id);
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        },
        SET_FOCUS_MSG => {
            if let Some(id) = lparam.0.try_into().ok().and_then(NonZeroU128::new) {
                let id = NodeId(id);
                if id == BUTTON_1_ID || id == BUTTON_2_ID {
                    let window_state = unsafe { &*get_window_state(window) };
                    let mut inner_state = window_state.inner_state.borrow_mut();
                    inner_state.focus = id;
                    let is_window_focused = inner_state.is_window_focused;
                    drop(inner_state);
                    update_focus(window, is_window_focused);
                }
            }
            LRESULT(0)
        }
        DO_DEFAULT_ACTION_MSG => {
            if let Some(id) = lparam.0.try_into().ok().and_then(NonZeroU128::new) {
                let id = NodeId(id);
                if id == BUTTON_1_ID || id == BUTTON_2_ID {
                    let window_state = unsafe { &*get_window_state(window) };
                    window_state.press_button(id);
                }
            }
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
            PCWSTR(*WINDOW_CLASS_ATOM as usize as _),
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
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!("- [Space] 'presses' the button, permanently renaming it.");
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");

    let window = create_window(WINDOW_TITLE, get_initial_state(), INITIAL_FOCUS)?;
    unsafe { ShowWindow(window, SW_SHOW) };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, HWND(0), 0, 0) }.into() {
        unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }

    Ok(())
}
