// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::Arc;

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::{FilterResult, Tree};
use objc2::{
    foundation::{NSArray, NSObject},
    rc::{Id, Shared},
};

use crate::{
    appkit::NSView,
    event::{EventGenerator, QueuedEvents},
    node::{filter, PlatformNode},
};

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

    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let mut event_generator =
            EventGenerator::new(Arc::downgrade(&self.tree), self.view.clone());
        self.tree
            .update_and_process_changes(update, &mut event_generator);
        event_generator.into_result()
    }

    pub fn view_children(&self) -> *mut NSArray<NSObject> {
        let state = self.tree.read();
        let node = state.root();
        let tree = Arc::downgrade(&self.tree);
        let platform_nodes = if filter(&node) == FilterResult::Include {
            vec![Id::into_super(Id::into_super(PlatformNode::get_or_create(
                node.id(),
                &tree,
                &self.view,
            )))]
        } else {
            node.filtered_children(filter)
                .map(|node| {
                    Id::into_super(Id::into_super(PlatformNode::get_or_create(
                        node.id(),
                        &tree,
                        &self.view,
                    )))
                })
                .collect::<Vec<Id<NSObject, Shared>>>()
        };
        let array = NSArray::from_vec(platform_nodes);
        Id::autorelease_return(array)
    }

    pub fn focus(&self) -> *mut NSObject {
        let state = self.tree.read();
        if let Some(node) = state.focus() {
            if filter(&node) == FilterResult::Include {
                let tree = Arc::downgrade(&self.tree);
                return Id::autorelease_return(PlatformNode::get_or_create(
                    node.id(),
                    &tree,
                    &self.view,
                )) as *mut _;
            }
        }
        null_mut()
    }
}
