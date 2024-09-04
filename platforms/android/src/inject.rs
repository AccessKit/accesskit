// Copyright 2024 The AccessKit Authors. All rights reserved.
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
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex, Weak,
    },
};

use crate::{adapter::Adapter, util::*};

struct InnerInjectingAdapter {
    adapter: Adapter,
    activation_handler: Box<dyn ActivationHandler + Send>,
    action_handler: Box<dyn ActionHandler + Send>,
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

    fn virtual_view_at_point(&mut self, x: jfloat, y: jfloat) -> jint {
        self.adapter
            .virtual_view_at_point(&mut *self.activation_handler, x, y)
    }

    fn perform_action(
        &mut self,
        env: &mut JNIEnv,
        host: &JObject,
        virtual_view_id: jint,
        action: jint,
        arguments: &JObject,
    ) -> Result<bool> {
        self.adapter.perform_action(
            &mut *self.action_handler,
            env,
            host,
            virtual_view_id,
            action,
            arguments,
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

extern "system" fn perform_action(
    mut env: JNIEnv,
    _class: JClass,
    adapter_handle: jlong,
    host: JObject,
    virtual_view_id: jint,
    action: jint,
    arguments: JObject,
) -> jboolean {
    let Some(inner_adapter) = inner_adapter_from_handle(adapter_handle) else {
        return JNI_FALSE;
    };
    let mut inner_adapter = inner_adapter.lock().unwrap();
    if inner_adapter
        .perform_action(&mut env, &host, virtual_view_id, action, &arguments)
        .unwrap()
    {
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
                    name: "performAction".into(),
                    sig: "(JLandroid/view/View;IILandroid/os/Bundle;)Z"
                        .into(),
                    fn_ptr: perform_action as *mut c_void,
                },
            ],
        )?;
        env.new_global_ref(class)
    })?;
    Ok(global.as_obj().into())
}

pub struct InjectingAdapter {
    vm: JavaVM,
    delegate_class: &'static JClass<'static>,
    host: WeakRef,
    handle: jlong,
    inner: Arc<Mutex<InnerInjectingAdapter>>,
    hover_view_id: jint,
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
            hover_view_id: HOST_VIEW_ID,
        })
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update. Note: If the caller's implementation of
    /// [`ActivationHandler::request_initial_tree`] initially returned `None`,
    /// the [`TreeUpdate`] returned by the provided function must contain
    /// a full tree.
    ///
    /// TODO: dispatch events
    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        let mut env = self.vm.get_env().unwrap();
        let Some(host) = self.host.upgrade_local(&env).unwrap() else {
            return;
        };
        self.inner.lock().unwrap().adapter.update_if_active(
            update_factory,
            &mut env,
            self.delegate_class,
            &host,
        );
    }

    pub fn handle_hover_enter_or_move(&mut self, x: jfloat, y: jfloat) {
        let old_id = self.hover_view_id;
        let new_id = self.inner.lock().unwrap().virtual_view_at_point(x, y);
        if new_id == old_id {
            return;
        }
        let mut env = self.vm.get_env().unwrap();
        let Some(host) = self.host.upgrade_local(&env).unwrap() else {
            return;
        };
        self.hover_view_id = new_id;
        if new_id != HOST_VIEW_ID {
            send_event(
                &mut env,
                self.delegate_class,
                &host,
                new_id,
                EVENT_VIEW_HOVER_ENTER,
            );
        }
        if old_id != HOST_VIEW_ID {
            send_event(
                &mut env,
                self.delegate_class,
                &host,
                old_id,
                EVENT_VIEW_HOVER_EXIT,
            );
        }
    }

    pub fn handle_hover_exit(&mut self) {
        if self.hover_view_id == HOST_VIEW_ID {
            return;
        }
        let old_id = self.hover_view_id;
        self.hover_view_id = HOST_VIEW_ID;
        let mut env = self.vm.get_env().unwrap();
        let Some(host) = self.host.upgrade_local(&env).unwrap() else {
            return;
        };
        send_event(
            &mut env,
            self.delegate_class,
            &host,
            old_id,
            EVENT_VIEW_HOVER_EXIT,
        );
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
