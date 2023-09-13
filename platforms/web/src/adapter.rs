// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, NodeId, TreeUpdate};
use accesskit_consumer::Tree;
use std::{cell::RefCell, collections::HashMap};
use web_sys::Element;

pub struct Adapter {
    tree: RefCell<Tree>,
    action_handler: Box<dyn ActionHandler>,
    root: Element,
    elements: HashMap<NodeId, Element>,
}

impl Adapter {
    pub fn new(
        parent_id: &str,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent = document.get_element_by_id(parent_id).unwrap();
        let root = document.create_element("div").unwrap();
        parent.append_child(&root).unwrap();
        Self {
            tree: RefCell::new(Tree::new(initial_state, true)),
            action_handler,
            root,
            elements: HashMap::new(),
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        // TODO: events
        self.tree.borrow_mut().update(update);
    }
}
