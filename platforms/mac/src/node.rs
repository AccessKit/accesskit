// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#![allow(non_upper_case_globals)]

use std::ffi::c_void;

use accesskit_consumer::{Node, WeakNode};
use cocoa::base::{id, nil, BOOL, NO, YES};
use cocoa::foundation::{NSArray, NSPoint, NSSize, NSValue};
use lazy_static::lazy_static;
use objc::declare::ClassDecl;
use objc::rc::{StrongPtr, WeakPtr};
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::util::from_nsstring;

struct Attribute(*const id, fn(&State, &Node) -> id);
unsafe impl Sync for Attribute {}

fn get_position(_state: &State, node: &Node) -> id {
    if let Some(bounds) = &node.data().bounds {
        // TODO: implement for real
        let ns_point = NSPoint { x: 100., y: 100. };
        unsafe { NSValue::valueWithPoint(nil, ns_point) }
    } else {
        nil
    }
}

fn get_role(_state: &State, node: &Node) -> id {
    // TODO: implement for real
    unsafe { NSAccessibilityWindowRole }
}

fn get_size(_state: &State, node: &Node) -> id {
    if let Some(bounds) = &node.data().bounds {
        let ns_size = NSSize {
            width: bounds.rect.width as f64,
            height: bounds.rect.height as f64,
        };
        unsafe { NSValue::valueWithSize(nil, ns_size) }
    } else {
        nil
    }
}

static ATTRIBUTE_MAP: &[Attribute] = unsafe {
    &[
        Attribute(&NSAccessibilityPositionAttribute, get_position),
        Attribute(&NSAccessibilityRoleAttribute, get_role),
        Attribute(&NSAccessibilitySizeAttribute, get_size),
    ]
};

struct State {
    node: WeakNode,
    view: WeakPtr,
}

impl State {
    fn attribute_names(&self) -> id {
        let names = ATTRIBUTE_MAP
            .iter()
            .map(|Attribute(name_ptr, _)| unsafe { **name_ptr })
            .collect::<Vec<id>>();
        // TODO: role-specific attributes
        println!("returning attribute names {:?}", names);
        unsafe { NSArray::arrayWithObjects(nil, &names) }
    }

    fn attribute_value(&self, attribute_name: id) -> id {
        self.node
            .map(|node| {
                println!("get attribute value {}", from_nsstring(attribute_name));

                for Attribute(test_name_ptr, f) in ATTRIBUTE_MAP {
                    let equal: BOOL = unsafe {
                        let test_name: id = **test_name_ptr;
                        msg_send![attribute_name, isEqualToString: test_name]
                    };
                    if equal == YES {
                        return f(&self, &node);
                    }
                }

                nil
            })
            .unwrap_or(nil)
    }

    fn is_ignored(&self) -> BOOL {
        self.node
            .map(|node| {
                if node.is_invisible_or_ignored() {
                    YES
                } else {
                    NO
                }
            })
            .unwrap_or(YES)
    }
}

pub(crate) struct PlatformNode;

impl PlatformNode {
    pub(crate) fn new(node: &Node, view: &StrongPtr) -> StrongPtr {
        let state = Box::new(State {
            node: node.downgrade(),
            view: view.weak(),
        });
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

        decl.add_method(sel!(accessibilityAttributeNames), attribute_names as extern "C" fn(&Object, Sel) -> id);
        extern "C" fn attribute_names(this: &Object, _sel: Sel) -> id {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                let state = &mut *(state as *mut State);
                state.attribute_names()
            }
        }

        decl.add_method(sel!(accessibilityAttributeValue:), attribute_value as extern "C" fn(&Object, Sel, id) -> id);
        extern "C" fn attribute_value(this: &Object, _sel: Sel, attribute_name: id) -> id {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                let state = &mut *(state as *mut State);
                state.attribute_value(attribute_name)
            }
        }

        decl.add_method(sel!(accessibilityIsIgnored), is_ignored as extern "C" fn(&Object, Sel) -> BOOL);
        extern "C" fn is_ignored(this: &Object, _sel: Sel) -> BOOL {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                let state = &mut *(state as *mut State);
                state.is_ignored()
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

// Constants declared in AppKit
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // Attributes
    static NSAccessibilityPositionAttribute: id;
    static NSAccessibilityRoleAttribute: id;
    static NSAccessibilitySizeAttribute: id;

    // Roles
    static NSAccessibilityWindowRole: id;
}
