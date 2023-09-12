// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::Tree;
use std::cell::RefCell;

pub struct Adapter {
    tree: RefCell<Tree>,
    action_handler: Box<dyn ActionHandler>,
}

impl Adapter {
    pub fn new(
        parent_id: &str,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        Self {
            tree: RefCell::new(Tree::new(initial_state, true)),
            action_handler,
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        // TODO: events
        self.tree.borrow_mut().update(update);
    }
}
