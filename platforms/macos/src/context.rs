// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use accesskit_consumer::Tree;
use objc2::rc::{Id, Shared};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

use crate::{appkit::NSView, node::PlatformNode};

struct PlatformNodePtr(Id<PlatformNode, Shared>);
unsafe impl Send for PlatformNodePtr {}

pub(crate) struct Context {
    pub(crate) view: Id<NSView, Shared>,
    pub(crate) tree: Tree,
    platform_nodes: Mutex<HashMap<NodeId, PlatformNodePtr>>,
}

impl Context {
    pub(crate) fn new(view: Id<NSView, Shared>, tree: Tree) -> Arc<Self> {
        Arc::new(Self {
            view,
            tree,
            platform_nodes: Mutex::new(HashMap::new()),
        })
    }

    pub(crate) fn get_or_create_platform_node(
        self: &Arc<Self>,
        id: NodeId,
    ) -> Id<PlatformNode, Shared> {
        let mut platform_nodes = self.platform_nodes.lock();
        if let Some(result) = platform_nodes.get(&id) {
            return result.0.clone();
        }

        let result = PlatformNode::new(Arc::downgrade(self), id);
        platform_nodes.insert(id, PlatformNodePtr(result.clone()));
        result
    }

    pub(crate) fn remove_platform_node(&self, id: NodeId) -> Option<Id<PlatformNode, Shared>> {
        let mut platform_nodes = self.platform_nodes.lock();
        platform_nodes.remove(&id).map(|ptr| ptr.0)
    }
}
