// Based on the create_window sample in windows-samples-rs.

use std::num::NonZeroU64;

use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use accesskit_windows_bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::ValidateRect, System::Com::*,
    System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*,
};
use windows::*;

const NODE_ID_1: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });

fn get_initial_state() -> TreeUpdate {
    let mut args = std::env::args();
    let _arg0 = args.next().unwrap();
    if let Some(initial_state_filename) = args.next() {
        let initial_state_str = std::fs::read_to_string(&initial_state_filename).unwrap();
        serde_json::from_str(&initial_state_str).unwrap()
    } else {
        let root = Node {
            name: Some("Hello world".into()),
            ..Node::new(NODE_ID_1, Role::Window)
        };
        let tree = Tree::new(TreeId("test".into()), StringEncoding::Utf8);
        TreeUpdate {
            clear: None,
            nodes: vec![root],
            tree: Some(tree),
            root: Some(NODE_ID_1),
        }
    }
}

// This simple example doesn't have a way of associating data with an HWND.
// So we'll just use a global variable for the AccessKit manager.
static mut MANAGER: Option<accesskit_windows::Manager> = None;

fn main() -> Result<()> {
    let initial_state = get_initial_state();

    unsafe {
        // Workaround for #37
        CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;

        let instance = GetModuleHandleA(None);
        debug_assert!(instance.0 != 0);

        let window_class = "window";

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW),
            hInstance: instance,
            lpszClassName: PSTR(b"window\0".as_ptr() as _),

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExA(
            Default::default(),
            window_class,
            "Hello world",
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
                if let Some(manager) = MANAGER.as_ref() {
                    manager.handle_wm_getobject(wparam, lparam)
                } else {
                    println!("no AccessKit manager yet");
                    DefWindowProcA(window, message, wparam, lparam)
                }
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
