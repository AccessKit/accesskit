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
    Env, JavaVM, NativeMethod,
    errors::Result,
    jni_sig, jni_str, native_method,
    objects::{JClass, JObject},
    refs::{Global, Weak as WeakRef},
    sys::{jfloat, jint, jlong},
};
use log::debug;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter},
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicI64, Ordering},
    },
};

use crate::{action::PlatformAction, adapter::Adapter, event::QueuedEvents};

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
        env: &mut Env<'local>,
        host: &JObject,
        virtual_view_id: jint,
    ) -> Result<JObject<'local>> {
        self.adapter.create_accessibility_node_info(
            &mut *self.activation_handler,
            env,
            host,
            virtual_view_id,
        )
    }

    fn find_focus<'local>(
        &mut self,
        env: &mut Env<'local>,
        host: &JObject,
        focus_type: jint,
    ) -> Result<JObject<'local>> {
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
    BTreeMap<jlong, Box<dyn FnOnce(&mut Env, &JClass, &JObject) -> Result<()> + Send>>,
> = Mutex::new(BTreeMap::new());

fn post_to_ui_thread(
    env: &mut Env,
    delegate_class: &JClass,
    host: &JObject,
    callback: impl FnOnce(&mut Env, &JClass, &JObject) -> Result<()> + Send + 'static,
) -> Result<()> {
    let handle = NEXT_CALLBACK_HANDLE.fetch_add(1, Ordering::Relaxed);
    CALLBACK_MAP
        .lock()
        .unwrap()
        .insert(handle, Box::new(callback));
    let result = (|| {
        let runnable = env
            .call_static_method(
                delegate_class,
                jni_str!("newCallback"),
                jni_sig!("(Landroid/view/View;J)Ljava/lang/Runnable;"),
                &[host.into(), handle.into()],
            )?
            .l()?;
        env.call_method(
            host,
            jni_str!("post"),
            jni_sig!("(Ljava/lang/Runnable;)Z"),
            &[(&runnable).into()],
        )?
        .z()
    })();
    if !matches!(result, Ok(true)) {
        CALLBACK_MAP.lock().unwrap().remove(&handle);
    }
    result.map(|_| ())
}

fn run_callback<'local>(
    env: &mut Env<'local>,
    class: JClass<'local>,
    host: JObject<'local>,
    handle: jlong,
) -> Result<()> {
    let Some(callback) = CALLBACK_MAP.lock().unwrap().remove(&handle) else {
        return Ok(());
    };
    callback(env, &class, &host)
}

fn create_accessibility_node_info<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
) -> Result<JObject<'local>> {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return Ok(JObject::null());
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    inner_adapter.create_accessibility_node_info(env, &host, virtual_view_id)
}

fn find_focus<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    focus_type: jint,
) -> Result<JObject<'local>> {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return Ok(JObject::null());
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    inner_adapter.find_focus(env, &host, focus_type)
}

fn perform_action<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    virtual_view_id: jint,
    action: jint,
    arguments: JObject<'local>,
) -> Result<bool> {
    let Some(action) = PlatformAction::from_java(env, action, &arguments)? else {
        return Ok(false);
    };
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return Ok(false);
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    let Some(events) = inner_adapter.perform_action(virtual_view_id, &action) else {
        return Ok(false);
    };
    drop(inner_adapter);
    events.raise(env, &host)?;
    Ok(true)
}

fn on_hover_event<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    adapter_handle: jlong,
    host: JObject<'local>,
    action: jint,
    x: jfloat,
    y: jfloat,
) -> Result<bool> {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return Ok(false);
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    let Some(events) = inner_adapter.on_hover_event(action, x, y) else {
        return Ok(false);
    };
    drop(inner_adapter);
    events.raise(env, &host)?;
    Ok(true)
}

// `native_method!` expands to `AtomicBool::fetch_update`, deprecated since Rust 1.100.
#[allow(deprecated)]
const NATIVE_METHODS: &[NativeMethod] = &[
    native_method! {
        catch_unwind = false,
        static fn run_callback(host: android.view.View, handle: jlong),
    },
    native_method! {
        catch_unwind = false,
        static fn create_accessibility_node_info(
            adapter_handle: jlong,
            host: android.view.View,
            virtual_view_id: jint,
        ) -> android.view.accessibility.AccessibilityNodeInfo,
    },
    native_method! {
        catch_unwind = false,
        static fn find_focus(
            adapter_handle: jlong,
            host: android.view.View,
            focus_type: jint,
        ) -> android.view.accessibility.AccessibilityNodeInfo,
    },
    native_method! {
        catch_unwind = false,
        static fn perform_action(
            adapter_handle: jlong,
            host: android.view.View,
            virtual_view_id: jint,
            action: jint,
            arguments: android.os.Bundle,
        ) -> bool,
    },
    native_method! {
        catch_unwind = false,
        static fn on_hover_event(
            adapter_handle: jlong,
            host: android.view.View,
            action: jint,
            x: jfloat,
            y: jfloat,
        ) -> bool,
    },
];

fn delegate_class(env: &mut Env) -> &'static JClass<'static> {
    static CLASS: OnceLock<Global<JClass<'static>>> = OnceLock::new();
    CLASS.get_or_init(|| {
        #[cfg(feature = "embedded-dex")]
        let class = {
            let dex_class_loader_class = env
                .find_class(jni_str!("dalvik/system/InMemoryDexClassLoader"))
                .unwrap();
            let dex_bytes = include_bytes!("../classes.dex");
            let dex_buffer = unsafe {
                env.new_direct_byte_buffer(dex_bytes.as_ptr() as *mut u8, dex_bytes.len())
            }
            .unwrap();
            let dex_class_loader = env
                .new_object(
                    &dex_class_loader_class,
                    jni_sig!("(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V"),
                    &[(&dex_buffer).into(), (&JObject::null()).into()],
                )
                .unwrap();
            let class_name = env.new_string("dev.accesskit.android.Delegate").unwrap();
            let class_obj = env
                .call_method(
                    &dex_class_loader,
                    jni_str!("loadClass"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
                    &[(&class_name).into()],
                )
                .unwrap()
                .l()
                .unwrap();
            env.cast_local::<JClass>(class_obj).unwrap()
        };
        #[cfg(not(feature = "embedded-dex"))]
        let class = env
            .find_class(jni_str!("dev/accesskit/android/Delegate"))
            .unwrap();
        // SAFETY: `native_method!` checks each function against its signature.
        unsafe { env.register_native_methods(&class, NATIVE_METHODS) }.unwrap();
        env.new_global_ref(class).unwrap()
    })
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
    host: WeakRef<JObject<'static>>,
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
        env: &mut Env,
        host: &JObject,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
    ) -> Self {
        let inner = Arc::new(Mutex::new(InnerInjectingAdapter {
            adapter: Adapter::default(),
            activation_handler: Box::new(activation_handler),
            action_handler: Box::new(action_handler),
        }));
        let vm = env.get_java_vm().unwrap();
        let weak_host = env.new_weak_ref(host).unwrap();
        let delegate_class = delegate_class(env);
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        HANDLE_MAP
            .lock()
            .unwrap()
            .insert(handle, Arc::downgrade(&inner));
        if let Err(err) = post_to_ui_thread(
            env,
            delegate_class,
            host,
            move |env, delegate_class, host| {
                let prev_delegate = env
                    .call_method(
                        host,
                        jni_str!("getAccessibilityDelegate"),
                        jni_sig!("()Landroid/view/View$AccessibilityDelegate;"),
                        &[],
                    )?
                    .l()?;
                if !prev_delegate.is_null() {
                    panic!("host already has an accessibility delegate");
                }
                let delegate =
                    env.new_object(delegate_class, jni_sig!("(J)V"), &[handle.into()])?;
                env.call_method(
                    host,
                    jni_str!("setAccessibilityDelegate"),
                    jni_sig!("(Landroid/view/View$AccessibilityDelegate;)V"),
                    &[(&delegate).into()],
                )?;
                env.call_method(
                    host,
                    jni_str!("setOnHoverListener"),
                    jni_sig!("(Landroid/view/View$OnHoverListener;)V"),
                    &[(&delegate).into()],
                )?;
                Ok(())
            },
        ) {
            HANDLE_MAP.lock().unwrap().remove(&handle);
            panic!("failed to post accessibility delegate installation: {err}");
        }
        Self {
            vm,
            delegate_class,
            host: weak_host,
            handle,
            inner,
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    ///
    /// This method may be called from any thread. If the calling thread
    /// isn't attached to the JVM, it's attached for the duration of the call,
    /// which is expensive; callers that update frequently from a native
    /// thread should attach that thread permanently.
    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        let events = self
            .inner
            .lock()
            .unwrap()
            .adapter
            .update_if_active(update_factory);
        let Some(events) = events else {
            return;
        };
        self.attach_and_post_to_ui_thread(|env, _delegate_class, host| events.raise(env, host))
            .expect("failed to post accessibility events to the UI thread");
    }

    fn attach_and_post_to_ui_thread(
        &self,
        callback: impl FnOnce(&mut Env, &JClass, &JObject) -> Result<()> + Send + 'static,
    ) -> Result<()> {
        self.vm.attach_current_thread_for_scope(|env| {
            let Some(host) = self.host.upgrade_local(env)? else {
                return Ok(());
            };
            post_to_ui_thread(env, self.delegate_class, &host, callback)
        })
    }
}

fn uninstall_delegate(env: &mut Env, delegate_class: &JClass, host: &JObject) -> Result<()> {
    let prev_delegate = env
        .call_method(
            host,
            jni_str!("getAccessibilityDelegate"),
            jni_sig!("()Landroid/view/View$AccessibilityDelegate;"),
            &[],
        )?
        .l()?;
    if prev_delegate.is_null() || !env.is_instance_of(&prev_delegate, delegate_class)? {
        return Ok(());
    }
    let null = JObject::null();
    env.call_method(
        host,
        jni_str!("setAccessibilityDelegate"),
        jni_sig!("(Landroid/view/View$AccessibilityDelegate;)V"),
        &[(&null).into()],
    )?;
    env.call_method(
        host,
        jni_str!("setOnHoverListener"),
        jni_sig!("(Landroid/view/View$OnHoverListener;)V"),
        &[(&null).into()],
    )?;
    Ok(())
}

impl Drop for InjectingAdapter {
    fn drop(&mut self) {
        if let Err(err) = self.attach_and_post_to_ui_thread(uninstall_delegate) {
            debug!("error dropping InjectingAdapter: {:#?}", err);
        }
        HANDLE_MAP.lock().unwrap().remove(&self.handle);
    }
}
