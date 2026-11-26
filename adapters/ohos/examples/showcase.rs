// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![cfg(target_env = "ohos")]

use std::{
    ffi::{CStr, c_char, c_void},
    ops::{Deref, DerefMut, Range},
    ptr::NonNull,
    sync::{Arc, Mutex, OnceLock},
};

use accesskit::{Action, ActionHandler, ActionRequest, ActivationHandler, Size, TreeUpdate, Vec2};
use accesskit_ohos::{Adapter, Provider};
use example_common::{
    Key, KeyEvent, KeyState, Modifiers, UiState,
    widgets::{ImeEdit, InputType, TextInputInfo},
};

// From OpenHarmony `<arkui/ui_input_event.h>` (`ArkUI_KeyEventType` & `ArkUI_KeyCode`).
const ARKUI_KEY_EVENT_DOWN: i32 = 0;
const ARKUI_KEY_EVENT_UP: i32 = 1;
const KEYCODE_DPAD_UP: i32 = 2012;
const KEYCODE_DPAD_DOWN: i32 = 2013;
const KEYCODE_DPAD_LEFT: i32 = 2014;
const KEYCODE_DPAD_RIGHT: i32 = 2015;
const KEYCODE_TAB: i32 = 2049;
const KEYCODE_SPACE: i32 = 2050;
const KEYCODE_ENTER: i32 = 2054;
const KEYCODE_DEL: i32 = 2055;
const KEYCODE_FORWARD_DEL: i32 = 2071;
const KEYCODE_MOVE_HOME: i32 = 2081;
const KEYCODE_MOVE_END: i32 = 2082;
const KEYCODE_NUMPAD_ENTER: i32 = 2119;

struct Ui {
    state: UiState,
    show_keyboard: bool,
}

impl Deref for Ui {
    type Target = UiState;

    fn deref(&self) -> &UiState {
        &self.state
    }
}

impl DerefMut for Ui {
    fn deref_mut(&mut self) -> &mut UiState {
        &mut self.state
    }
}

struct SharedAdapter(NonNull<Adapter<'static>>);

// SAFETY: `Adapter` internals are synchronized with `Arc<Mutex<State>>` and
// `ProviderRegistration` only unregisters from a process-wide `RwLock` registry.
unsafe impl Send for SharedAdapter {}
unsafe impl Sync for SharedAdapter {}

struct ShowcaseActivationHandler {
    ui: Arc<Mutex<Ui>>,
}

impl ActivationHandler for ShowcaseActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let mut ui = self.ui.lock().ok()?;
        Some(ui.build_tree_update())
    }
}

struct ShowcaseActionHandler {
    ui: Arc<Mutex<Ui>>,
    adapter: Arc<OnceLock<SharedAdapter>>,
}

impl ActionHandler for ShowcaseActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        {
            let Ok(mut ui) = self.ui.lock() else {
                return;
            };
            ui.do_action(&request);
            if request.action == Action::Click
                && ui
                    .focused_text_input()
                    .is_some_and(|input| input.id == request.target_node)
            {
                ui.show_keyboard = true;
            }
        }
        if let Some(shared) = self.adapter.get() {
            let adapter = unsafe { shared.0.as_ref() };
            let _ = adapter.update_if_active(|| {
                self.ui
                    .lock()
                    .expect("showcase UI mutex poisoned")
                    .build_tree_update()
            });
        }
    }
}

pub struct ShowcaseState {
    adapter: Box<Adapter<'static>>,
    ui: Arc<Mutex<Ui>>,
}

impl ShowcaseState {
    fn new(
        register: impl FnOnce(
            ShowcaseActivationHandler,
            ShowcaseActionHandler,
        ) -> accesskit_ohos::Result<Adapter<'static>>,
    ) -> accesskit_ohos::Result<Self> {
        let ui = Arc::new(Mutex::new(Ui {
            state: UiState::new(),
            show_keyboard: false,
        }));
        let shared_adapter = Arc::new(OnceLock::new());
        let activation_handler = ShowcaseActivationHandler {
            ui: Arc::clone(&ui),
        };
        let action_handler = ShowcaseActionHandler {
            ui: Arc::clone(&ui),
            adapter: Arc::clone(&shared_adapter),
        };
        let mut adapter = Box::new(register(activation_handler, action_handler)?);
        let ptr = NonNull::from(adapter.as_mut());
        let _ = shared_adapter.set(SharedAdapter(ptr));
        Ok(Self { adapter, ui })
    }

    fn update_tree(&self) {
        let _ = self.adapter.update_if_active(|| {
            self.ui
                .lock()
                .expect("showcase UI mutex poisoned")
                .build_tree_update()
        });
    }
}

fn selection(input: &TextInputInfo) -> Range<usize> {
    let (anchor, focus) = (input.selection_anchor, input.selection_focus);
    anchor.min(focus)..anchor.max(focus)
}

fn replace_text(input: &TextInputInfo, text: String, is_composing: bool) -> Vec<ImeEdit> {
    let range = input
        .composition
        .clone()
        .unwrap_or_else(|| selection(input));
    let inserted = range.start..range.start + text.len();
    let caret = inserted.end;
    vec![
        ImeEdit::Replace { range, text },
        ImeEdit::SetComposition(is_composing.then_some(inserted)),
        ImeEdit::SetSelection {
            anchor: caret,
            focus: caret,
        },
    ]
}

fn delete_surrounding_code_points(
    input: &TextInputInfo,
    before_count: usize,
    after_count: usize,
) -> Vec<ImeEdit> {
    let sel = selection(input);
    let kept = match &input.composition {
        Some(composition) => sel.start.min(composition.start)..sel.end.max(composition.end),
        None => sel,
    };
    let before_start = input.text[..kept.start]
        .char_indices()
        .rev()
        .take(before_count)
        .last()
        .map_or(kept.start, |(offset, _)| offset);
    let after_end = input.text[kept.end..]
        .char_indices()
        .nth(after_count)
        .map_or(input.text.len(), |(offset, _)| kept.end + offset);
    vec![
        ImeEdit::Replace {
            range: kept.end..after_end,
            text: String::new(),
        },
        ImeEdit::Replace {
            range: before_start..kept.start,
            text: String::new(),
        },
    ]
}

fn translate_key(key_code: i32, unicode_char: u32, key_state: KeyState) -> Option<Key> {
    match key_code {
        KEYCODE_ENTER | KEYCODE_NUMPAD_ENTER => Some(Key::Enter),
        KEYCODE_SPACE => Some(Key::Space),
        KEYCODE_TAB => Some(Key::Tab),
        KEYCODE_DEL => Some(Key::Backspace),
        KEYCODE_FORWARD_DEL => Some(Key::Delete),
        KEYCODE_DPAD_LEFT => Some(Key::Left),
        KEYCODE_DPAD_RIGHT => Some(Key::Right),
        KEYCODE_DPAD_UP => Some(Key::Up),
        KEYCODE_DPAD_DOWN => Some(Key::Down),
        KEYCODE_MOVE_HOME => Some(Key::Home),
        KEYCODE_MOVE_END => Some(Key::End),
        _ if key_state == KeyState::Pressed => {
            char::from_u32(unicode_char).and_then(Key::character)
        }
        _ => None,
    }
}

/// Create a showcase instance bound to an `ArkUI_AccessibilityProvider*`.
///
/// # Safety
///
/// `provider` must be a valid `ArkUI_AccessibilityProvider*` pointer that outlives
/// the returned showcase instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_create(
    provider: *mut c_void,
) -> *mut ShowcaseState {
    let Ok(provider) = (unsafe { Provider::from_raw(provider.cast()) }) else {
        return std::ptr::null_mut();
    };
    match ShowcaseState::new(|activation, action| Adapter::new(provider, activation, action)) {
        Ok(state) => Box::into_raw(Box::new(state)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Create a multi-instance showcase bound to an `ArkUI_AccessibilityProvider*` (API 15+).
///
/// # Safety
///
/// `provider` must be a valid `ArkUI_AccessibilityProvider*` pointer and
/// `instance_id` must be a valid NUL-terminated UTF-8 string.
#[cfg(feature = "api-15")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_create_with_instance(
    provider: *mut c_void,
    instance_id: *const c_char,
) -> *mut ShowcaseState {
    if instance_id.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(instance_id) = unsafe { CStr::from_ptr(instance_id) }.to_str() else {
        return std::ptr::null_mut();
    };
    let Ok(provider) = (unsafe { Provider::from_raw(provider.cast()) }) else {
        return std::ptr::null_mut();
    };
    match ShowcaseState::new(|activation, action| {
        Adapter::new_with_instance(provider, instance_id, activation, action)
    }) {
        Ok(state) => Box::into_raw(Box::new(state)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy a showcase instance created by `accesskit_ohos_showcase_create`.
///
/// # Safety
///
/// `handle` must be either null or a valid pointer returned by
/// `accesskit_ohos_showcase_create` / `accesskit_ohos_showcase_create_with_instance`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_destroy(handle: *mut ShowcaseState) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// Update the showcase viewport dimensions and scale factor.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_set_viewport(
    handle: *mut ShowcaseState,
    scale_factor: f64,
    safe_area_inset_x: f64,
    safe_area_inset_y: f64,
    safe_area_width: f64,
    safe_area_height: f64,
) {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return;
    };
    let scale_factor = if scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    {
        let Ok(mut ui) = state.ui.lock() else {
            return;
        };
        ui.set_viewport(
            scale_factor,
            Vec2::new(safe_area_inset_x, safe_area_inset_y),
            Size::new(safe_area_width, safe_area_height) / scale_factor,
        );
    }
    state.update_tree();
}

/// Update the host XComponent focus state.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_on_focus_change(
    handle: *mut ShowcaseState,
    is_focused: bool,
) {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return;
    };
    let _ = state.adapter.set_host_focus_state(is_focused);
}

/// Forward a hardware or virtual keyboard event to the showcase UI.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_on_key_event(
    handle: *mut ShowcaseState,
    action: i32,
    key_code: i32,
    unicode_char: u32,
    shift_pressed: bool,
    ctrl_pressed: bool,
) -> bool {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    let key_state = match action {
        ARKUI_KEY_EVENT_DOWN => KeyState::Pressed,
        ARKUI_KEY_EVENT_UP => KeyState::Released,
        _ => return false,
    };
    let Some(key) = translate_key(key_code, unicode_char, key_state) else {
        return false;
    };
    let handled = {
        let Ok(mut ui) = state.ui.lock() else {
            return false;
        };
        ui.handle_key(KeyEvent {
            key,
            state: key_state,
            modifiers: Modifiers {
                shift: shift_pressed,
                control: ctrl_pressed,
            },
        })
    };
    state.update_tree();
    handled
}

/// Commit or compose text from the OpenHarmony input method into the focused text input.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState` and `text` must be a valid
/// NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_on_ime_text(
    handle: *mut ShowcaseState,
    text: *const c_char,
    is_composing: bool,
) -> bool {
    if text.is_null() {
        return false;
    }
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    let text = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned();
    {
        let Ok(mut ui) = state.ui.lock() else {
            return false;
        };
        let Some(input) = ui.focused_text_input() else {
            return false;
        };
        for edit in replace_text(&input, text, is_composing) {
            ui.edit_focused_text(&edit);
        }
    }
    state.update_tree();
    true
}

/// Finish any active IME composition in the focused text input.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_on_ime_finish_composing(
    handle: *mut ShowcaseState,
) -> bool {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    {
        let Ok(mut ui) = state.ui.lock() else {
            return false;
        };
        if !ui.edit_focused_text(&ImeEdit::SetComposition(None)) {
            return false;
        }
    }
    state.update_tree();
    true
}

/// Delete code points before and/or after the current selection in the focused text input.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_on_ime_delete(
    handle: *mut ShowcaseState,
    before_count: usize,
    after_count: usize,
) -> bool {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    {
        let Ok(mut ui) = state.ui.lock() else {
            return false;
        };
        let Some(input) = ui.focused_text_input() else {
            return false;
        };
        for edit in delete_surrounding_code_points(&input, before_count, after_count) {
            ui.edit_focused_text(&edit);
        }
    }
    state.update_tree();
    true
}

/// Submit the focused text input (or advance focus on Next/Previous).
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_on_ime_submit(handle: *mut ShowcaseState) -> bool {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    let handled = {
        let Ok(mut ui) = state.ui.lock() else {
            return false;
        };
        ui.edit_focused_text(&ImeEdit::Submit)
    };
    state.update_tree();
    handled
}

/// Process any due timed UI messages (such as clearing live-region announcements).
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_process_due_messages(
    handle: *mut ShowcaseState,
) -> bool {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    {
        let Ok(mut ui) = state.ui.lock() else {
            return false;
        };
        if !ui.process_due_messages() {
            return false;
        }
    }
    state.update_tree();
    true
}

/// Return the delay in milliseconds until the next scheduled UI message, or `-1` if none.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_next_message_delay_ms(
    handle: *mut ShowcaseState,
) -> i64 {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return -1;
    };
    let Ok(ui) = state.ui.lock() else {
        return -1;
    };
    ui.time_until_next_message()
        .map_or(-1, |delay| delay.as_nanos().div_ceil(1_000_000) as i64)
}

/// Return the focused text input type (`0` = none, `1` = text, `2` = email,
/// `3` = password, `4` = multiline).
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_focused_text_input_type(
    handle: *mut ShowcaseState,
) -> i32 {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return 0;
    };
    let Ok(ui) = state.ui.lock() else {
        return 0;
    };
    match ui.focused_text_input().map(|input| input.input_type) {
        None => 0,
        Some(InputType::Text) => 1,
        Some(InputType::Email) => 2,
        Some(InputType::Password) => 3,
        Some(InputType::Multiline) => 4,
    }
}

/// Take and clear the `show_keyboard` request flag set when a text input is clicked.
///
/// # Safety
///
/// `handle` must be a valid pointer to `ShowcaseState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accesskit_ohos_showcase_take_show_keyboard(
    handle: *mut ShowcaseState,
) -> bool {
    let Some(state) = (unsafe { handle.as_ref() }) else {
        return false;
    };
    let Ok(mut ui) = state.ui.lock() else {
        return false;
    };
    std::mem::take(&mut ui.show_keyboard)
}
