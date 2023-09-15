// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, NodeId, TreeUpdate};
use accesskit_consumer::{FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use std::collections::HashMap;
use web_sys::{Document, Element};

use crate::{filters::filter, node::NodeWrapper};

pub struct Adapter {
    tree: Tree,
    action_handler: Box<dyn ActionHandler>,
    document: Document,
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

        let tree = Tree::new(initial_state, true);
        let mut elements = HashMap::new();
        let root_node = tree.state().root();
        add_element(&document, &root, &root_node, &mut elements);
        Self {
            tree,
            action_handler,
            document,
            root,
            elements,
        }
    }

    pub fn update(&mut self, update: TreeUpdate) {
        let mut handler = AdapterChangeHandler {
            elements: &mut self.elements,
        };
        self.tree.update_and_process_changes(update, &mut handler);
    }
}

fn add_element(
    document: &Document,
    parent: &Element,
    node: &Node,
    elements: &mut HashMap<NodeId, Element>,
) {
    let element = document.create_element("div").unwrap();
    let wrapper = NodeWrapper(*node);
    wrapper.set_all_attributes(&element);
    parent.append_child(&element).unwrap();
    for child in node.filtered_children(&filter) {
        add_element(document, &element, &child, elements);
    }
    elements.insert(node.id(), element);
}

struct AdapterChangeHandler<'a> {
    elements: &'a mut HashMap<NodeId, Element>,
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
    fn node_added(&mut self, node: &Node) {
        if filter(node) != FilterResult::Include {
            return;
        }
        // TODO
    }

    fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
        if filter(new_node) != FilterResult::Include {
            return;
        }
        let element = match self.elements.get(&new_node.id()) {
            Some(element) => element,
            None => {
                return;
            }
        };
        let old_wrapper = NodeWrapper(*old_node);
        let new_wrapper = NodeWrapper(*new_node);
        new_wrapper.update_attributes(element, &old_wrapper);
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        // TODO
    }

    fn node_removed(&mut self, node: &Node) {
        // TODO
    }
}
