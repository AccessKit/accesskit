// Based on the create_window sample in windows-samples-rs.

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Rect, Role, Tree,
    TreeId, TreeUpdate,
};
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
    let class_name = w!("AccessKitTreeDemo");

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

const WINDOW_TITLE: &str = "AccessKit tree demo";

const WINDOW_ID: NodeId = NodeId(0);
const TREE_ID: NodeId = NodeId(1);
const FRUITS_ID: NodeId = NodeId(2);
const APPLE_ID: NodeId = NodeId(3);
const ORANGE_ID: NodeId = NodeId(4);
const VEGETABLES_ID: NodeId = NodeId(5);
const CARROT_ID: NodeId = NodeId(6);
const ABOUT_ID: NodeId = NodeId(7);
const INITIAL_FOCUS: NodeId = FRUITS_ID;

const TREE_RECT: Rect = Rect {
    x0: 20.0,
    y0: 20.0,
    x1: 280.0,
    y1: 220.0,
};

const SET_FOCUS_MSG: u32 = WM_USER;
const EXPAND_MSG: u32 = WM_USER + 1;
const COLLAPSE_MSG: u32 = WM_USER + 2;

fn item_rect(row: usize, level: usize) -> Rect {
    let top = 24.0 + (row as f64) * 28.0;
    let left = 24.0 + (level as f64) * 24.0;
    Rect {
        x0: left,
        y0: top,
        x1: 260.0,
        y1: top + 24.0,
    }
}

fn label_for(id: NodeId) -> &'static str {
    match id {
        FRUITS_ID => "Fruits",
        APPLE_ID => "Apple",
        ORANGE_ID => "Orange",
        VEGETABLES_ID => "Vegetables",
        CARROT_ID => "Carrot",
        ABOUT_ID => "About this demo",
        _ => unreachable!(),
    }
}

struct InnerWindowState {
    focus: NodeId,
    fruits_expanded: bool,
    vegetables_expanded: bool,
}

impl InnerWindowState {
    fn visible_items(&self) -> Vec<NodeId> {
        let mut items = vec![FRUITS_ID];
        if self.fruits_expanded {
            items.push(APPLE_ID);
            items.push(ORANGE_ID);
        }
        items.push(VEGETABLES_ID);
        if self.vegetables_expanded {
            items.push(CARROT_ID);
        }
        items.push(ABOUT_ID);
        items
    }

    fn level_of(id: NodeId) -> usize {
        match id {
            APPLE_ID | ORANGE_ID | CARROT_ID => 1,
            FRUITS_ID | VEGETABLES_ID | ABOUT_ID => 0,
            _ => unreachable!(),
        }
    }

    fn parent_of(id: NodeId) -> Option<NodeId> {
        match id {
            APPLE_ID | ORANGE_ID => Some(FRUITS_ID),
            CARROT_ID => Some(VEGETABLES_ID),
            _ => None,
        }
    }

    fn first_child(&self, id: NodeId) -> Option<NodeId> {
        match id {
            FRUITS_ID if self.fruits_expanded => Some(APPLE_ID),
            VEGETABLES_ID if self.vegetables_expanded => Some(CARROT_ID),
            _ => None,
        }
    }

    fn position_and_size(id: NodeId) -> (usize, usize) {
        match id {
            FRUITS_ID => (0, 3),
            VEGETABLES_ID => (1, 3),
            ABOUT_ID => (2, 3),
            APPLE_ID => (0, 2),
            ORANGE_ID => (1, 2),
            CARROT_ID => (0, 1),
            _ => unreachable!(),
        }
    }

    fn is_expanded(&self, id: NodeId) -> Option<bool> {
        match id {
            FRUITS_ID => Some(self.fruits_expanded),
            VEGETABLES_ID => Some(self.vegetables_expanded),
            _ => None,
        }
    }

    fn set_focus(&mut self, focus: NodeId) {
        self.focus = focus;
    }

    fn set_expanded(&mut self, id: NodeId, expanded: bool) {
        match id {
            FRUITS_ID => self.fruits_expanded = expanded,
            VEGETABLES_ID => self.vegetables_expanded = expanded,
            _ => return,
        }
        if !self.visible_items().contains(&self.focus) {
            self.focus = id;
        }
    }

    fn move_focus(&mut self, direction: isize) {
        let items = self.visible_items();
        let current = items.iter().position(|id| *id == self.focus).unwrap_or(0) as isize;
        let len = items.len() as isize;
        let next = (current + direction).rem_euclid(len) as usize;
        self.focus = items[next];
    }

    fn activate_focused_item(&mut self) {
        if let Some(expanded) = self.is_expanded(self.focus) {
            self.set_expanded(self.focus, !expanded);
        }
    }

    fn move_focus_left(&mut self) {
        match self.is_expanded(self.focus) {
            Some(true) => self.set_expanded(self.focus, false),
            _ => {
                if let Some(parent) = Self::parent_of(self.focus) {
                    self.focus = parent;
                }
            }
        }
    }

    fn move_focus_right(&mut self) {
        match self.is_expanded(self.focus) {
            Some(false) => self.set_expanded(self.focus, true),
            Some(true) => {
                if let Some(child) = self.first_child(self.focus) {
                    self.focus = child;
                }
            }
            None => {}
        }
    }

    fn build_root(&self) -> Node {
        let mut node = Node::new(Role::Window);
        node.set_children(vec![TREE_ID]);
        node.set_language("en");
        node
    }

    fn build_tree(&self) -> Node {
        let mut node = Node::new(Role::Tree);
        node.set_bounds(TREE_RECT);
        node.set_label("Sample tree");
        node.set_children(vec![FRUITS_ID, VEGETABLES_ID, ABOUT_ID]);
        node
    }

    fn build_item(&self, id: NodeId, row: usize) -> Node {
        let (position, size) = Self::position_and_size(id);
        let mut node = Node::new(Role::TreeItem);
        node.set_label(label_for(id));
        node.set_bounds(item_rect(row, Self::level_of(id)));
        node.set_level(Self::level_of(id));
        node.set_position_in_set(position);
        node.set_size_of_set(size);
        node.set_selected(self.focus == id);
        node.add_action(Action::Focus);
        if let Some(expanded) = self.is_expanded(id) {
            node.set_expanded(expanded);
            node.add_action(Action::Expand);
            node.add_action(Action::Collapse);
        }
        match id {
            FRUITS_ID if self.fruits_expanded => node.set_children(vec![APPLE_ID, ORANGE_ID]),
            VEGETABLES_ID if self.vegetables_expanded => node.set_children(vec![CARROT_ID]),
            _ => {}
        }
        node
    }

    fn build_tree_update(&self) -> TreeUpdate {
        let mut nodes = vec![(WINDOW_ID, self.build_root()), (TREE_ID, self.build_tree())];
        for (row, id) in self.visible_items().into_iter().enumerate() {
            nodes.push((id, self.build_item(id, row)));
        }
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(WINDOW_ID)),
            tree_id: TreeId::ROOT,
            focus: self.focus,
        }
    }
}

impl ActivationHandler for InnerWindowState {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        println!("Initial tree requested");
        Some(self.build_tree_update())
    }
}

struct WindowState {
    adapter: RefCell<Adapter>,
    inner_state: RefCell<InnerWindowState>,
}

impl WindowState {
    fn update_accessibility(&self, mutate: impl FnOnce(&mut InnerWindowState)) {
        let mut inner_state = self.inner_state.borrow_mut();
        mutate(&mut inner_state);
        let update = inner_state.build_tree_update();
        let mut adapter = self.adapter.borrow_mut();
        if let Some(events) = adapter.update_if_active(|| update) {
            drop(adapter);
            drop(inner_state);
            events.raise();
        }
    }

    fn set_focus(&self, focus: NodeId) {
        self.update_accessibility(|state| state.set_focus(focus));
    }

    fn set_expanded(&self, id: NodeId, expanded: bool) {
        self.update_accessibility(|state| state.set_expanded(id, expanded));
    }

    fn move_focus(&self, direction: isize) {
        self.update_accessibility(|state| state.move_focus(direction));
    }

    fn move_focus_left(&self) {
        self.update_accessibility(InnerWindowState::move_focus_left);
    }

    fn move_focus_right(&self) {
        self.update_accessibility(InnerWindowState::move_focus_right);
    }

    fn activate_focused_item(&self) {
        self.update_accessibility(InnerWindowState::activate_focused_item);
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

struct WindowCreateParams(NodeId);

struct SimpleActionHandler {
    window: HWND,
}

unsafe impl Send for SimpleActionHandler {}
unsafe impl Sync for SimpleActionHandler {}

impl ActionHandler for SimpleActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let (message, node_id) = match request.action {
            Action::Focus => (SET_FOCUS_MSG, request.target_node.0),
            Action::Expand => (EXPAND_MSG, request.target_node.0),
            Action::Collapse => (COLLAPSE_MSG, request.target_node.0),
            _ => return,
        };
        unsafe { PostMessageW(Some(self.window), message, WPARAM(0), LPARAM(node_id as _)) }
            .unwrap();
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
                fruits_expanded: true,
                vegetables_expanded: false,
            });
            let adapter = Adapter::new(window, false, SimpleActionHandler { window });
            let state = Box::new(WindowState {
                adapter: RefCell::new(adapter),
                inner_state,
            });
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as _) };
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        WM_PAINT => {
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
            let state = unsafe { &*get_window_state(window) };
            match VIRTUAL_KEY(wparam.0 as u16) {
                VK_TAB | VK_DOWN => state.move_focus(1),
                VK_UP => state.move_focus(-1),
                VK_LEFT => state.move_focus_left(),
                VK_RIGHT => state.move_focus_right(),
                VK_SPACE | VK_RETURN => state.activate_focused_item(),
                _ => return unsafe { DefWindowProcW(window, message, wparam, lparam) },
            }
            LRESULT(0)
        }
        SET_FOCUS_MSG => {
            let id = NodeId(lparam.0 as _);
            let state = unsafe { &*get_window_state(window) };
            state.set_focus(id);
            LRESULT(0)
        }
        EXPAND_MSG => {
            let id = NodeId(lparam.0 as _);
            let state = unsafe { &*get_window_state(window) };
            state.set_expanded(id, true);
            LRESULT(0)
        }
        COLLAPSE_MSG => {
            let id = NodeId(lparam.0 as _);
            let state = unsafe { &*get_window_state(window) };
            state.set_expanded(id, false);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn create_window(title: &str, initial_focus: NodeId) -> Result<HWND> {
    let create_params = Box::new(WindowCreateParams(initial_focus));
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
            Some(Box::into_raw(create_params) as _),
        )?
    };
    if window.is_invalid() {
        return Err(Error::from_thread());
    }

    Ok(window)
}

fn main() -> Result<()> {
    println!("Keyboard commands:");
    println!("- [Up]/[Down] move between visible tree items.");
    println!("- [Right] expands a collapsed item or moves to its first child.");
    println!("- [Left] collapses an expanded item or moves to its parent.");
    println!("- [Space] or [Enter] toggles the focused item's expansion state.");
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");

    let window = create_window(WINDOW_TITLE, INITIAL_FOCUS)?;
    let _ = unsafe { ShowWindow(window, SW_SHOW) };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.into() {
        let _ = unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }

    Ok(())
}
