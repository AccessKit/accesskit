// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

pub(crate) mod tree;
pub use tree::{ChangeHandler as TreeChangeHandler, State as TreeState, Tree};

pub(crate) mod node;
pub use node::{DetachedNode, Node, NodeState};

pub(crate) mod iterators;
pub use iterators::FilterResult;

pub(crate) mod text;
pub use text::{
    AttributeValue as TextAttributeValue, Position as TextPosition, Range as TextRange,
    WeakRange as WeakTextRange,
};

#[cfg(test)]
mod tests {
    use accesskit::{
        Affine, NodeBuilder, NodeClassSet, NodeId, Rect, Role, Tree, TreeUpdate, Vec2,
    };
    use std::num::NonZeroU128;

    use crate::FilterResult;

    pub const ROOT_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
    pub const PARAGRAPH_0_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
    pub const STATIC_TEXT_0_0_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
    pub const PARAGRAPH_1_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
    pub const STATIC_TEXT_1_0_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(5) });
    pub const PARAGRAPH_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(6) });
    pub const STATIC_TEXT_2_0_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(7) });
    pub const PARAGRAPH_3_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(8) });
    pub const EMPTY_CONTAINER_3_0_IGNORED_ID: NodeId =
        NodeId(unsafe { NonZeroU128::new_unchecked(9) });
    pub const LINK_3_1_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(10) });
    pub const STATIC_TEXT_3_1_0_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(11) });
    pub const BUTTON_3_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(12) });
    pub const EMPTY_CONTAINER_3_3_IGNORED_ID: NodeId =
        NodeId(unsafe { NonZeroU128::new_unchecked(13) });

    pub fn test_tree() -> crate::tree::Tree {
        let mut classes = NodeClassSet::new();
        let root = {
            let mut builder = NodeBuilder::new(Role::RootWebArea);
            builder.set_children(vec![
                PARAGRAPH_0_ID,
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID,
            ]);
            builder.build(&mut classes)
        };
        let paragraph_0 = {
            let mut builder = NodeBuilder::new(Role::Paragraph);
            builder.set_children(vec![STATIC_TEXT_0_0_IGNORED_ID]);
            builder.build(&mut classes)
        };
        let static_text_0_0_ignored = {
            let mut builder = NodeBuilder::new(Role::StaticText);
            builder.set_name("static_text_0_0_ignored");
            builder.build(&mut classes)
        };
        let paragraph_1_ignored = {
            let mut builder = NodeBuilder::new(Role::Paragraph);
            builder.set_transform(Affine::translate(Vec2::new(10.0, 40.0)));
            builder.set_bounds(Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 800.0,
                y1: 40.0,
            });
            builder.set_children(vec![STATIC_TEXT_1_0_ID]);
            builder.build(&mut classes)
        };
        let static_text_1_0 = {
            let mut builder = NodeBuilder::new(Role::StaticText);
            builder.set_bounds(Rect {
                x0: 10.0,
                y0: 10.0,
                x1: 90.0,
                y1: 30.0,
            });
            builder.set_name("static_text_1_0");
            builder.build(&mut classes)
        };
        let paragraph_2 = {
            let mut builder = NodeBuilder::new(Role::Paragraph);
            builder.set_children(vec![STATIC_TEXT_2_0_ID]);
            builder.build(&mut classes)
        };
        let static_text_2_0 = {
            let mut builder = NodeBuilder::new(Role::StaticText);
            builder.set_name("static_text_2_0");
            builder.build(&mut classes)
        };
        let paragraph_3_ignored = {
            let mut builder = NodeBuilder::new(Role::Paragraph);
            builder.set_children(vec![
                EMPTY_CONTAINER_3_0_IGNORED_ID,
                LINK_3_1_IGNORED_ID,
                BUTTON_3_2_ID,
                EMPTY_CONTAINER_3_3_IGNORED_ID,
            ]);
            builder.build(&mut classes)
        };
        let empty_container_3_0_ignored =
            NodeBuilder::new(Role::GenericContainer).build(&mut classes);
        let link_3_1_ignored = {
            let mut builder = NodeBuilder::new(Role::Link);
            builder.set_children(vec![STATIC_TEXT_3_1_0_ID]);
            builder.set_linked();
            builder.build(&mut classes)
        };
        let static_text_3_1_0 = {
            let mut builder = NodeBuilder::new(Role::StaticText);
            builder.set_name("static_text_3_1_0");
            builder.build(&mut classes)
        };
        let button_3_2 = {
            let mut builder = NodeBuilder::new(Role::Button);
            builder.set_name("button_3_2");
            builder.build(&mut classes)
        };
        let empty_container_3_3_ignored =
            NodeBuilder::new(Role::GenericContainer).build(&mut classes);
        let initial_update = TreeUpdate {
            nodes: vec![
                (ROOT_ID, root),
                (PARAGRAPH_0_ID, paragraph_0),
                (STATIC_TEXT_0_0_IGNORED_ID, static_text_0_0_ignored),
                (PARAGRAPH_1_IGNORED_ID, paragraph_1_ignored),
                (STATIC_TEXT_1_0_ID, static_text_1_0),
                (PARAGRAPH_2_ID, paragraph_2),
                (STATIC_TEXT_2_0_ID, static_text_2_0),
                (PARAGRAPH_3_IGNORED_ID, paragraph_3_ignored),
                (EMPTY_CONTAINER_3_0_IGNORED_ID, empty_container_3_0_ignored),
                (LINK_3_1_IGNORED_ID, link_3_1_ignored),
                (STATIC_TEXT_3_1_0_ID, static_text_3_1_0),
                (BUTTON_3_2_ID, button_3_2),
                (EMPTY_CONTAINER_3_3_IGNORED_ID, empty_container_3_3_ignored),
            ],
            tree: Some(Tree::new(ROOT_ID)),
            focus: None,
        };
        crate::tree::Tree::new(initial_update)
    }

    pub fn test_tree_filter(node: &crate::Node) -> FilterResult {
        let id = node.id();
        if id == STATIC_TEXT_0_0_IGNORED_ID
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
