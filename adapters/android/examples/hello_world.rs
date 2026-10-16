use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate, Vec2};
use accesskit_android::{
    Adapter, PlatformAction, QueuedEvents,
    jni::{
        JNIEnv, JavaVM, NativeMethod,
        objects::{JClass, JObject},
        sys::{JNI_FALSE, JNI_TRUE, JNI_VERSION_1_6, jboolean, jfloat, jint, jlong},
    },
};
use example_common::{Key, KeyEvent, KeyState, Modifiers, UiState};
use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

const ACTION_DOWN: jint = 0;
const ACTION_UP: jint = 1;
const KEYCODE_TAB: jint = 61;
const KEYCODE_SPACE: jint = 62;
const KEYCODE_ENTER: jint = 66;

struct Ui(UiState);

impl ActivationHandler for Ui {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.0.build_tree_update())
    }
}

impl ActionHandler for Ui {
    fn do_action(&mut self, request: ActionRequest) {
        self.0.do_action(&request);
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

#[derive(Default)]
struct DeferredCallbacks(Vec<DeferredCallback>);

type DeferredCallback = Box<dyn FnOnce(&mut JNIEnv, &JObject)>;

impl DeferredCallbacks {
    fn push(&mut self, callback: impl FnOnce(&mut JNIEnv, &JObject) + 'static) {
        self.0.push(Box::new(callback));
    }

    fn raise(&mut self, events: QueuedEvents) {
        self.push(move |env, host| events.raise(env, host));
    }

    fn run(self, env: &mut JNIEnv, host: &JObject) {
        for callback in self.0 {
            callback(env, host);
        }
    }
}

struct ViewState {
    adapter: Adapter,
    ui: Ui,
}

impl ViewState {
    fn update_accessibility_tree(&mut self, deferred: &mut DeferredCallbacks) {
        let Self { adapter, ui } = self;
        if let Some(events) = adapter.update_if_active(|| ui.build_tree_update()) {
            deferred.raise(events);
        }
    }

    fn after_input(&mut self, deferred: &mut DeferredCallbacks) {
        self.update_accessibility_tree(deferred);
        let Some(delay) = self.ui.time_until_announcement() else {
            return;
        };
        let delay = delay.as_millis() as jlong;
        deferred.push(move |env, host| {
            env.call_method(host, "scheduleAnnouncement", "(J)V", &[delay.into()])
                .unwrap();
        });
    }
}

type ViewHandle = Mutex<ViewState>;

/// # Safety
///
/// `handle` must have been returned by `nativeCreate` and not yet passed
/// to `nativeDestroy`.
unsafe fn with_view_state<'local, T>(
    env: &mut JNIEnv<'local>,
    host: &JObject,
    handle: jlong,
    f: impl FnOnce(&mut JNIEnv<'local>, &mut ViewState, &mut DeferredCallbacks) -> T,
) -> T {
    let state = unsafe { &*(handle as *const ViewHandle) };
    let mut deferred = DeferredCallbacks::default();
    let mut guard = state.lock().unwrap();
    let result = f(env, &mut guard, &mut deferred);
    drop(guard);
    deferred.run(env, host);
    result
}

fn translate_key(key_code: jint) -> Option<Key> {
    match key_code {
        KEYCODE_ENTER => Some(Key::Enter),
        KEYCODE_SPACE => Some(Key::Space),
        KEYCODE_TAB => Some(Key::Tab),
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

extern "system" fn create(_env: JNIEnv, _class: JClass) -> jlong {
    let state = Box::new(Mutex::new(ViewState {
        adapter: Adapter::default(),
        ui: Ui(UiState::new()),
    }));
    Box::into_raw(state) as jlong
}

extern "system" fn destroy(_env: JNIEnv, _class: JClass, handle: jlong) {
    // SAFETY: The Java view calls this at most once per handle and never
    // uses the handle afterwards.
    drop(unsafe { Box::from_raw(handle as *mut ViewHandle) });
}

extern "system" fn set_viewport(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    host: JObject,
    scale_factor: jfloat,
    safe_area_inset_x: jfloat,
    safe_area_inset_y: jfloat,
) {
    unsafe {
        with_view_state(&mut env, &host, handle, |_env, state, deferred| {
            state.ui.set_viewport(
                scale_factor.into(),
                Vec2::new(safe_area_inset_x.into(), safe_area_inset_y.into()),
            );
            state.update_accessibility_tree(deferred);
        })
    }
}

extern "system" fn create_accessibility_node_info<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
) -> JObject<'local> {
    unsafe {
        with_view_state(&mut env, &host, handle, |env, state, _deferred| {
            state
                .adapter
                .create_accessibility_node_info(&mut state.ui, env, &host, virtual_view_id)
        })
    }
}

extern "system" fn find_focus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    host: JObject<'local>,
    focus_type: jint,
) -> JObject<'local> {
    unsafe {
        with_view_state(&mut env, &host, handle, |env, state, _deferred| {
            state
                .adapter
                .find_focus(&mut state.ui, env, &host, focus_type)
        })
    }
}

extern "system" fn perform_action(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    host: JObject,
    virtual_view_id: jint,
    action: jint,
    arguments: JObject,
) -> jboolean {
    unsafe {
        with_view_state(&mut env, &host, handle, |env, state, deferred| {
            let Some(action) = PlatformAction::from_java(env, action, &arguments) else {
                return JNI_FALSE;
            };
            let ViewState { adapter, ui } = state;
            let Some(events) = adapter.perform_action(ui, virtual_view_id, &action) else {
                return JNI_FALSE;
            };
            deferred.raise(events);
            state.after_input(deferred);
            JNI_TRUE
        })
    }
}

extern "system" fn on_hover_event(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    host: JObject,
    action: jint,
    x: jfloat,
    y: jfloat,
) -> jboolean {
    unsafe {
        with_view_state(&mut env, &host, handle, |_env, state, deferred| {
            let ViewState { adapter, ui } = state;
            let Some(events) = adapter.on_hover_event(ui, action, x, y) else {
                return JNI_FALSE;
            };
            deferred.raise(events);
            JNI_TRUE
        })
    }
}

extern "system" fn on_key_event(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    host: JObject,
    action: jint,
    key_code: jint,
    shift_pressed: jboolean,
) -> jboolean {
    let Some(key) = translate_key(key_code) else {
        return JNI_FALSE;
    };
    let key_state = match action {
        ACTION_DOWN => KeyState::Pressed,
        ACTION_UP => KeyState::Released,
        _ => return JNI_FALSE,
    };
    unsafe {
        with_view_state(&mut env, &host, handle, |_env, state, deferred| {
            state.ui.handle_key(KeyEvent {
                key,
                state: key_state,
                modifiers: Modifiers {
                    shift: shift_pressed != JNI_FALSE,
                },
            });
            state.after_input(deferred);
            JNI_TRUE
        })
    }
}

extern "system" fn flush_announcement(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    host: JObject,
) {
    unsafe {
        with_view_state(&mut env, &host, handle, |_env, state, deferred| {
            if state.ui.flush_announcement() {
                state.update_accessibility_tree(deferred);
            }
        })
    }
}

/// # Safety
///
/// `vm` must be a valid pointer to the Java VM.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(
    vm: *mut accesskit_android::jni::sys::JavaVM,
    _reserved: *mut c_void,
) -> jint {
    install_panic_hook();
    let vm = unsafe { JavaVM::from_raw(vm) }.unwrap();
    let mut env = vm.get_env().unwrap();
    env.register_native_methods(
        "dev/accesskit/helloworld/HelloWorldView",
        &[
            NativeMethod {
                name: "nativeCreate".into(),
                sig: "()J".into(),
                fn_ptr: create as *mut c_void,
            },
            NativeMethod {
                name: "nativeDestroy".into(),
                sig: "(J)V".into(),
                fn_ptr: destroy as *mut c_void,
            },
            NativeMethod {
                name: "nativeSetViewport".into(),
                sig: "(JLandroid/view/View;FFF)V".into(),
                fn_ptr: set_viewport as *mut c_void,
            },
            NativeMethod {
                name: "nativeCreateAccessibilityNodeInfo".into(),
                sig: "(JLandroid/view/View;I)Landroid/view/accessibility/AccessibilityNodeInfo;"
                    .into(),
                fn_ptr: create_accessibility_node_info as *mut c_void,
            },
            NativeMethod {
                name: "nativeFindFocus".into(),
                sig: "(JLandroid/view/View;I)Landroid/view/accessibility/AccessibilityNodeInfo;"
                    .into(),
                fn_ptr: find_focus as *mut c_void,
            },
            NativeMethod {
                name: "nativePerformAction".into(),
                sig: "(JLandroid/view/View;IILandroid/os/Bundle;)Z".into(),
                fn_ptr: perform_action as *mut c_void,
            },
            NativeMethod {
                name: "nativeOnHoverEvent".into(),
                sig: "(JLandroid/view/View;IFF)Z".into(),
                fn_ptr: on_hover_event as *mut c_void,
            },
            NativeMethod {
                name: "nativeOnKeyEvent".into(),
                sig: "(JLandroid/view/View;IIZ)Z".into(),
                fn_ptr: on_key_event as *mut c_void,
            },
            NativeMethod {
                name: "nativeFlushAnnouncement".into(),
                sig: "(JLandroid/view/View;)V".into(),
                fn_ptr: flush_announcement as *mut c_void,
            },
        ],
    )
    .unwrap();
    JNI_VERSION_1_6
}
