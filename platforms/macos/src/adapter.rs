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
    rc::{Id, Shared},
};

use crate::{appkit::NSView, node::PlatformNode};

pub struct Adapter {
    pub(crate) view: Id<NSView, Shared>,
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

    pub fn view_children(&self) -> *mut NSArray<NSObject> {
        // TODO: return unignored children
        let platform_node = self.root_platform_node();
        let ids = vec![platform_node];
        let array = NSArray::from_vec(ids);
        Id::autorelease_return(array)
    }
}
