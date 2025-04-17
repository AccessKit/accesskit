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
    errors::Result,
    objects::{GlobalRef, JClass, JObject, WeakRef},
    sys::{jboolean, jfloat, jint, jlong, JNI_FALSE, JNI_TRUE},
    JNIEnv, JavaVM, NativeMethod,
};
use log::debug;
use once_cell::sync::OnceCell;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    fmt::{Debug, Formatter},
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex, Weak,
    },
};

use crate::{
    adapter::{Adapter, QueuedEvents},
    util::*,
};

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
    fn populate_node_info(
        &mut self,
        env: &mut JNIEnv,
        host: &JObject,
        host_screen_x: jint,
        host_screen_y: jint,
        virtual_view_id: jint,
        jni_node: &JObject,
    ) -> Result<bool> {
        self.adapter.populate_node_info(
            &mut *self.activation_handler,
            env,
            host,
            host_screen_x,
            host_screen_y,
            virtual_view_id,
            jni_node,
        )
    }

    fn input_focus(&mut self) -> jint {
        self.adapter.input_focus(&mut *self.activation_handler)
    }

    fn virtual_view_at_point(&mut self, x: jfloat, y: jfloat) -> jint {
        self.adapter
            .virtual_view_at_point(&mut *self.activation_handler, x, y)
    }

    fn perform_action(&mut self, virtual_view_id: jint, action: jint) -> Option<QueuedEvents> {
        self.adapter
            .perform_action(&mut *self.action_handler, virtual_view_id, action)
    }

    fn set_text_selection(
        &mut self,
        virtual_view_id: jint,
        anchor: jint,
        focus: jint,
    ) -> Option<QueuedEvents> {
        self.adapter
            .set_text_selection(&mut *self.action_handler, virtual_view_id, anchor, focus)
    }

    fn collapse_text_selection(&mut self, virtual_view_id: jint) -> Option<QueuedEvents> {
        self.adapter
            .collapse_text_selection(&mut *self.action_handler, virtual_view_id)
    }

    #[allow(clippy::too_many_arguments)]
    fn traverse_text(
        &mut self,
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    ) -> Option<QueuedEvents> {
        self.adapter.traverse_text(
            &mut *self.action_handler,
            virtual_view_id,
            granularity,
            forward,
            extend_selection,
        )
    }
}

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(0);
static HANDLE_MAP: Mutex<BTreeMap<jlong, Weak<Mutex<InnerInjectingAdapter>>>> =
    Mutex::new(BTreeMap::new());

fn inner_adapter_from_handle(handle: jlong) -> Option<Arc<Mutex<InnerInjectingAdapter>>> {
    let handle_map_guard = HANDLE_MAP.lock().unwrap();
    handle_map_guard.get(&handle).and_then(Weak::upgrade)
}

extern "system" fn populate_node_info(
    mut env: JNIEnv,
    _class: JClass,
    adapter_handle: jlong,
    host: JObject,
    host_screen_x: jint,
    host_screen_y: jint,
    virtual_view_id: jint,
    node_info: JObject,
) -> jboolean {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return JNI_FALSE;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    if inner_adapter
        .populate_node_info(
            &mut env,
            &host,
            host_screen_x,
            host_screen_y,
            virtual_view_id,
            &node_info,
        )
        .unwrap()
    {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

extern "system" fn get_input_focus(_env: JNIEnv, _class: JClass, adapter_handle: jlong) -> jint {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return HOST_VIEW_ID;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    inner_adapter.input_focus()
}

extern "system" fn get_virtual_view_at_point(
    _env: JNIEnv,
    _class: JClass,
    adapter_handle: jlong,
    x: jfloat,
    y: jfloat,
) -> jint {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return HOST_VIEW_ID;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    inner_adapter.virtual_view_at_point(x, y)
}

extern "system" fn perform_action(
    mut env: JNIEnv,
    class: JClass,
    adapter_handle: jlong,
    host: JObject,
    virtual_view_id: jint,
    action: jint,
) -> jboolean {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return JNI_FALSE;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    if let Some(events) = inner_adapter.perform_action(virtual_view_id, action) {
        events.raise(&mut env, &class, &host);
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

extern "system" fn set_text_selection(
    mut env: JNIEnv,
    class: JClass,
    adapter_handle: jlong,
    host: JObject,
    virtual_view_id: jint,
    anchor: jint,
    focus: jint,
) -> jboolean {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return JNI_FALSE;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    if let Some(events) = inner_adapter.set_text_selection(virtual_view_id, anchor, focus) {
        events.raise(&mut env, &class, &host);
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

extern "system" fn collapse_text_selection(
    mut env: JNIEnv,
    class: JClass,
    adapter_handle: jlong,
    host: JObject,
    virtual_view_id: jint,
) -> jboolean {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return JNI_FALSE;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    if let Some(events) = inner_adapter.collapse_text_selection(virtual_view_id) {
        events.raise(&mut env, &class, &host);
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

extern "system" fn traverse_text(
    mut env: JNIEnv,
    class: JClass,
    adapter_handle: jlong,
    host: JObject,
    virtual_view_id: jint,
    granularity: jint,
    forward: jboolean,
    extend_selection: jboolean,
) -> jboolean {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return JNI_FALSE;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    if let Some(events) = inner_adapter.traverse_text(
        virtual_view_id,
        granularity,
        forward == JNI_TRUE,
        extend_selection == JNI_TRUE,
    ) {
        events.raise(&mut env, &class, &host);
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

fn delegate_class(env: &mut JNIEnv) -> Result<&'static JClass<'static>> {
    static CLASS: OnceCell<GlobalRef> = OnceCell::new();
    let global = CLASS.get_or_try_init(|| {
        #[cfg(feature = "embedded-dex")]
        let class = {
            let dex_class_loader_class = env.find_class("dalvik/system/InMemoryDexClassLoader")?;
            let dex_bytes = include_bytes!("../classes.dex");
            let dex_buffer = unsafe {
                env.new_direct_byte_buffer(dex_bytes.as_ptr() as *mut u8, dex_bytes.len())
            }?;
            let dex_class_loader = env.new_object(
                &dex_class_loader_class,
                "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
                &[(&dex_buffer).into(), (&JObject::null()).into()],
            )?;
            let class_name = env.new_string("dev.accesskit.android.Delegate")?;
            let class_obj = env
                .call_method(
                    &dex_class_loader,
                    "loadClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[(&class_name).into()],
                )?
                .l()?;
            JClass::from(class_obj)
        };
        #[cfg(not(feature = "embedded-dex"))]
        let class = env.find_class("dev/accesskit/android/Delegate")?;
        env.register_native_methods(
            &class,
            &[
                NativeMethod {
                    name: "populateNodeInfo".into(),
                    sig: "(JLandroid/view/View;IIILandroid/view/accessibility/AccessibilityNodeInfo;)Z"
                        .into(),
                    fn_ptr: populate_node_info as *mut c_void,
                },
                NativeMethod {
                    name: "getInputFocus".into(),
                    sig: "(J)I".into(),
                    fn_ptr: get_input_focus as *mut c_void,
                },
                NativeMethod {
                    name: "getVirtualViewAtPoint".into(),
                    sig: "(JFF)I".into(),
                    fn_ptr: get_virtual_view_at_point as *mut c_void,
                },
                NativeMethod {
                    name: "performAction".into(),
                    sig: "(JLandroid/view/View;II)Z".into(),
                    fn_ptr: perform_action as *mut c_void,
                },
                NativeMethod {
                    name: "setTextSelection".into(),
                    sig: "(JLandroid/view/View;III)Z".into(),
                    fn_ptr: set_text_selection as *mut c_void,
                },
                NativeMethod {
                    name: "collapseTextSelection".into(),
                    sig: "(JLandroid/view/View;I)Z".into(),
                    fn_ptr: collapse_text_selection as *mut c_void,
                },
                NativeMethod {
                    name: "traverseText".into(),
                    sig: "(JLandroid/view/View;IIZZ)Z".into(),
                    fn_ptr: traverse_text as *mut c_void,
                },
            ],
        )?;
        env.new_global_ref(class)
    })?;
    Ok(global.as_obj().into())
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
        host_view: &JObject,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
    ) -> Result<Self> {
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
        let delegate_class = delegate_class(env)?;
        env.call_static_method(
            delegate_class,
            "inject",
            "(Landroid/view/View;J)V",
            &[host_view.into(), handle.into()],
        )?;
        Ok(Self {
            vm: env.get_java_vm()?,
            delegate_class,
            host: env.new_weak_ref(host_view)?.unwrap(),
            handle,
            inner,
        })
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
        if let Some(events) = self
            .inner
            .lock()
            .unwrap()
            .adapter
            .update_if_active(update_factory)
        {
            events.raise(&mut env, self.delegate_class, &host);
        }
    }
}

impl Drop for InjectingAdapter {
    fn drop(&mut self) {
        fn drop_impl(env: &mut JNIEnv, host: &WeakRef) -> Result<()> {
            let Some(host) = host.upgrade_local(env)? else {
                return Ok(());
            };
            let delegate_class = delegate_class(env)?;
            env.call_static_method(
                delegate_class,
                "remove",
                "(Landroid/view/View;)V",
                &[(&host).into()],
            )?;
            Ok(())
        }

        let res = match self.vm.get_env() {
            Ok(mut env) => drop_impl(&mut env, &self.host),
            Err(_) => self
                .vm
                .attach_current_thread()
                .and_then(|mut env| drop_impl(&mut env, &self.host)),
        };

        if let Err(err) = res {
            debug!("error dropping InjectingAdapter: {:#?}", err);
        }

        HANDLE_MAP.lock().unwrap().remove(&self.handle);
    }
}
