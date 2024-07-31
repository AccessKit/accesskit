// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use jni::{
    errors::Result,
    objects::{GlobalRef, JMethodID, JObject, JValueOwned},
    signature::{Primitive, ReturnType},
    sys::jvalue,
    JNIEnv,
};

macro_rules! java_class {
    (
        package $package_name:literal;
        class $class_name:ident {
        $(method $method_return_type:literal $method_name:ident($($method_arg_type:literal $method_arg_name:ident,)*);)*}
    ) => {
        paste::paste! {
            #[derive(Clone)]
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            pub(crate) struct $class_name {
                _class: GlobalRef,
                $([<$method_name _id>]: JMethodID,)*
            }
            #[allow(non_snake_case)]
            impl $class_name {
                pub(crate) fn initialize_class(env: &mut JNIEnv) -> Result<Self> {
                    let class = env.find_class(concat!($package_name, "/", stringify!($class_name)))?;
                    Ok(Self {
                        _class: env.new_global_ref(&class)?,
                        $([<$method_name _id>]: env.get_method_id(
                            &class,
                            stringify!($method_name),
                            concat!("(", $($method_arg_type,)* ")", $method_return_type),
                        )?,)*
                    })
                }
                $(#[inline]
                pub(crate) fn $method_name<'local, 'other_local, O>(
                    &self,
                    env: &mut JNIEnv<'local>,
                    instance: O,
                    $($method_arg_name: jvalue,)*
                ) -> Result<JValueOwned<'local>>
                where O: AsRef<JObject<'other_local>>
                {
                    unsafe {
                        env.call_method_unchecked(
                            instance,
                            self.[<$method_name _id>],
                            return_type!($method_return_type),
                            &[$($method_arg_name),*]
                        )
                    }
                })*
            }
        }
    }
}

macro_rules! return_type {
    ("V") => {
        ReturnType::Primitive(Primitive::Void)
    };
}

java_class! {
    package "android/view/accessibility";

    class AccessibilityNodeInfo {
        method "V" addChild("Landroid/view/View;" view, "I" virtual_descendant_id,);
        method "V" setCheckable("Z" checkable,);
        method "V" setChecked("Z" checked,);
        method "V" setEnabled("Z" enabled,);
        method "V" setFocusable("Z" focusable,);
        method "V" setFocused("Z" focused,);
        method "V" setParent("Landroid/view/View;" view, "I" virtual_descendant_id,);
        method "V" setPassword("Z" password,);
        method "V" setSelected("Z" selected,);
        method "V" setText("Ljava/lang/CharSequence;" text,);
    }
}
