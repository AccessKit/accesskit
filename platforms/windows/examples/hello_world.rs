// Based on the create_window sample in windows-samples-rs.

use std::num::NonZeroU64;

use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use windows::{
    runtime::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::Com::*,
        System::LibraryLoader::GetModuleHandleA,
        UI::{KeyboardAndMouseInput::*, WindowsAndMessaging::*},
    },
};

const WINDOW_CLASS_NAME: &str = "AccessKitExample";
const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });

fn get_tree(is_window_focused: bool) -> Tree {
    Tree {
        focus: is_window_focused.then(|| unsafe { FOCUS }),
        ..Tree::new(TreeId("test".into()), StringEncoding::Utf8)
    }
}

fn get_button_1(name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(BUTTON_1_ID, Role::Button)
    }
}

fn get_button_2(name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(BUTTON_2_ID, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = get_button_1("Button 1");
    let button_2 = get_button_2("Button 2");
    TreeUpdate {
        clear: None,
        nodes: vec![root, button_1, button_2],
        tree: Some(get_tree(false)),
        root: Some(WINDOW_ID),
    }
}

// This simple example doesn't have a way of associating data with an HWND.
// So we'll just use global variables.
static mut MANAGER: Option<accesskit_windows::Manager> = None;
static mut FOCUS: NodeId = BUTTON_1_ID;

fn main() -> Result<()> {
    let initial_state = get_initial_state();

    unsafe {
        // Workaround for #37
        CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;

        let instance = GetModuleHandleA(None);
        debug_assert!(instance.0 != 0);

        let class_name_sz: Vec<_> = WINDOW_CLASS_NAME
            .bytes()
            .chain(std::iter::once(0))
            .collect();

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW),
            hInstance: instance,
            lpszClassName: PSTR(class_name_sz.as_ptr() as _),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExA(
            Default::default(),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            std::ptr::null_mut(),
        );

        let manager = accesskit_windows::Manager::new(hwnd, initial_state);
        MANAGER = Some(manager);

        ShowWindow(hwnd, SW_SHOW);

        let mut message = MSG::default();
        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&mut message);
        }

        Ok(())
    }
}

fn update_focus(is_window_focused: bool) {
    if let Some(manager) = unsafe { MANAGER.as_ref() } {
        let tree = get_tree(is_window_focused);
        let update = TreeUpdate {
            clear: None,
            nodes: vec![],
            tree: Some(tree),
            root: None,
        };
        manager.update(update);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, std::ptr::null());
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_GETOBJECT => {
                let manager = MANAGER.as_ref().unwrap();
                manager.handle_wm_getobject(wparam, lparam)
            }
            WM_SETFOCUS => {
                update_focus(true);
                LRESULT(0)
            }
            WM_KILLFOCUS => {
                update_focus(false);
                LRESULT(0)
            }
            WM_KEYDOWN => match VIRTUAL_KEY(wparam.0 as u16) {
                VK_TAB => {
                    FOCUS = if FOCUS == BUTTON_1_ID {
                        BUTTON_2_ID
                    } else {
                        BUTTON_1_ID
                    };
                    update_focus(true);
                    LRESULT(0)
                }
                VK_SPACE => {
                    if let Some(manager) = MANAGER.as_ref() {
                        // This is a pretty hacky way of updating a node.
                        // A real GUI framework would have a consistent way
                        // of building a node from underlying data.
                        let node = if FOCUS == BUTTON_1_ID {
                            get_button_1("You pressed button 1")
                        } else {
                            get_button_2("You pressed button 2")
                        };
                        let update = TreeUpdate {
                            clear: None,
                            nodes: vec![node],
                            tree: None,
                            root: None,
                        };
                        manager.update(update);
                    }
                    LRESULT(0)
                }
                _ => DefWindowProcA(window, message, wparam, lparam),
            },
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
