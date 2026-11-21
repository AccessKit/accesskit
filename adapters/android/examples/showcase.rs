use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, NodeId, Size, TreeUpdate, Vec2,
};
use accesskit_android::{
    Adapter, PlatformAction, QueuedEvents,
    jni::{
        Env, JavaVM, NativeMethod,
        errors::Result,
        jni_sig, jni_str, native_method,
        objects::{JClass, JObject, JString},
        sys::{JNI_FALSE, JNI_TRUE, JNI_VERSION_1_6, jboolean, jfloat, jint, jlong},
    },
};
use example_common::{
    Key, KeyEvent, KeyState, Modifiers, UiState,
    widgets::{ImeEdit, InputType, TextInputInfo},
};
use std::{
    ffi::c_void,
    ops::{Deref, DerefMut, Range},
    sync::Mutex,
};

const ACTION_DOWN: jint = 0;
const ACTION_UP: jint = 1;
const KEYCODE_DPAD_UP: jint = 19;
const KEYCODE_DPAD_DOWN: jint = 20;
const KEYCODE_DPAD_LEFT: jint = 21;
const KEYCODE_DPAD_RIGHT: jint = 22;
const KEYCODE_TAB: jint = 61;
const KEYCODE_SPACE: jint = 62;
const KEYCODE_ENTER: jint = 66;
const KEYCODE_DEL: jint = 67;
const KEYCODE_FORWARD_DEL: jint = 112;
const KEYCODE_MOVE_HOME: jint = 122;
const KEYCODE_MOVE_END: jint = 123;

// From `android.text.InputType`.
const TYPE_CLASS_TEXT: jint = 0x1;
const TYPE_TEXT_VARIATION_EMAIL_ADDRESS: jint = 0x20;
const TYPE_TEXT_VARIATION_PASSWORD: jint = 0x80;
const TYPE_TEXT_FLAG_CAP_SENTENCES: jint = 0x4000;
const TYPE_TEXT_FLAG_MULTI_LINE: jint = 0x20000;

// From `android.view.inputmethod.EditorInfo`.
const IME_ACTION_NEXT: jint = 5;
const IME_ACTION_DONE: jint = 6;
const IME_ACTION_PREVIOUS: jint = 7;
const IME_FLAG_NO_PERSONALIZED_LEARNING: jint = 0x1000000;
const IME_FLAG_NO_FULLSCREEN: jint = 0x2000000;
const IME_FLAG_NO_EXTRACT_UI: jint = 0x10000000;
const IME_FLAG_NO_ENTER_ACTION: jint = 0x40000000;

// The requests of `ShowcaseView.ShowcaseInputConnection`.
const IME_TEXT_BEFORE_CURSOR: jint = 0;
const IME_TEXT_AFTER_CURSOR: jint = 1;
const IME_SELECTED_TEXT: jint = 2;
const IME_COMMIT_TEXT: jint = 0;
const IME_SET_COMPOSING_TEXT: jint = 1;
const IME_FINISH_COMPOSING_TEXT: jint = 2;
const IME_SET_COMPOSING_REGION: jint = 3;
const IME_DELETE_SURROUNDING_TEXT: jint = 4;
const IME_DELETE_SURROUNDING_TEXT_IN_CODE_POINTS: jint = 5;
const IME_SET_SELECTION: jint = 6;
const IME_PERFORM_EDITOR_ACTION: jint = 7;

struct Ui {
    state: UiState,
    /// Whether a text input was clicked, which is how TalkBack activates it,
    /// so that the on-screen keyboard should appear.
    show_keyboard: bool,
}

impl ActivationHandler for Ui {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.state.build_tree_update())
    }
}

impl ActionHandler for Ui {
    fn do_action(&mut self, request: ActionRequest) {
        self.state.do_action(&request);
        if request.action == Action::Click
            && self
                .state
                .focused_text_input()
                .is_some_and(|input| input.id == request.target_node)
        {
            self.show_keyboard = true;
        }
    }
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

/// What the input method knows about the focused text input.
#[derive(Default)]
struct InputMethodState {
    connection: Option<(NodeId, InputType, bool)>,
    text: String,
    selection: Option<(jint, jint, Option<Range<usize>>)>,
    batch_edit_depth: usize,
}

fn reported_selection(input: &TextInputInfo) -> (jint, jint, Option<Range<usize>>) {
    let utf16 = |offset| utf16_offset(&input.text, offset);
    (
        utf16(input.selection_anchor) as jint,
        utf16(input.selection_focus) as jint,
        input
            .composition
            .as_ref()
            .map(|composition| utf16(composition.start)..utf16(composition.end)),
    )
}

/// The position of the byte at `offset` in UTF-16 code units, which the
/// input method counts in.
fn utf16_offset(text: &str, offset: usize) -> usize {
    text[..offset].encode_utf16().count()
}

/// The byte offset of a position in UTF-16 code units, rounded down to a
/// code point boundary.
fn byte_offset(text: &str, utf16_offset: usize) -> usize {
    let mut utf16_length = 0;
    text.char_indices()
        .find(|(_, c)| {
            utf16_length += c.len_utf16();
            utf16_length > utf16_offset
        })
        .map_or(text.len(), |(offset, _)| offset)
}

fn selection(input: &TextInputInfo) -> Range<usize> {
    let (anchor, focus) = (input.selection_anchor, input.selection_focus);
    anchor.min(focus)..anchor.max(focus)
}

/// The edits of `commitText` and `setComposingText`, which replace the text
/// being composed, or the selection.
fn compose(
    input: &TextInputInfo,
    text: String,
    new_cursor_position: jint,
    is_composing: bool,
) -> Vec<ImeEdit> {
    let range = input
        .composition
        .clone()
        .unwrap_or_else(|| selection(input));
    let inserted = range.start..range.start + text.len();
    let mut new_text = input.text.clone();
    new_text.replace_range(range.clone(), &text);
    let caret = if new_cursor_position > 0 {
        utf16_offset(&new_text, inserted.end) + new_cursor_position as usize - 1
    } else {
        utf16_offset(&new_text, inserted.start)
            .saturating_sub(new_cursor_position.unsigned_abs() as usize)
    };
    let caret = byte_offset(&new_text, caret);
    vec![
        ImeEdit::Replace { range, text },
        ImeEdit::SetComposition(is_composing.then_some(inserted)),
        ImeEdit::SetSelection {
            anchor: caret,
            focus: caret,
        },
    ]
}

/// The edits deleting text around the selection, and around the text being
/// composed, which `BaseInputConnection` keeps.
fn delete_surrounding_text(
    input: &TextInputInfo,
    count_before: impl FnOnce(&str, usize) -> usize,
    count_after: impl FnOnce(&str, usize) -> usize,
) -> Vec<ImeEdit> {
    let selection = selection(input);
    let kept = match &input.composition {
        Some(composition) => {
            selection.start.min(composition.start)..selection.end.max(composition.end)
        }
        None => selection,
    };
    let before_start = count_before(&input.text, kept.start);
    let after_end = count_after(&input.text, kept.end);
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

/// The edits of an `InputConnection` request, whose positions are in UTF-16
/// code units.
fn input_connection_edits(
    input: &TextInputInfo,
    request: jint,
    text: String,
    a: jint,
    b: jint,
) -> Option<Vec<ImeEdit>> {
    let (a_count, b_count) = (a.max(0) as usize, b.max(0) as usize);
    let byte = |utf16_offset| byte_offset(&input.text, utf16_offset);
    let edits = match request {
        IME_COMMIT_TEXT => compose(input, text, a, false),
        IME_SET_COMPOSING_TEXT => compose(input, text, a, true),
        IME_FINISH_COMPOSING_TEXT => vec![ImeEdit::SetComposition(None)],
        IME_SET_COMPOSING_REGION => {
            vec![ImeEdit::SetComposition(Some(byte(a_count)..byte(b_count)))]
        }
        IME_DELETE_SURROUNDING_TEXT => delete_surrounding_text(
            input,
            |text, start| byte_offset(text, utf16_offset(text, start).saturating_sub(a_count)),
            |text, end| byte_offset(text, utf16_offset(text, end) + b_count),
        ),
        IME_DELETE_SURROUNDING_TEXT_IN_CODE_POINTS => delete_surrounding_text(
            input,
            |text, start| {
                text[..start]
                    .char_indices()
                    .rev()
                    .take(a_count)
                    .last()
                    .map_or(start, |(offset, _)| offset)
            },
            |text, end| {
                text[end..]
                    .char_indices()
                    .nth(b_count)
                    .map_or(text.len(), |(offset, _)| end + offset)
            },
        ),
        IME_SET_SELECTION => vec![ImeEdit::SetSelection {
            anchor: byte(a_count),
            focus: byte(b_count),
        }],
        _ => return None,
    };
    Some(edits)
}

#[derive(Default)]
struct DeferredCallbacks(Vec<DeferredCallback>);

type DeferredCallback = Box<dyn FnOnce(&mut Env, &JObject) -> Result<()>>;

impl DeferredCallbacks {
    fn push(&mut self, callback: impl FnOnce(&mut Env, &JObject) -> Result<()> + 'static) {
        self.0.push(Box::new(callback));
    }

    fn raise(&mut self, events: QueuedEvents) {
        self.push(move |env, host| events.raise(env, host));
    }

    fn run(self, env: &mut Env, host: &JObject) -> Result<()> {
        for callback in self.0 {
            callback(env, host)?;
        }
        Ok(())
    }
}

struct ViewState {
    adapter: Adapter,
    ui: Ui,
    input_method: InputMethodState,
}

impl ViewState {
    fn update_accessibility_tree(&mut self, deferred: &mut DeferredCallbacks) {
        let Self { adapter, ui, .. } = self;
        if let Some(events) = adapter.update_if_active(|| ui.build_tree_update()) {
            deferred.raise(events);
        }
    }

    fn after_input(&mut self, deferred: &mut DeferredCallbacks) {
        self.update_accessibility_tree(deferred);
        self.update_input_method(deferred);
        self.schedule_next_message(deferred);
    }

    /// Like [`ViewState::after_input`], for edits the input method made, so
    /// that it already knows the new text.
    fn after_ime_edit(&mut self, deferred: &mut DeferredCallbacks) {
        if let Some(input) = self.ui.focused_text_input() {
            self.input_method.text = input.text;
        }
        self.after_input(deferred);
    }

    /// Tells the input method about changes to the focused text input,
    /// restarting it when focus moves, the type of input changes, or the
    /// text changes without the input method editing it. Text typed with
    /// keys restarts it too, which `TextView` avoids, for simplicity.
    fn update_input_method(&mut self, deferred: &mut DeferredCallbacks) {
        let input = self.ui.focused_text_input();
        let connection = input
            .as_ref()
            .map(|input| (input.id, input.input_type, input.can_submit));
        let state = &mut self.input_method;
        let text_changed = input.as_ref().is_some_and(|input| input.text != state.text);
        if connection != state.connection || text_changed {
            state.connection = connection;
            state.text = input
                .as_ref()
                .map_or_else(String::new, |input| input.text.clone());
            state.selection = input.as_ref().map(reported_selection);
            state.batch_edit_depth = 0;
            let is_text_editor = connection.is_some();
            deferred.push(move |env, host| {
                env.call_method(
                    host,
                    jni_str!("restartInput"),
                    jni_sig!("(Z)V"),
                    &[is_text_editor.into()],
                )?;
                Ok(())
            });
        } else if let Some(input) = input.filter(|_| state.batch_edit_depth == 0) {
            let selection = reported_selection(&input);
            if state.selection.as_ref() != Some(&selection) {
                let (anchor, focus, composition) = selection.clone();
                let (composition_start, composition_end) =
                    composition.map_or((-1, -1), |range| (range.start as jint, range.end as jint));
                state.selection = Some(selection);
                deferred.push(move |env, host| {
                    env.call_method(
                        host,
                        jni_str!("updateSelection"),
                        jni_sig!("(IIII)V"),
                        &[
                            anchor.into(),
                            focus.into(),
                            composition_start.into(),
                            composition_end.into(),
                        ],
                    )?;
                    Ok(())
                });
            }
        }
        if std::mem::take(&mut self.ui.show_keyboard) {
            deferred.push(|env, host| {
                env.call_method(host, jni_str!("showKeyboard"), jni_sig!("()V"), &[])?;
                Ok(())
            });
        }
    }

    /// Starts a new connection with the input method, which knows nothing of
    /// the composition and batch edits of previous ones.
    fn connect_input_method(&mut self) -> Option<TextInputInfo> {
        if self.ui.focused_text_input()?.composition.is_some() {
            self.ui.edit_focused_text(&ImeEdit::SetComposition(None));
        }
        let input = self.ui.focused_text_input()?;
        let state = &mut self.input_method;
        state.text = input.text.clone();
        state.selection = Some(reported_selection(&input));
        state.batch_edit_depth = 0;
        Some(input)
    }

    fn schedule_next_message(&self, deferred: &mut DeferredCallbacks) {
        let Some(delay) = self.ui.time_until_next_message() else {
            return;
        };
        let delay = delay.as_nanos().div_ceil(1_000_000) as jlong;
        deferred.push(move |env, host| {
            env.call_method(
                host,
                jni_str!("scheduleDelayedMessage"),
                jni_sig!("(J)V"),
                &[delay.into()],
            )?;
            Ok(())
        });
    }
}

type ViewHandle = Mutex<ViewState>;

/// # Safety
///
/// `handle` must have been returned by `nativeCreate` and not yet passed
/// to `nativeDestroy`.
unsafe fn with_view_state<'local, T>(
    env: &mut Env<'local>,
    host: &JObject,
    handle: jlong,
    f: impl FnOnce(&mut Env<'local>, &mut ViewState, &mut DeferredCallbacks) -> Result<T>,
) -> Result<T> {
    let state = unsafe { &*(handle as *const ViewHandle) };
    let mut deferred = DeferredCallbacks::default();
    let mut guard = state.lock().unwrap();
    let result = f(env, &mut guard, &mut deferred);
    drop(guard);
    let result = result?;
    deferred.run(env, host)?;
    Ok(result)
}

fn translate_key(key_code: jint, unicode_char: jint, key_state: KeyState) -> Option<Key> {
    match key_code {
        KEYCODE_ENTER => Some(Key::Enter),
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
            char::from_u32(unicode_char as u32).and_then(Key::character)
        }
        _ => None,
    }
}

#[cfg(target_os = "android")]
fn install_panic_hook() {
    use std::ffi::{CString, c_char, c_int};

    #[link(name = "log")]
    unsafe extern "C" {
        fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
    }

    const ANDROID_LOG_ERROR: c_int = 6;

    std::panic::set_hook(Box::new(|info| {
        let tag = c"AccessKit";
        let text = CString::new(info.to_string()).unwrap_or_default();
        // SAFETY: Both strings are valid, NUL-terminated C strings.
        unsafe { __android_log_write(ANDROID_LOG_ERROR, tag.as_ptr(), text.as_ptr()) };
    }));
}

#[cfg(not(target_os = "android"))]
fn install_panic_hook() {}

fn native_create<'local>(_env: &mut Env<'local>, _class: JClass<'local>) -> Result<jlong> {
    let state = Box::new(Mutex::new(ViewState {
        adapter: Adapter::default(),
        ui: Ui {
            state: UiState::new(),
            show_keyboard: false,
        },
        input_method: InputMethodState::default(),
    }));
    Ok(Box::into_raw(state) as jlong)
}

fn native_destroy<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> Result<()> {
    // SAFETY: The Java view calls this at most once per handle and never
    // uses the handle afterwards.
    drop(unsafe { Box::from_raw(handle as *mut ViewHandle) });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn native_set_viewport<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    scale_factor: jfloat,
    safe_area_inset_x: jfloat,
    safe_area_inset_y: jfloat,
    safe_area_width: jfloat,
    safe_area_height: jfloat,
) -> Result<()> {
    unsafe {
        with_view_state(env, &host, handle, |_env, state, deferred| {
            state.ui.set_viewport(
                scale_factor.into(),
                Vec2::new(safe_area_inset_x.into(), safe_area_inset_y.into()),
                Size::new(safe_area_width.into(), safe_area_height.into()) / scale_factor.into(),
            );
            state.update_accessibility_tree(deferred);
            state.update_input_method(deferred);
            Ok(())
        })
    }
}

fn native_create_accessibility_node_info<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
) -> Result<JObject<'local>> {
    unsafe {
        with_view_state(env, &host, handle, |env, state, _deferred| {
            state
                .adapter
                .create_accessibility_node_info(&mut state.ui, env, &host, virtual_view_id)
        })
    }
}

fn native_find_focus<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    focus_type: jint,
) -> Result<JObject<'local>> {
    unsafe {
        with_view_state(env, &host, handle, |env, state, _deferred| {
            state
                .adapter
                .find_focus(&mut state.ui, env, &host, focus_type)
        })
    }
}

fn native_perform_action<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
    action: jint,
    arguments: JObject<'local>,
) -> Result<jboolean> {
    unsafe {
        with_view_state(env, &host, handle, |env, state, deferred| {
            let Some(action) = PlatformAction::from_java(env, action, &arguments)? else {
                return Ok(JNI_FALSE);
            };
            let ViewState { adapter, ui, .. } = state;
            let Some(events) = adapter.perform_action(ui, virtual_view_id, &action) else {
                return Ok(JNI_FALSE);
            };
            deferred.raise(events);
            state.after_input(deferred);
            Ok(JNI_TRUE)
        })
    }
}

fn native_on_hover_event<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    action: jint,
    x: jfloat,
    y: jfloat,
) -> Result<jboolean> {
    unsafe {
        with_view_state(env, &host, handle, |_env, state, deferred| {
            let ViewState { adapter, ui, .. } = state;
            let Some(events) = adapter.on_hover_event(ui, action, x, y) else {
                return Ok(JNI_FALSE);
            };
            deferred.raise(events);
            Ok(JNI_TRUE)
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn native_on_key_event<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    action: jint,
    key_code: jint,
    unicode_char: jint,
    shift_pressed: jboolean,
    ctrl_pressed: jboolean,
) -> Result<jboolean> {
    let key_state = match action {
        ACTION_DOWN => KeyState::Pressed,
        ACTION_UP => KeyState::Released,
        _ => return Ok(JNI_FALSE),
    };
    let Some(key) = translate_key(key_code, unicode_char, key_state) else {
        return Ok(JNI_FALSE);
    };
    unsafe {
        with_view_state(env, &host, handle, |_env, state, deferred| {
            let handled = state.ui.handle_key(KeyEvent {
                key,
                state: key_state,
                modifiers: Modifiers {
                    shift: shift_pressed != JNI_FALSE,
                    control: ctrl_pressed != JNI_FALSE,
                },
            });
            state.after_input(deferred);
            Ok(if handled { JNI_TRUE } else { JNI_FALSE })
        })
    }
}

fn native_process_due_messages<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
) -> Result<()> {
    unsafe {
        with_view_state(env, &host, handle, |_env, state, deferred| {
            if state.ui.process_due_messages() {
                state.update_accessibility_tree(deferred);
                state.update_input_method(deferred);
            }
            state.schedule_next_message(deferred);
            Ok(())
        })
    }
}

fn native_create_input_connection<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    editor_info: JObject<'local>,
) -> Result<jboolean> {
    unsafe {
        with_view_state(env, &host, handle, |env, state, _deferred| {
            let Some(input) = state.connect_input_method() else {
                return Ok(JNI_FALSE);
            };
            let enter_action = if input.input_type == InputType::Multiline {
                IME_FLAG_NO_ENTER_ACTION
            } else if input.can_submit {
                IME_ACTION_DONE
            } else {
                IME_ACTION_NEXT
            };
            let (input_type, ime_options) = match input.input_type {
                InputType::Text => (TYPE_CLASS_TEXT, enter_action),
                InputType::Email => (
                    TYPE_CLASS_TEXT | TYPE_TEXT_VARIATION_EMAIL_ADDRESS,
                    enter_action,
                ),
                InputType::Password => (
                    TYPE_CLASS_TEXT | TYPE_TEXT_VARIATION_PASSWORD,
                    enter_action | IME_FLAG_NO_PERSONALIZED_LEARNING,
                ),
                InputType::Multiline => (
                    TYPE_CLASS_TEXT | TYPE_TEXT_FLAG_MULTI_LINE | TYPE_TEXT_FLAG_CAP_SENTENCES,
                    enter_action,
                ),
            };
            let ime_options = ime_options | IME_FLAG_NO_FULLSCREEN | IME_FLAG_NO_EXTRACT_UI;
            let (anchor, focus, _) = reported_selection(&input);
            for (name, value) in [
                (jni_str!("inputType"), input_type),
                (jni_str!("imeOptions"), ime_options),
                (jni_str!("initialSelStart"), anchor),
                (jni_str!("initialSelEnd"), focus),
            ] {
                env.set_field(&editor_info, name, jni_sig!("I"), value.into())?;
            }
            Ok(JNI_TRUE)
        })
    }
}

fn native_ime_get_text<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    request: jint,
    length: jint,
) -> Result<JString<'local>> {
    unsafe {
        with_view_state(env, &host, handle, |env, state, _deferred| {
            let Some(input) = state.ui.focused_text_input() else {
                return Ok(JString::default());
            };
            let selection = selection(&input);
            let start = utf16_offset(&input.text, selection.start);
            let end = utf16_offset(&input.text, selection.end);
            let text: Vec<u16> = input.text.encode_utf16().collect();
            let length = length.max(0) as usize;
            let range = match request {
                IME_TEXT_BEFORE_CURSOR => start.saturating_sub(length)..start,
                IME_TEXT_AFTER_CURSOR => end..end.saturating_add(length).min(text.len()),
                IME_SELECTED_TEXT if start != end => start..end,
                _ => return Ok(JString::default()),
            };
            env.new_string(String::from_utf16_lossy(&text[range]))
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn native_ime_edit<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    request: jint,
    text: JString<'local>,
    a: jint,
    b: jint,
) -> Result<jboolean> {
    unsafe {
        with_view_state(env, &host, handle, |env, state, deferred| {
            let text = if text.is_null() {
                String::new()
            } else {
                text.try_to_string(env)?
            };
            if request == IME_PERFORM_EDITOR_ACTION {
                let handled = match a {
                    IME_ACTION_NEXT | IME_ACTION_PREVIOUS => state.ui.handle_key(KeyEvent {
                        key: Key::Tab,
                        state: KeyState::Pressed,
                        modifiers: Modifiers {
                            shift: a == IME_ACTION_PREVIOUS,
                            control: false,
                        },
                    }),
                    _ => state.ui.edit_focused_text(&ImeEdit::Submit),
                };
                state.after_input(deferred);
                return Ok(if handled { JNI_TRUE } else { JNI_FALSE });
            }
            let Some(edits) = state
                .ui
                .focused_text_input()
                .and_then(|input| input_connection_edits(&input, request, text, a, b))
            else {
                return Ok(JNI_FALSE);
            };
            for edit in &edits {
                state.ui.edit_focused_text(edit);
            }
            state.after_ime_edit(deferred);
            Ok(JNI_TRUE)
        })
    }
}

fn native_ime_batch_edit<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    begin: jboolean,
) -> Result<jboolean> {
    unsafe {
        with_view_state(env, &host, handle, |_env, state, deferred| {
            let depth = &mut state.input_method.batch_edit_depth;
            if begin != JNI_FALSE {
                *depth += 1;
                return Ok(JNI_TRUE);
            }
            let Some(new_depth) = depth.checked_sub(1) else {
                return Ok(JNI_FALSE);
            };
            *depth = new_depth;
            if new_depth > 0 {
                return Ok(JNI_TRUE);
            }
            state.update_input_method(deferred);
            Ok(JNI_FALSE)
        })
    }
}

// `native_method!` expands to `AtomicBool::fetch_update`, deprecated since Rust 1.100.
#[allow(deprecated)]
const NATIVE_METHODS: &[NativeMethod] = &[
    native_method! {
        catch_unwind = false,
        static fn native_create() -> jlong,
    },
    native_method! {
        catch_unwind = false,
        static fn native_destroy(handle: jlong),
    },
    native_method! {
        catch_unwind = false,
        static fn native_set_viewport(
            handle: jlong,
            host: android.view.View,
            scale_factor: jfloat,
            safe_area_inset_x: jfloat,
            safe_area_inset_y: jfloat,
            safe_area_width: jfloat,
            safe_area_height: jfloat,
        ),
    },
    native_method! {
        catch_unwind = false,
        static fn native_create_accessibility_node_info(
            handle: jlong,
            host: android.view.View,
            virtual_view_id: jint,
        ) -> android.view.accessibility.AccessibilityNodeInfo,
    },
    native_method! {
        catch_unwind = false,
        static fn native_find_focus(
            handle: jlong,
            host: android.view.View,
            focus_type: jint,
        ) -> android.view.accessibility.AccessibilityNodeInfo,
    },
    native_method! {
        catch_unwind = false,
        static fn native_perform_action(
            handle: jlong,
            host: android.view.View,
            virtual_view_id: jint,
            action: jint,
            arguments: android.os.Bundle,
        ) -> jboolean,
    },
    native_method! {
        catch_unwind = false,
        static fn native_on_hover_event(
            handle: jlong,
            host: android.view.View,
            action: jint,
            x: jfloat,
            y: jfloat,
        ) -> jboolean,
    },
    native_method! {
        catch_unwind = false,
        static fn native_on_key_event(
            handle: jlong,
            host: android.view.View,
            action: jint,
            key_code: jint,
            unicode_char: jint,
            shift_pressed: jboolean,
            ctrl_pressed: jboolean,
        ) -> jboolean,
    },
    native_method! {
        catch_unwind = false,
        static fn native_process_due_messages(handle: jlong, host: android.view.View),
    },
    native_method! {
        catch_unwind = false,
        static fn native_create_input_connection(
            handle: jlong,
            host: android.view.View,
            editor_info: android.view.inputmethod.EditorInfo,
        ) -> jboolean,
    },
    native_method! {
        catch_unwind = false,
        static fn native_ime_get_text(
            handle: jlong,
            host: android.view.View,
            request: jint,
            length: jint,
        ) -> JString,
    },
    native_method! {
        catch_unwind = false,
        static fn native_ime_edit(
            handle: jlong,
            host: android.view.View,
            request: jint,
            text: JString,
            a: jint,
            b: jint,
        ) -> jboolean,
    },
    native_method! {
        catch_unwind = false,
        static fn native_ime_batch_edit(
            handle: jlong,
            host: android.view.View,
            begin: jboolean,
        ) -> jboolean,
    },
];

/// # Safety
///
/// `vm` must be a valid pointer to the Java VM.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(
    vm: *mut accesskit_android::jni::sys::JavaVM,
    _reserved: *mut c_void,
) -> jint {
    install_panic_hook();
    let vm = unsafe { JavaVM::from_raw(vm) };
    vm.attach_current_thread(|env| {
        // SAFETY: `native_method!` checks each function against its signature.
        unsafe {
            env.register_native_methods(
                jni_str!("dev/accesskit/showcase/ShowcaseView"),
                NATIVE_METHODS,
            )
        }
    })
    .unwrap();
    JNI_VERSION_1_6
}
