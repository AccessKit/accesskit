// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use jni::{objects::JObject, sys::jint, JNIEnv};

use crate::util::*;

pub(crate) enum PlatformActionInner {
    Simple {
        action: jint,
    },
    SetTextSelection {
        anchor: jint,
        focus: jint,
    },
    CollapseTextSelection,
    TraverseText {
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    },
}

pub struct PlatformAction(pub(crate) PlatformActionInner);

impl PlatformAction {
    pub fn from_java(env: &mut JNIEnv, action: jint, arguments: &JObject) -> Option<Self> {
        match action {
            ACTION_SET_SELECTION => {
                if !(!arguments.is_null()
                    && bundle_contains_key(env, arguments, ACTION_ARGUMENT_SELECTION_START_INT)
                    && bundle_contains_key(env, arguments, ACTION_ARGUMENT_SELECTION_END_INT))
                {
                    return Some(Self(PlatformActionInner::CollapseTextSelection));
                }
                let anchor = bundle_get_int(env, arguments, ACTION_ARGUMENT_SELECTION_START_INT);
                let focus = bundle_get_int(env, arguments, ACTION_ARGUMENT_SELECTION_END_INT);
                Some(Self(PlatformActionInner::SetTextSelection {
                    anchor,
                    focus,
                }))
            }
            ACTION_NEXT_AT_MOVEMENT_GRANULARITY | ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY => {
                if !(!arguments.is_null()
                    && bundle_contains_key(
                        env,
                        arguments,
                        ACTION_ARGUMENT_MOVEMENT_GRANULARITY_INT,
                    )
                    && bundle_contains_key(
                        env,
                        arguments,
                        ACTION_ARGUMENT_EXTEND_SELECTION_BOOLEAN,
                    ))
                {
                    return None;
                }
                let granularity =
                    bundle_get_int(env, arguments, ACTION_ARGUMENT_MOVEMENT_GRANULARITY_INT);
                let forward = action == ACTION_NEXT_AT_MOVEMENT_GRANULARITY;
                let extend_selection =
                    bundle_get_bool(env, arguments, ACTION_ARGUMENT_EXTEND_SELECTION_BOOLEAN);
                Some(Self(PlatformActionInner::TraverseText {
                    granularity,
                    forward,
                    extend_selection,
                }))
            }
            _ => Some(Self(PlatformActionInner::Simple { action })),
        }
    }
}
