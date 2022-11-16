// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::ffi::c_void;
use std::sync::Arc;

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::Tree;
use objc2::{
    foundation::{NSArray, NSObject},
    msg_send,
    rc::{Id, Shared},
};

use crate::{appkit::NSView, node::PlatformNode};

pub struct Adapter {
    view: Id<NSView, Shared>,
    tree: Arc<Tree>,
}

impl Adapter {
    /// Create a new macOS adapter.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    /// This method will retain an additional reference to `view`.
    pub unsafe fn new(
        view: *mut c_void,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        Self {
            view: Id::retain(view as *mut _).unwrap(),
            tree: Arc::new(Tree::new(initial_state, action_handler)),
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        self.tree.update(update);
        // TODO: events
    }

    pub fn root_platform_node(&self) -> Id<NSObject, Shared> {
        let state = self.tree.read();
        let node = state.root();
        Id::into_super(PlatformNode::get_or_create(
            &node,
            Arc::downgrade(&self.tree),
            &self.view,
        ))
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
        let ids = vec![platform_node];
        let array = NSArray::from_vec(ids);
        unsafe {
            let () = msg_send![&self.view, setAccessibilityChildren: &*array];
        }
    }
}
