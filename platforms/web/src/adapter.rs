// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActivationHandler, NodeId, TreeUpdate};
use accesskit_consumer::{FilterResult, Node, Tree, TreeChangeHandler, TreeState};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement};

use crate::{filters::filter, node::NodeWrapper};

enum State {
    Pending {
        is_host_focused: bool,
        document: Document,
        parent: Element,
    },
    Active {
        tree: Tree,
        document: Document,
        root: HtmlElement,
        elements: HashMap<NodeId, HtmlElement>,
    },
}

pub struct Adapter {
    state: State,
    action_handler: Box<dyn ActionHandler>,
}

impl Adapter {
    pub fn new(
        parent_id: &str,
        mut activation_handler: impl ActivationHandler,
        action_handler: impl 'static + ActionHandler + Send,
    ) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent = document.get_element_by_id(parent_id).unwrap();

        let state = match activation_handler.request_initial_tree() {
            Some(initial_state) => {
                let tree = Tree::new(initial_state, true);
                let (root, elements) = add_initial_tree(&document, &parent, &tree);
                State::Active {
                    tree,
                    document,
                    root,
                    elements,
                }
            }
            None => State::Pending {
                is_host_focused: true,
                document,
                parent,
            },
        };
        Self {
            state,
            action_handler: Box::new(action_handler),
        }
    }

    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) {
        match &mut self.state {
            State::Pending {
                is_host_focused,
                document,
                parent,
            } => {
                let tree = Tree::new(update_factory(), *is_host_focused);
                let (root, elements) = add_initial_tree(document, parent, &tree);
                self.state = State::Active {
                    tree,
                    document: document.clone(),
                    root,
                    elements,
                };
            }
            State::Active {
                tree,
                document,
                elements,
                ..
            } => {
                let mut handler = AdapterChangeHandler { document, elements };
                tree.update_and_process_changes(update_factory(), &mut handler);
            }
        }
    }

    pub fn update_host_focus_state(&mut self, is_focused: bool) {
        match &mut self.state {
            State::Pending {
                is_host_focused, ..
            } => *is_host_focused = is_focused,
            State::Active {
                tree,
                document,
                elements,
                ..
            } => {
                let mut handler = AdapterChangeHandler { document, elements };
                tree.update_host_focus_state_and_process_changes(is_focused, &mut handler);
            }
        }
    }
}

fn add_initial_tree(
    document: &Document,
    parent: &Element,
    tree: &Tree,
) -> (HtmlElement, HashMap<NodeId, HtmlElement>) {
    let root = document
        .create_element("div")
        .unwrap()
        .unchecked_into::<HtmlElement>();
    root.set_attribute("role", "application").unwrap();
    parent.append_child(&root).unwrap();
    let mut elements = HashMap::new();
    let root_node = tree.state().root();
    add_element_recursive(document, &root, &root_node, &mut elements);
    if let Some(focus_id) = tree.state().focus_id() {
        if let Some(element) = elements.get(&focus_id) {
            focus(element);
        }
    }
    (root, elements)
}

fn add_element(
    document: &Document,
    parent: &HtmlElement,
    node: &Node,
    elements: &mut HashMap<NodeId, HtmlElement>,
) -> HtmlElement {
    let element = document
        .create_element("div")
        .unwrap()
        .unchecked_into::<HtmlElement>();
    let wrapper = NodeWrapper(*node);
    wrapper.set_all_attributes(&element);
    parent.append_child(&element).unwrap();
    elements.insert(node.id(), element.clone());
    element
}

fn add_element_recursive(
    document: &Document,
    parent: &HtmlElement,
    node: &Node,
    elements: &mut HashMap<NodeId, HtmlElement>,
) {
    let element = add_element(document, parent, node, elements);
    for child in node.filtered_children(&filter) {
        add_element_recursive(document, &element, &child, elements);
    }
}

fn focus(element: &HtmlElement) {
    element.focus().unwrap();
}

fn blur(element: &HtmlElement) {
    element.blur().unwrap();
}

struct AdapterChangeHandler<'a> {
    document: &'a Document,
    elements: &'a mut HashMap<NodeId, HtmlElement>,
}

impl TreeChangeHandler for AdapterChangeHandler<'_> {
    fn node_added(&mut self, node: &Node) {
        if filter(node) != FilterResult::Include {
            return;
        }
        if self.elements.contains_key(&node.id()) {
            return;
        }
        let parent = node.filtered_parent(&filter).unwrap();
        let parent_element = self.elements.get(&parent.id()).unwrap().clone();
        add_element(self.document, &parent_element, node, self.elements);
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

    fn focus_moved(&mut self, old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            if let Some(element) = self.elements.get(&new_node.id()) {
                focus(element);
            }
        } else if let Some(old_node) = old_node {
            if let Some(element) = self.elements.get(&old_node.id()) {
                blur(element);
            }
        }
    }

    fn node_removed(&mut self, node: &Node) {
        if let Some(element) = self.elements.remove(&node.id()) {
            element.remove();
        }
    }
}
