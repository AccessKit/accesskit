// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::ffi::c_void;

use accesskit_consumer::{Node, WeakNode};
use cocoa::base::{id, nil};
use lazy_static::lazy_static;
use objc::declare::ClassDecl;
use objc::rc::StrongPtr;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::util::from_nsstring;

struct State {
    node: WeakNode,
}

impl State {
    fn attribute_value(&self, attribute_name: id) -> id {
        println!("get attribute value {}", from_nsstring(attribute_name));
        nil
    }
}

pub(crate) struct PlatformNode;

impl PlatformNode {
    pub(crate) fn new(node: &Node) -> StrongPtr {
        let state = Box::new(State { node: node.downgrade() });
        unsafe {
            let object: id = msg_send![PLATFORM_NODE_CLASS.0, alloc];
            let () = msg_send![object, init];
            let state_ptr = Box::into_raw(state);
            (*object).set_ivar(STATE_IVAR, state_ptr as *mut c_void);
            StrongPtr::new(object)
        }
    }
}

static STATE_IVAR: &str = "accessKitPlatformNodeState";

struct PlatformNodeClass(*const Class);
unsafe impl Sync for PlatformNodeClass {}

lazy_static! {
    static ref PLATFORM_NODE_CLASS: PlatformNodeClass = unsafe {
        let mut decl = ClassDecl::new("AccessKitPlatformNode", class!(NSObject))
            .expect("platform node class definition failed");
        decl.add_ivar::<*mut c_void>(STATE_IVAR);

        // TODO: methods

        decl.add_method(sel!(accessibilityAttributeValue:), attribute_value as extern "C" fn(&Object, Sel, id) -> id);
        extern "C" fn attribute_value(this: &Object, _sel: Sel, attribute_name: id) -> id {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                let state = &mut *(state as *mut State);
                state.attribute_value(attribute_name)
            }
        }

        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        extern "C" fn dealloc(this: &Object, _sel: Sel) {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                Box::from_raw(state as *mut State); // drops the state
            }
        }

        PlatformNodeClass(decl.register())
    };
}
