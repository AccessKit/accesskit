// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::sync::Arc;

use accesskit_consumer::Tree;
use accesskit_schema::TreeUpdate;
use cocoa::base::{id, nil};
use cocoa::foundation::NSArray;
use objc::rc::StrongPtr;
use objc::{msg_send, sel, sel_impl};

use crate::node::PlatformNode;

pub struct Manager {
    view: StrongPtr,
    tree: Arc<Tree>,
}

impl Manager {
    pub fn new(view: id, initial_state: TreeUpdate) -> Self {
        assert!(!view.is_null());
        Self {
            view: unsafe { StrongPtr::retain(view) },
            tree: Tree::new(initial_state),
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.tree.update(update);
        // TODO: events
    }

    pub fn root_platform_node(&self) -> StrongPtr {
        let reader = self.tree.read();
        let node = reader.root();
        PlatformNode::get_or_create(&node, &self.view)
    }

    /// Inject accessibility into the view specified at initialization time.
    /// This is useful when working with libraries that create an NSView
    /// and don't provide an easy way to customize it.
    // TODO: In the common case where the role of the tree root is Window,
    // the approach taken by this function doesn't work well, because
    // it introduces an extraneous "group" object as the one child
    // of the view. It should really inject all of the unignored children
    // of the root. But that list of children is often dynamic,
    // so we need something different than the static solution used here.
    // Maybe we can patch the accessibilityChildren method on the view?
    pub fn inject(&self) {
        let platform_node = self.root_platform_node();
        let ids = [*platform_node];
        unsafe {
            let array = NSArray::arrayWithObjects(nil, &ids);
            let () = msg_send![*self.view, setAccessibilityChildren: array];
            let description: id = msg_send![*self.view, debugDescription];
            println!("injected into {}", crate::util::from_nsstring(description));
        }
    }
}
