// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from jni-rs
// Copyright 2016 Prevoty, Inc. and jni-rs contributors
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActivationHandler, TreeUpdate};
use jni::{
    JNIEnv, JavaVM, NativeMethod,
    errors::Result,
    objects::{GlobalRef, JClass, JObject, WeakRef},
    sys::{JNI_FALSE, JNI_TRUE, jboolean, jfloat, jint, jlong},
};
use log::debug;
use std::{
    any::Any,
    collections::BTreeMap,
    ffi::c_void,
    fmt::{Debug, Formatter},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicI64, Ordering},
    },
};

use crate::{action::PlatformAction, adapter::Adapter, event::QueuedEvents};

/// Log a panic caught at a JNI boundary and clear any Java exception it left
/// pending (returning to the JVM with one pending rethrows it in the Java
/// caller — for `runCallback` that is inside the UI `Looper`).
fn report_contained_panic(env: &mut JNIEnv, what: &str, payload: Box<dyn Any + Send>) {
    let msg: &str = if let Some(s) = payload.downcast_ref::<&'static str>() {
        s
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s
    } else {
        "non-string panic payload"
    };
    log::error!("accesskit_android: panic contained at the JNI boundary in {what}: {msg}");
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
}

/// Run a native-method body with the unwind boundary an `extern "system"` fn
/// requires, returning `T::default()` (null / `JNI_FALSE`) if it panicked.
///
/// Rust aborts the process when a panic reaches an `extern "system"` frame,
/// and the code under these entry points surfaces every JNI error (a detached
/// host view, a missing method, an OOM) as a panic — so without this
/// boundary, any of those errors aborted the process. A contained panic
/// poisons the adapter's mutex, so later delegate calls fail too (also
/// contained): the process survives and accessibility degrades.
fn guard_ffi<'local, T: Default>(
    env: &mut JNIEnv<'local>,
    what: &str,
    f: impl FnOnce(&mut JNIEnv<'local>) -> T,
) -> T {
    match catch_unwind(AssertUnwindSafe(|| f(&mut *env))) {
        Ok(value) => value,
        Err(payload) => {
            report_contained_panic(env, what, payload);
            T::default()
        }
    }
}

struct InnerInjectingAdapter {
    adapter: Adapter,
    activation_handler: Box<dyn ActivationHandler + Send>,
    action_handler: Box<dyn ActionHandler + Send>,
}

impl Debug for InnerInjectingAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerInjectingAdapter")
            .field("adapter", &self.adapter)
            .field("activation_handler", &"ActivationHandler")
            .field("action_handler", &"ActionHandler")
            .finish()
    }
}

impl InnerInjectingAdapter {
    fn create_accessibility_node_info<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        host: &JObject,
        virtual_view_id: jint,
    ) -> JObject<'local> {
        self.adapter.create_accessibility_node_info(
            &mut *self.activation_handler,
            env,
            host,
            virtual_view_id,
        )
    }

    fn find_focus<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        host: &JObject,
        focus_type: jint,
    ) -> JObject<'local> {
        self.adapter
            .find_focus(&mut *self.activation_handler, env, host, focus_type)
    }

    fn perform_action(
        &mut self,
        virtual_view_id: jint,
        action: &PlatformAction,
    ) -> Option<QueuedEvents> {
        self.adapter
            .perform_action(&mut *self.action_handler, virtual_view_id, action)
    }

    fn on_hover_event(&mut self, action: jint, x: jfloat, y: jfloat) -> Option<QueuedEvents> {
        self.adapter
            .on_hover_event(&mut *self.activation_handler, action, x, y)
    }
}

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(0);
static HANDLE_MAP: Mutex<BTreeMap<jlong, Weak<Mutex<InnerInjectingAdapter>>>> =
    Mutex::new(BTreeMap::new());

fn inner_adapter_from_handle(handle: jlong) -> Option<Arc<Mutex<InnerInjectingAdapter>>> {
    let handle_map_guard = HANDLE_MAP.lock().unwrap();
    handle_map_guard.get(&handle).and_then(Weak::upgrade)
}

static NEXT_CALLBACK_HANDLE: AtomicI64 = AtomicI64::new(0);
#[allow(clippy::type_complexity)]
static CALLBACK_MAP: Mutex<
    BTreeMap<jlong, Box<dyn FnOnce(&mut JNIEnv, &JClass, &JObject) + Send>>,
> = Mutex::new(BTreeMap::new());

fn post_to_ui_thread(
    env: &mut JNIEnv,
    delegate_class: &JClass,
    host: &JObject,
    callback: impl FnOnce(&mut JNIEnv, &JClass, &JObject) + Send + 'static,
) {
    let handle = NEXT_CALLBACK_HANDLE.fetch_add(1, Ordering::Relaxed);
    CALLBACK_MAP
        .lock()
        .unwrap()
        .insert(handle, Box::new(callback));
    let runnable = env
        .call_static_method(
            delegate_class,
            "newCallback",
            "(Landroid/view/View;J)Ljava/lang/Runnable;",
            &[host.into(), handle.into()],
        )
        .unwrap()
        .l()
        .unwrap();
    // Free the `Runnable` local reference: this can be reached from a caller
    // thread that never pops a local frame.
    let runnable = env.auto_local(runnable);
    env.call_method(
        host,
        "post",
        "(Ljava/lang/Runnable;)Z",
        &[(&runnable).into()],
    )
    .unwrap();
}

extern "system" fn run_callback<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    host: JObject<'local>,
    handle: jlong,
) {
    let Some(callback) = CALLBACK_MAP.lock().unwrap().remove(&handle) else {
        return;
    };
    guard_ffi(&mut env, "runCallback", |env| callback(env, &class, &host));
}

extern "system" fn create_accessibility_node_info<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
) -> JObject<'local> {
    guard_ffi(&mut env, "createAccessibilityNodeInfo", |env| {
        let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
            return JObject::null();
        };
        let mut inner_adapter = inner_adapter.lock().unwrap();
        inner_adapter.create_accessibility_node_info(env, &host, virtual_view_id)
    })
}

extern "system" fn find_focus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    focus_type: jint,
) -> JObject<'local> {
    guard_ffi(&mut env, "findFocus", |env| {
        let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
            return JObject::null();
        };
        let mut inner_adapter = inner_adapter.lock().unwrap();
        inner_adapter.find_focus(env, &host, focus_type)
    })
}

extern "system" fn perform_action<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
    action: jint,
    arguments: JObject<'local>,
) -> jboolean {
    guard_ffi(&mut env, "performAction", |env| {
        let Some(action) = PlatformAction::from_java(env, action, &arguments) else {
            return JNI_FALSE;
        };
        let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
            return JNI_FALSE;
        };
        let mut inner_adapter = inner_adapter.lock().unwrap();
        let Some(events) = inner_adapter.perform_action(virtual_view_id, &action) else {
            return JNI_FALSE;
        };
        drop(inner_adapter);
        events.raise(env, &host);
        JNI_TRUE
    })
}

extern "system" fn on_hover_event<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    action: jint,
    x: jfloat,
    y: jfloat,
) -> jboolean {
    guard_ffi(&mut env, "onHoverEvent", |env| {
        let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
            return JNI_FALSE;
        };
        let mut inner_adapter = inner_adapter.lock().unwrap();
        let Some(events) = inner_adapter.on_hover_event(action, x, y) else {
            return JNI_FALSE;
        };
        drop(inner_adapter);
        events.raise(env, &host);
        JNI_TRUE
    })
}

fn delegate_class(env: &mut JNIEnv) -> &'static JClass<'static> {
    static CLASS: OnceLock<GlobalRef> = OnceLock::new();
    let global = CLASS.get_or_init(|| {
        #[cfg(feature = "embedded-dex")]
        let class = {
            let dex_class_loader_class = env
                .find_class("dalvik/system/InMemoryDexClassLoader")
                .unwrap();
            let dex_bytes = include_bytes!("../classes.dex");
            let dex_buffer = unsafe {
                env.new_direct_byte_buffer(dex_bytes.as_ptr() as *mut u8, dex_bytes.len())
            }
            .unwrap();
            let dex_class_loader = env
                .new_object(
                    &dex_class_loader_class,
                    "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
                    &[(&dex_buffer).into(), (&JObject::null()).into()],
                )
                .unwrap();
            let class_name = env.new_string("dev.accesskit.android.Delegate").unwrap();
            let class_obj = env
                .call_method(
                    &dex_class_loader,
                    "loadClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[(&class_name).into()],
                )
                .unwrap()
                .l()
                .unwrap();
            JClass::from(class_obj)
        };
        #[cfg(not(feature = "embedded-dex"))]
        let class = env.find_class("dev/accesskit/android/Delegate").unwrap();
        env.register_native_methods(
            &class,
            &[
                NativeMethod {
                    name: "runCallback".into(),
                    sig: "(Landroid/view/View;J)V".into(),
                    fn_ptr: run_callback as *mut c_void,
                },
                NativeMethod {
                    name: "createAccessibilityNodeInfo".into(),
                    sig:
                        "(JLandroid/view/View;I)Landroid/view/accessibility/AccessibilityNodeInfo;"
                            .into(),
                    fn_ptr: create_accessibility_node_info as *mut c_void,
                },
                NativeMethod {
                    name: "findFocus".into(),
                    sig:
                        "(JLandroid/view/View;I)Landroid/view/accessibility/AccessibilityNodeInfo;"
                            .into(),
                    fn_ptr: find_focus as *mut c_void,
                },
                NativeMethod {
                    name: "performAction".into(),
                    sig: "(JLandroid/view/View;IILandroid/os/Bundle;)Z".into(),
                    fn_ptr: perform_action as *mut c_void,
                },
                NativeMethod {
                    name: "onHoverEvent".into(),
                    sig: "(JLandroid/view/View;IFF)Z".into(),
                    fn_ptr: on_hover_event as *mut c_void,
                },
            ],
        )
        .unwrap();
        env.new_global_ref(class).unwrap()
    });
    global.as_obj().into()
}

/// High-level AccessKit Android adapter that injects itself into an Android
/// view without requiring the view class to be modified for accessibility.
///
/// This depends on the Java `dev.accesskit.android.Delegate` class, the source
/// code for which is in this crate's `java` directory. If the `embedded-dex`
/// feature is enabled, then that class is loaded from a prebuilt `.dex` file
/// that this crate embeds. Otherwise, it's simply assumed that the class
/// is in the application package. None of this type's public functions
/// make assumptions about whether they're called from the Android UI thread.
/// As such, some requests are posted to the UI thread and handled
/// asynchronously.
pub struct InjectingAdapter {
    vm: JavaVM,
    delegate_class: &'static JClass<'static>,
    host: WeakRef,
    handle: jlong,
    inner: Arc<Mutex<InnerInjectingAdapter>>,
}

impl Debug for InjectingAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerInjectingAdapter")
            .field("vm", &self.vm)
            .field("delegate_class", &self.delegate_class)
            .field("host", &"WeakRef")
            .field("handle", &self.handle)
            .field("inner", &self.inner)
            .finish()
    }
}

impl InjectingAdapter {
    pub fn new(
        env: &mut JNIEnv,
        host: &JObject,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
    ) -> Self {
        let inner = Arc::new(Mutex::new(InnerInjectingAdapter {
            adapter: Adapter::default(),
            activation_handler: Box::new(activation_handler),
            action_handler: Box::new(action_handler),
        }));
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        HANDLE_MAP
            .lock()
            .unwrap()
            .insert(handle, Arc::downgrade(&inner));
        let delegate_class = delegate_class(env);
        post_to_ui_thread(
            env,
            delegate_class,
            host,
            move |env, delegate_class, host| {
                let prev_delegate = env
                    .call_method(
                        host,
                        "getAccessibilityDelegate",
                        "()Landroid/view/View$AccessibilityDelegate;",
                        &[],
                    )
                    .unwrap()
                    .l()
                    .unwrap();
                if !prev_delegate.is_null() {
                    panic!("host already has an accessibility delegate");
                }
                let delegate = env
                    .new_object(delegate_class, "(J)V", &[handle.into()])
                    .unwrap();
                env.call_method(
                    host,
                    "setAccessibilityDelegate",
                    "(Landroid/view/View$AccessibilityDelegate;)V",
                    &[(&delegate).into()],
                )
                .unwrap();
                env.call_method(
                    host,
                    "setOnHoverListener",
                    "(Landroid/view/View$OnHoverListener;)V",
                    &[(&delegate).into()],
                )
                .unwrap();
            },
        );
        Self {
            vm: env.get_java_vm().unwrap(),
            delegate_class,
            host: env.new_weak_ref(host).unwrap().unwrap(),
            handle,
            inner,
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        let mut env = self.vm.get_env().unwrap();
        let Some(host) = self.host.upgrade_local(&env).unwrap() else {
            return;
        };
        // `upgrade_local` creates a new local reference, and on a caller thread
        // that is not executing a JNI native method (which this type documents
        // as supported) nothing pops a local frame — so without an explicit
        // free, every call leaks one local until ART hits its local reference
        // table limit.
        let host = env.auto_local(host);
        // A contained panic in a delegate callback poisons this mutex; skip
        // rather than propagate the poison panic into the caller's thread —
        // accessibility is already degraded at that point.
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        let Some(events) = inner.adapter.update_if_active(update_factory) else {
            return;
        };
        drop(inner);
        post_to_ui_thread(
            &mut env,
            self.delegate_class,
            &host,
            |env, _delegate_class, host| {
                events.raise(env, host);
            },
        );
    }
}

impl Drop for InjectingAdapter {
    fn drop(&mut self) {
        fn drop_impl(env: &mut JNIEnv, delegate_class: &JClass, host: &WeakRef) -> Result<()> {
            let Some(host) = host.upgrade_local(env)? else {
                return Ok(());
            };
            // Same local-reference discipline as `update_if_active`.
            let host = env.auto_local(host);
            post_to_ui_thread(env, delegate_class, &host, |env, delegate_class, host| {
                let prev_delegate = env
                    .call_method(
                        host,
                        "getAccessibilityDelegate",
                        "()Landroid/view/View$AccessibilityDelegate;",
                        &[],
                    )
                    .unwrap()
                    .l()
                    .unwrap();
                if prev_delegate.is_null()
                    && !env.is_instance_of(&prev_delegate, delegate_class).unwrap()
                {
                    return;
                }
                let null = JObject::null();
                env.call_method(
                    host,
                    "setAccessibilityDelegate",
                    "(Landroid/view/View$AccessibilityDelegate;)V",
                    &[(&null).into()],
                )
                .unwrap();
                env.call_method(
                    host,
                    "setOnHoverListener",
                    "(Landroid/view/View$OnHoverListener;)V",
                    &[(&null).into()],
                )
                .unwrap();
            });
            Ok(())
        }

        let res = match self.vm.get_env() {
            Ok(mut env) => drop_impl(&mut env, self.delegate_class, &self.host),
            Err(_) => self
                .vm
                .attach_current_thread()
                .and_then(|mut env| drop_impl(&mut env, self.delegate_class, &self.host)),
        };

        if let Err(err) = res {
            debug!("error dropping InjectingAdapter: {:#?}", err);
        }

        HANDLE_MAP.lock().unwrap().remove(&self.handle);
    }
}
