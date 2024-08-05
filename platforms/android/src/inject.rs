// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from jni-rs
// Copyright 2016 Prevoty, Inc. and jni-rs contributors
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use jni::{
    errors::Result,
    objects::{GlobalRef, JClass, JObject, WeakRef},
    sys::jlong,
    JNIEnv, JavaVM,
};
use log::debug;
use once_cell::sync::OnceCell;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex, Weak,
    },
};

use crate::adapter::Adapter;

struct InnerInjectingAdapter {
    adapter: Adapter,
    action_handler: Box<dyn ActionHandler + Send>,
}

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(0);
static HANDLE_MAP: Mutex<BTreeMap<jlong, Weak<Mutex<InnerInjectingAdapter>>>> =
    Mutex::new(BTreeMap::new());

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
                "(Ljava/nio/ByteBUffer;Ljava/lang/ClassLoader;)V",
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
        // TODO: register JNI methods
        env.new_global_ref(class)
    })?;
    Ok(global.as_obj().into())
}

pub struct InjectingAdapter {
    vm: JavaVM,
    host: WeakRef,
    handle: jlong,
    inner: Arc<Mutex<InnerInjectingAdapter>>,
}

impl InjectingAdapter {
    pub fn new(
        env: &mut JNIEnv,
        host_view: &JObject,
        initial_state: TreeUpdate,
        action_handler: impl 'static + ActionHandler + Send,
    ) -> Result<Self> {
        let inner = Arc::new(Mutex::new(InnerInjectingAdapter {
            adapter: Adapter::new(initial_state),
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
            host: env.new_weak_ref(host_view)?.unwrap(),
            handle,
            inner,
        })
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
