// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest, NodeId};
use accesskit_consumer::Tree;
use icrate::{AppKit::*, Foundation::MainThreadMarker};
use objc2::rc::{Id, WeakId};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::node::PlatformNode;

pub(crate) struct Context {
    pub(crate) view: WeakId<NSView>,
    pub(crate) tree: RefCell<Tree>,
    pub(crate) action_handler: RefCell<Box<dyn ActionHandler>>,
    platform_nodes: RefCell<HashMap<NodeId, Id<PlatformNode>>>,
    _mtm: MainThreadMarker,
}

impl Context {
    pub(crate) fn new(
        view: WeakId<NSView>,
        tree: Tree,
        action_handler: Box<dyn ActionHandler>,
        mtm: MainThreadMarker,
    ) -> Rc<Self> {
        Rc::new(Self {
            view,
            tree: RefCell::new(tree),
            action_handler: RefCell::new(action_handler),
            platform_nodes: RefCell::new(HashMap::new()),
            _mtm: mtm,
        })
    }

    pub(crate) fn get_or_create_platform_node(self: &Rc<Self>, id: NodeId) -> Id<PlatformNode> {
        let mut platform_nodes = self.platform_nodes.borrow_mut();
        if let Some(result) = platform_nodes.get(&id) {
            return result.clone();
        }

        let result = PlatformNode::new(Rc::downgrade(self), id);
        platform_nodes.insert(id, result.clone());
        result
    }

    pub(crate) fn remove_platform_node(&self, id: NodeId) -> Option<Id<PlatformNode>> {
        let mut platform_nodes = self.platform_nodes.borrow_mut();
        platform_nodes.remove(&id)
    }

    pub(crate) fn do_action(&self, request: ActionRequest) {
        self.action_handler.borrow_mut().do_action(request);
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let platform_nodes = self.platform_nodes.borrow();
        for platform_node in platform_nodes.values() {
            unsafe {
                NSAccessibilityPostNotification(
                    platform_node,
                    NSAccessibilityUIElementDestroyedNotification,
                )
            };
        }
    }
}
