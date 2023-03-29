// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::{FilterResult, Tree};
use objc2::{
    foundation::{MainThreadMarker, NSArray, NSObject, NSPoint},
    rc::{Id, Shared, WeakId},
};
use std::{ffi::c_void, ptr::null_mut, rc::Rc};

use crate::{
    appkit::NSView,
    context::Context,
    event::{EventGenerator, QueuedEvents},
    node::{can_be_focused, filter},
    util::*,
};

pub struct Adapter {
    context: Rc<Context>,
}

impl Adapter {
    /// Create a new macOS adapter. This function must be called on
    /// the main thread.
    ///
    /// The action handler will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    pub unsafe fn new(
        view: *mut c_void,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let view = unsafe { Id::retain(view as *mut NSView) }.unwrap();
        let view = WeakId::new(&view);
        let tree = Tree::new(initial_state);
        let mtm = MainThreadMarker::new().unwrap();
        Self {
            context: Context::new(view, tree, action_handler, mtm),
        }
    }

    /// Apply the provided update to the tree.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let mut event_generator = EventGenerator::new(self.context.clone());
        let mut tree = self.context.tree.borrow_mut();
        tree.update_and_process_changes(update, &mut event_generator);
        event_generator.into_result()
    }

    pub fn view_children(&self) -> *mut NSArray<NSObject> {
        let tree = self.context.tree.borrow();
        let state = tree.state();
        let node = state.root();
        let platform_nodes = if filter(&node) == FilterResult::Include {
            vec![Id::into_super(Id::into_super(
                self.context.get_or_create_platform_node(node.id()),
            ))]
        } else {
            node.filtered_children(filter)
                .map(|node| {
                    Id::into_super(Id::into_super(
                        self.context.get_or_create_platform_node(node.id()),
                    ))
                })
                .collect::<Vec<Id<NSObject, Shared>>>()
        };
        let array = NSArray::from_vec(platform_nodes);
        Id::autorelease_return(array)
    }

    pub fn focus(&self) -> *mut NSObject {
        let tree = self.context.tree.borrow();
        let state = tree.state();
        if let Some(node) = state.focus() {
            if can_be_focused(&node) {
                return Id::autorelease_return(self.context.get_or_create_platform_node(node.id()))
                    as *mut _;
            }
        }
        null_mut()
    }

    pub fn hit_test(&self, point: NSPoint) -> *mut NSObject {
        let view = match self.context.view.load() {
            Some(view) => view,
            None => {
                return null_mut();
            }
        };

        let tree = self.context.tree.borrow();
        let state = tree.state();
        let root = state.root();
        let point = from_ns_point(&view, &root, point);
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        Id::autorelease_return(self.context.get_or_create_platform_node(node.id())) as *mut _
    }
}
