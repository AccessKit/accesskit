// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use accesskit_schema::{NodeId, Role};
use std::sync::{Arc, Weak};

use crate::tree::{NodeState, Reader as TreeReader, Tree};
use crate::NodeData;

pub struct Node<'a> {
    pub tree_reader: &'a TreeReader<'a>,
    pub(crate) state: &'a NodeState,
}

impl Node<'_> {
    pub fn data(&self) -> &NodeData {
        &self.state.data
    }

    pub fn is_focused(&self) -> bool {
        return self.tree_reader.state.data.focus == Some(self.id())
    }

    pub fn is_ignored(&self) -> bool {
        self.data().ignored || (self.role() == Role::Presentation)
    }

    pub fn is_invisible_or_ignored(&self) -> bool {
        (self.is_invisible() || self.is_ignored()) && !self.is_focused()
    }

    // Convenience getters

    pub fn id(&self) -> NodeId {
        self.data().id
    }

    pub fn role(&self) -> Role {
        self.data().role
    }

    pub fn is_invisible(&self) -> bool {
        self.data().invisible
    }
}

#[derive(Clone)]
pub struct WeakNode {
    pub tree: Weak<Tree>,
    pub id: NodeId,
}

impl WeakNode {
    pub fn map<F, T>(&self, f: F) -> Option<T>
    where
        for<'a> F: FnOnce(&Node<'a>) -> T,
    {
        self.tree
            .upgrade()
            .map(|tree| tree.read().node_by_id(self.id).map(|node| f(&node)))
            .flatten()
    }
}

impl Node<'_> {
    pub fn downgrade(&self) -> WeakNode {
        WeakNode {
            tree: Arc::downgrade(self.tree_reader.tree),
            id: self.id(),
        }
    }
}
