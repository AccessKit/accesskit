// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![no_std]

extern crate alloc;

use accesskit::ActivationHandler;

pub(crate) mod tree;
pub use tree::{
    ChangeHandler as TreeChangeHandler, State as TreeState, Tree, Update as TreeUpdate,
};

pub(crate) mod node;
pub use node::Node;

pub(crate) mod filters;
pub use filters::{common_filter, common_filter_with_root_exception, FilterResult};

pub(crate) mod iterators;

pub(crate) mod text;
pub use text::{
    AttributeValue as TextAttributeValue, Position as TextPosition, Range as TextRange,
    WeakRange as WeakTextRange,
};

/// A wrapper over [`ActivationHandler`] that doesn't have any generics in its
/// definition. Applications should generally not implement this directly.
/// It's intended for wrappers over platform adapters that need to box
/// the application-provided activation handler; they can do so by wrapping it
/// in [`BoxedActivationHandler`], which implements this trait.
pub trait NonGenericActivationHandler {
    fn request_initial_tree(&mut self, update: &mut TreeUpdate);
}

impl<H: ActivationHandler + ?Sized> NonGenericActivationHandler for H {
    fn request_initial_tree(&mut self, update: &mut TreeUpdate) {
        ActivationHandler::request_initial_tree(self, update);
    }
}

pub struct BoxedActivationHandler<H: ActivationHandler>(pub H);

impl<H: ActivationHandler> NonGenericActivationHandler for BoxedActivationHandler<H> {
    fn request_initial_tree(&mut self, update: &mut TreeUpdate) {
        self.0.request_initial_tree(update);
    }
}

#[cfg(test)]
mod tests {
    use accesskit::{Affine, NodeId, Rect, Role, Tree, TreeUpdate, Vec2};

    use crate::FilterResult;

    pub const ROOT_ID: NodeId = NodeId(0);
    pub const PARAGRAPH_0_ID: NodeId = NodeId(1);
    pub const LABEL_0_0_IGNORED_ID: NodeId = NodeId(2);
    pub const PARAGRAPH_1_IGNORED_ID: NodeId = NodeId(3);
    pub const BUTTON_1_0_HIDDEN_ID: NodeId = NodeId(4);
    pub const CONTAINER_1_0_0_HIDDEN_ID: NodeId = NodeId(5);
    pub const LABEL_1_1_ID: NodeId = NodeId(6);
    pub const BUTTON_1_2_HIDDEN_ID: NodeId = NodeId(7);
    pub const CONTAINER_1_2_0_HIDDEN_ID: NodeId = NodeId(8);
    pub const PARAGRAPH_2_ID: NodeId = NodeId(9);
    pub const LABEL_2_0_ID: NodeId = NodeId(10);
    pub const PARAGRAPH_3_IGNORED_ID: NodeId = NodeId(11);
    pub const EMPTY_CONTAINER_3_0_IGNORED_ID: NodeId = NodeId(12);
    pub const LINK_3_1_IGNORED_ID: NodeId = NodeId(13);
    pub const LABEL_3_1_0_ID: NodeId = NodeId(14);
    pub const BUTTON_3_2_ID: NodeId = NodeId(15);
    pub const EMPTY_CONTAINER_3_3_IGNORED_ID: NodeId = NodeId(16);

    pub fn build_test_tree(update: &mut crate::TreeUpdate) {
        update.set_node(ROOT_ID, Role::RootWebArea, |node| {
            node.set_children(&[
                PARAGRAPH_0_ID,
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID,
            ]);
        });
        update.set_node(PARAGRAPH_0_ID, Role::Paragraph, |node| {
            node.set_children(&[LABEL_0_0_IGNORED_ID]);
        });
        update.set_node(LABEL_0_0_IGNORED_ID, Role::Label, |node| {
            node.set_value("label_0_0_ignored");
        });
        update.set_node(PARAGRAPH_1_IGNORED_ID, Role::Paragraph, |node| {
            node.set_transform(Affine::translate(Vec2::new(10.0, 40.0)));
            node.set_bounds(Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 800.0,
                y1: 40.0,
            });
            node.set_children(&[BUTTON_1_0_HIDDEN_ID, LABEL_1_1_ID, BUTTON_1_2_HIDDEN_ID]);
        });
        update.set_node(BUTTON_1_0_HIDDEN_ID, Role::Button, |node| {
            node.set_label("button_1_0_hidden");
            node.set_hidden();
            node.set_children(&[CONTAINER_1_0_0_HIDDEN_ID]);
        });
        update.set_node(CONTAINER_1_0_0_HIDDEN_ID, Role::GenericContainer, |node| {
            node.set_hidden();
        });
        update.set_node(LABEL_1_1_ID, Role::Label, |node| {
            node.set_bounds(Rect {
                x0: 10.0,
                y0: 10.0,
                x1: 90.0,
                y1: 30.0,
            });
            node.set_value("label_1_1");
        });
        update.set_node(BUTTON_1_2_HIDDEN_ID, Role::Button, |node| {
            node.set_label("button_1_2_hidden");
            node.set_hidden();
            node.set_children(&[CONTAINER_1_2_0_HIDDEN_ID]);
        });
        update.set_node(CONTAINER_1_2_0_HIDDEN_ID, Role::GenericContainer, |node| {
            node.set_hidden();
        });
        update.set_node(PARAGRAPH_2_ID, Role::Paragraph, |node| {
            node.set_children(&[LABEL_2_0_ID]);
        });
        update.set_node(LABEL_2_0_ID, Role::Label, |node| {
            node.set_label("label_2_0");
        });
        update.set_node(PARAGRAPH_3_IGNORED_ID, Role::Paragraph, |node| {
            node.set_children(&[
                EMPTY_CONTAINER_3_0_IGNORED_ID,
                LINK_3_1_IGNORED_ID,
                BUTTON_3_2_ID,
                EMPTY_CONTAINER_3_3_IGNORED_ID,
            ]);
        });
        update.set_node(
            EMPTY_CONTAINER_3_0_IGNORED_ID,
            Role::GenericContainer,
            |_| (),
        );
        update.set_node(LINK_3_1_IGNORED_ID, Role::Link, |node| {
            node.set_children(&[LABEL_3_1_0_ID]);
        });
        update.set_node(LABEL_3_1_0_ID, Role::Label, |node| {
            node.set_value("label_3_1_0");
        });
        update.set_node(BUTTON_3_2_ID, Role::Button, |node| {
            node.set_label("button_3_2");
        });
        update.set_node(
            EMPTY_CONTAINER_3_3_IGNORED_ID,
            Role::GenericContainer,
            |_| (),
        );
        update.set_tree(Tree::new(ROOT_ID));
        update.set_focus(ROOT_ID);
    }

    pub fn test_tree() -> crate::Tree {
        crate::Tree::new(false, build_test_tree)
    }

    pub fn test_tree_filter(node: &crate::Node) -> FilterResult {
        let id = node.id();
        if node.is_hidden() {
            FilterResult::ExcludeSubtree
        } else if id == LABEL_0_0_IGNORED_ID
            || id == PARAGRAPH_1_IGNORED_ID
            || id == PARAGRAPH_3_IGNORED_ID
            || id == EMPTY_CONTAINER_3_0_IGNORED_ID
            || id == LINK_3_1_IGNORED_ID
            || id == EMPTY_CONTAINER_3_3_IGNORED_ID
        {
            FilterResult::ExcludeNode
        } else {
            FilterResult::Include
        }
    }
}
