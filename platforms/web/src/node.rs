// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, TreeState};

use crate::{
    filters::{filter, filter_detached, filter_with_root_exception},
};

pub(crate) enum NodeWrapper<'a> {
    Node(&'a Node<'a>),
    DetachedNode(&'a DetachedNode),
}

impl<'a> NodeWrapper<'a> {
    fn node_state(&self) -> &'a NodeState {
        match self {
            Self::Node(node) => node.state(),
            Self::DetachedNode(node) => node.state(),
        }
    }

    fn role(&self) -> Option<String> {
        let role = self.node_state().role();
        todo!()
    }

    fn name(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    fn value(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.value(),
            Self::DetachedNode(node) => node.value(),
        }
    }
}

macro_rules! attributes {
    ($(($name:literal, $m:ident)),+) => {
        impl NodeWrapper<'_> {
            fn set_all_attributes(&self) {
                // TODO: add the element as a parameter
                $(let value = self.$m();
                // TODO: set the attribute if necessary
                )*
            }
        }
    };
}

attributes! {
    ("role", role),
    ("aria-label", name)
}
