// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

pub use accesskit::{Node as NodeData, Tree as TreeData};

pub(crate) mod tree;
pub use tree::{Change as TreeChange, Reader as TreeReader, Tree};

pub(crate) mod node;
pub use node::{Node, WeakNode};

pub(crate) mod iterators;
pub use iterators::{
    FollowingSiblings, FollowingUnignoredSiblings, PrecedingSiblings, PrecedingUnignoredSiblings,
    UnignoredChildren,
};

#[cfg(test)]
mod tests {
    use accesskit::kurbo::{Affine, Rect, Vec2};
    use accesskit::{ActionHandler, ActionRequest, Node, NodeId, Role, Tree, TreeUpdate};
    use std::num::NonZeroU128;
    use std::sync::Arc;

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

    pub struct NullActionHandler;

    impl ActionHandler for NullActionHandler {
        fn do_action(&self, _request: ActionRequest) {}
    }

    pub fn test_tree() -> Arc<crate::tree::Tree> {
        let root = Arc::new(Node {
            role: Role::RootWebArea,
            children: vec![
                PARAGRAPH_0_ID,
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID,
            ],
            ..Default::default()
        });
        let paragraph_0 = Arc::new(Node {
            role: Role::Paragraph,
            children: vec![STATIC_TEXT_0_0_IGNORED_ID],
            ..Default::default()
        });
        let static_text_0_0_ignored = Arc::new(Node {
            role: Role::StaticText,
            ignored: true,
            name: Some("static_text_0_0_ignored".into()),
            ..Default::default()
        });
        let paragraph_1_ignored = Arc::new(Node {
            role: Role::Paragraph,
            transform: Some(Box::new(Affine::translate(Vec2::new(10.0, 40.0)))),
            bounds: Some(Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 800.0,
                y1: 40.0,
            }),
            children: vec![STATIC_TEXT_1_0_ID],
            ignored: true,
            ..Default::default()
        });
        let static_text_1_0 = Arc::new(Node {
            role: Role::StaticText,
            bounds: Some(Rect {
                x0: 10.0,
                y0: 10.0,
                x1: 90.0,
                y1: 30.0,
            }),
            name: Some("static_text_1_0".into()),
            ..Default::default()
        });
        let paragraph_2 = Arc::new(Node {
            role: Role::Paragraph,
            children: vec![STATIC_TEXT_2_0_ID],
            ..Default::default()
        });
        let static_text_2_0 = Arc::new(Node {
            role: Role::StaticText,
            name: Some("static_text_2_0".into()),
            ..Default::default()
        });
        let paragraph_3_ignored = Arc::new(Node {
            role: Role::Paragraph,
            children: vec![
                EMPTY_CONTAINER_3_0_IGNORED_ID,
                LINK_3_1_IGNORED_ID,
                BUTTON_3_2_ID,
                EMPTY_CONTAINER_3_3_IGNORED_ID,
            ],
            ignored: true,
            ..Default::default()
        });
        let empty_container_3_0_ignored = Arc::new(Node {
            role: Role::GenericContainer,
            ignored: true,
            ..Default::default()
        });
        let link_3_1_ignored = Arc::new(Node {
            role: Role::Link,
            children: vec![STATIC_TEXT_3_1_0_ID],
            ignored: true,
            linked: true,
            ..Default::default()
        });
        let static_text_3_1_0 = Arc::new(Node {
            role: Role::StaticText,
            name: Some("static_text_3_1_0".into()),
            ..Default::default()
        });
        let button_3_2 = Arc::new(Node {
            role: Role::Button,
            name: Some("button_3_2".into()),
            ..Default::default()
        });
        let empty_container_3_3_ignored = Arc::new(Node {
            role: Role::GenericContainer,
            ignored: true,
            ..Default::default()
        });
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
        crate::tree::Tree::new(initial_update, Box::new(NullActionHandler {}))
    }
}
