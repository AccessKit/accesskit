// Based on the create_window sample in windows-samples-rs.

use std::mem::MaybeUninit;

use accesskit_schema::TreeUpdate;
use accesskit_windows_bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::*,
};
use windows::*;

fn get_initial_state() -> TreeUpdate {
    let mut args = std::env::args();
    let _arg0 = args.next().unwrap();
    let initial_state_filename = args.next().unwrap();
    let initial_state_str = std::fs::read_to_string(&initial_state_filename).unwrap();
    serde_json::from_str(&initial_state_str).unwrap()
}

// This simple example doesn't have a way of associating data with an HWND.
// So we'll just use a global variable for the AccessKit manager.
static mut MANAGER: MaybeUninit<accesskit_windows::Manager> = MaybeUninit::uninit();

fn main() -> Result<()> {
    let initial_state = get_initial_state();

    unsafe {
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
            "This is a sample window",
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
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
        MANAGER.write(manager);

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
                println!("WM_GETOBJECT");
                let manager = MANAGER.assume_init_ref();
                manager.handle_wm_getobject(wparam, lparam)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
