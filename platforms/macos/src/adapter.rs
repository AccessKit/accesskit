// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::{FilterResult, Tree};
use objc2::{
    foundation::{NSArray, NSObject},
    rc::{Id, Shared},
};
use std::{ffi::c_void, ptr::null_mut, sync::Arc};

use crate::{
    appkit::NSView,
    context::Context,
    event::{EventGenerator, QueuedEvents},
    node::filter,
};

pub struct Adapter {
    pub(crate) context: Arc<Context>,
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
        let view = Id::retain(view as *mut NSView).unwrap();
        let tree = Tree::new(initial_state, action_handler);
        Self {
            context: Context::new(view, tree),
        }
    }

    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let mut event_generator = EventGenerator::new(self.context.clone());
        self.context
            .tree
            .update_and_process_changes(update, &mut event_generator);
        event_generator.into_result()
    }

    pub fn view_children(&self) -> *mut NSArray<NSObject> {
        let state = self.context.tree.read();
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
        let state = self.context.tree.read();
        if let Some(node) = state.focus() {
            if filter(&node) == FilterResult::Include {
                return Id::autorelease_return(self.context.get_or_create_platform_node(node.id()))
                    as *mut _;
            }
        }
        null_mut()
    }
}
