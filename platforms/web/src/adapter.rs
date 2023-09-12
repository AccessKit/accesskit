// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, NodeId, TreeUpdate};
use accesskit_consumer::{DetachedNode, Node, Tree, TreeChangeHandler, TreeState};
use std::collections::HashMap;
use web_sys::Element;

pub struct Adapter {
    tree: Tree,
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
            tree: Tree::new(initial_state, true),
            action_handler,
            root,
            elements: HashMap::new(),
        }
    }

    pub fn update(&mut self, update: TreeUpdate) {
        let mut handler = AdapterChangeHandler {
            elements: &mut self.elements,
        };
        self.tree.update_and_process_changes(update, &mut handler);
    }
}

struct AdapterChangeHandler<'a> {
    elements: &'a mut HashMap<NodeId, Element>,
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
    fn node_added(&mut self, node: &Node) {
        // TODO
    }

    fn node_updated(&mut self, old_node: &DetachedNode, new_node: &Node) {
        // TODO
    }

    fn focus_moved(
        &mut self,
        _old_node: Option<&DetachedNode>,
        new_node: Option<&Node>,
        _current_state: &TreeState,
    ) {
        // TODO
    }

    fn node_removed(&mut self, node: &DetachedNode, current_state: &TreeState) {
        // TODO
    }
}
