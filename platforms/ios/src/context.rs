// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::{NodeId, Tree};
use hashbrown::HashMap;
use objc2::rc::{Retained, WeakId};
use objc2_foundation::MainThreadMarker;
use objc2_ui_kit::UIView;
use std::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

use crate::node::PlatformNode;

pub(crate) trait ActionHandlerNoMut {
    fn do_action(&self, request: ActionRequest);
}

pub(crate) struct ActionHandlerWrapper<H: ActionHandler>(RefCell<H>);

impl<H: 'static + ActionHandler> ActionHandlerWrapper<H> {
    pub(crate) fn new(inner: H) -> Self {
        Self(RefCell::new(inner))
    }
}

impl<H: ActionHandler> ActionHandlerNoMut for ActionHandlerWrapper<H> {
    fn do_action(&self, request: ActionRequest) {
        self.0.borrow_mut().do_action(request)
    }
}

pub(crate) struct Context {
    pub(crate) view: WeakId<UIView>,
    pub(crate) tree: RefCell<Tree>,
    pub(crate) action_handler: Rc<dyn ActionHandlerNoMut>,
    platform_nodes: RefCell<HashMap<NodeId, Retained<PlatformNode>>>,
    pub(crate) platform_focus: RefCell<Option<NodeId>>,
    pub(crate) mtm: MainThreadMarker,
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("view", &self.view)
            .field("tree", &self.tree)
            .field("action_handler", &"ActionHandler")
            .field("platform_nodes", &self.platform_nodes)
            .field("platform_focus", &self.platform_focus)
            .field("mtm", &self.mtm)
            .finish()
    }
}

impl Context {
    pub(crate) fn new(
        view: WeakId<UIView>,
        tree: Tree,
        action_handler: Rc<dyn ActionHandlerNoMut>,
        mtm: MainThreadMarker,
    ) -> Rc<Self> {
        Rc::new(Self {
            view,
            tree: RefCell::new(tree),
            action_handler,
            platform_nodes: RefCell::new(HashMap::new()),
            platform_focus: RefCell::new(None),
            mtm,
        })
    }

    pub(crate) fn get_or_create_platform_node(
        self: &Rc<Self>,
        id: NodeId,
    ) -> Option<Retained<PlatformNode>> {
        if let Some(result) = self.platform_nodes.borrow().get(&id) {
            return Some(result.clone());
        }

        let result = PlatformNode::new(self, id)?;
        self.platform_nodes.borrow_mut().insert(id, result.clone());
        Some(result)
    }

    pub(crate) fn remove_platform_node(&self, id: NodeId) -> Option<Retained<PlatformNode>> {
        let mut platform_nodes = self.platform_nodes.borrow_mut();
        platform_nodes.remove(&id)
    }

    pub(crate) fn do_action(&self, request: ActionRequest) {
        self.action_handler.do_action(request);
    }
}
