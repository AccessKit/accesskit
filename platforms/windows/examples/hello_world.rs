// Based on the create_window sample in windows-samples-rs.

use accesskit::{
    Action, ActionHandler, ActionRequest, DefaultActionVerb, Live, Node, NodeBuilder, NodeClassSet,
    NodeId, Rect, Role, Tree, TreeUpdate,
};
use accesskit_windows::UiaInitMarker;
use once_cell::{sync::Lazy, unsync::OnceCell};
use std::{cell::RefCell, convert::TryInto, num::NonZeroU128};
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
    let class_name = w!("AccessKitTest");

    let wc = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap(),
        hInstance: unsafe { GetModuleHandleW(None) }.unwrap(),
        lpszClassName: class_name,
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

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
const ANNOUNCEMENT_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
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

fn build_button(id: NodeId, name: &str, classes: &mut NodeClassSet) -> Node {
    let rect = match id {
        BUTTON_1_ID => BUTTON_1_RECT,
        BUTTON_2_ID => BUTTON_2_RECT,
        _ => unreachable!(),
    };

    let mut builder = NodeBuilder::new(Role::Button);
    builder.set_bounds(rect);
    builder.set_name(name);
    builder.add_action(Action::Focus);
    builder.set_default_action_verb(DefaultActionVerb::Click);
    builder.build(classes)
}

fn build_announcement(text: &str, classes: &mut NodeClassSet) -> Node {
    let mut builder = NodeBuilder::new(Role::StaticText);
    builder.set_name(text);
    builder.set_live(Live::Polite);
    builder.build(classes)
}

struct InnerWindowState {
    focus: NodeId,
    is_window_focused: bool,
    announcement: Option<String>,
    node_classes: NodeClassSet,
}

impl InnerWindowState {
    fn focus(&self) -> Option<NodeId> {
        self.is_window_focused.then_some(self.focus)
    }

    fn build_root(&mut self) -> Node {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        if self.announcement.is_some() {
            builder.push_child(ANNOUNCEMENT_ID);
        }
        builder.build(&mut self.node_classes)
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let button_1 = build_button(BUTTON_1_ID, "Button 1", &mut self.node_classes);
        let button_2 = build_button(BUTTON_2_ID, "Button 2", &mut self.node_classes);
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (BUTTON_1_ID, button_1),
                (BUTTON_2_ID, button_2),
            ],
            tree: Some(Tree::new(WINDOW_ID)),
            focus: self.focus(),
        };
        if let Some(announcement) = &self.announcement {
            result.nodes.push((
                ANNOUNCEMENT_ID,
                build_announcement(announcement, &mut self.node_classes),
            ));
        }
        result
    }
}

struct WindowState {
    uia_init_marker: UiaInitMarker,
    adapter: OnceCell<accesskit_windows::Adapter>,
    inner_state: RefCell<InnerWindowState>,
}

impl WindowState {
    fn get_or_init_accesskit_adapter(&self, window: HWND) -> &accesskit_windows::Adapter {
        self.adapter.get_or_init(|| {
            let mut inner_state = self.inner_state.borrow_mut();
            let initial_tree = inner_state.build_initial_tree();
            drop(inner_state);
            let action_handler = Box::new(SimpleActionHandler { window });
            accesskit_windows::Adapter::new(
                window,
                initial_tree,
                action_handler,
                self.uia_init_marker,
            )
        })
    }

    fn press_button(&self, id: NodeId) {
        let mut inner_state = self.inner_state.borrow_mut();
        let text = if id == BUTTON_1_ID {
            "You pressed button 1"
        } else {
            "You pressed button 2"
        };
        inner_state.announcement = Some(text.into());
        if let Some(adapter) = self.adapter.get() {
            let announcement = build_announcement(text, &mut inner_state.node_classes);
            let root = inner_state.build_root();
            let update = TreeUpdate {
                nodes: vec![(ANNOUNCEMENT_ID, announcement), (WINDOW_ID, root)],
                tree: None,
                focus: inner_state.focus(),
            };
            let events = adapter.update(update);
            events.raise();
        }
    }
}

unsafe fn get_window_state(window: HWND) -> *const WindowState {
    GetWindowLongPtrW(window, GWLP_USERDATA) as _
}

fn update_focus(window: HWND, is_window_focused: bool) {
    let window_state = unsafe { &*get_window_state(window) };
    let mut inner_state = window_state.inner_state.borrow_mut();
    inner_state.is_window_focused = is_window_focused;
    let focus = inner_state.focus();
    drop(inner_state);
    if let Some(adapter) = window_state.adapter.get() {
        let events = adapter.update(TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
        });
        events.raise();
    }
}

struct WindowCreateParams(NodeId);

struct SimpleActionHandler {
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
    match message {
        WM_NCCREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { &mut *(lparam.0 as *mut _) };
            let create_params: Box<WindowCreateParams> =
                unsafe { Box::from_raw(create_struct.lpCreateParams as _) };
            let WindowCreateParams(initial_focus) = *create_params;
            let inner_state = RefCell::new(InnerWindowState {
                focus: initial_focus,
                is_window_focused: false,
                announcement: None,
                node_classes: NodeClassSet::new(),
            });
            let state = Box::new(WindowState {
                uia_init_marker: UiaInitMarker::new(),
                adapter: OnceCell::new(),
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
            let adapter = window_state.get_or_init_accesskit_adapter(window);
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

fn create_window(title: &str, initial_focus: NodeId) -> Result<HWND> {
    let create_params = Box::new(WindowCreateParams(initial_focus));

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

fn main() -> Result<()> {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!("- [Space] 'presses' the button, adding static text in a live region announcing that it was pressed.");
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");

    let window = create_window(WINDOW_TITLE, INITIAL_FOCUS)?;
    unsafe { ShowWindow(window, SW_SHOW) };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, HWND(0), 0, 0) }.into() {
        unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }

    Ok(())
}
