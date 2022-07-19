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
    use accesskit::{
        ActionHandler, ActionRequest, Node, NodeId, Role, StringEncoding, Tree, TreeUpdate,
    };
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
        let root = Node {
            children: vec![
                PARAGRAPH_0_ID,
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID,
            ],
            ..Node::new(ROOT_ID, Role::RootWebArea)
        };
        let paragraph_0 = Node {
            children: vec![STATIC_TEXT_0_0_IGNORED_ID],
            ..Node::new(PARAGRAPH_0_ID, Role::Paragraph)
        };
        let static_text_0_0_ignored = Node {
            ignored: true,
            name: Some("static_text_0_0_ignored".into()),
            ..Node::new(STATIC_TEXT_0_0_IGNORED_ID, Role::StaticText)
        };
        let paragraph_1_ignored = Node {
            transform: Some(Box::new(Affine::translate(Vec2::new(10.0, 40.0)))),
            bounds: Some(Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 800.0,
                y1: 40.0,
            }),
            children: vec![STATIC_TEXT_1_0_ID],
            ignored: true,
            ..Node::new(PARAGRAPH_1_IGNORED_ID, Role::Paragraph)
        };
        let static_text_1_0 = Node {
            bounds: Some(Rect {
                x0: 10.0,
                y0: 10.0,
                x1: 90.0,
                y1: 30.0,
            }),
            name: Some("static_text_1_0".into()),
            ..Node::new(STATIC_TEXT_1_0_ID, Role::StaticText)
        };
        let paragraph_2 = Node {
            children: vec![STATIC_TEXT_2_0_ID],
            ..Node::new(PARAGRAPH_2_ID, Role::Paragraph)
        };
        let static_text_2_0 = Node {
            name: Some("static_text_2_0".into()),
            ..Node::new(STATIC_TEXT_2_0_ID, Role::StaticText)
        };
        let paragraph_3_ignored = Node {
            children: vec![
                EMPTY_CONTAINER_3_0_IGNORED_ID,
                LINK_3_1_IGNORED_ID,
                BUTTON_3_2_ID,
                EMPTY_CONTAINER_3_3_IGNORED_ID,
            ],
            ignored: true,
            ..Node::new(PARAGRAPH_3_IGNORED_ID, Role::Paragraph)
        };
        let empty_container_3_0_ignored = Node {
            ignored: true,
            ..Node::new(EMPTY_CONTAINER_3_0_IGNORED_ID, Role::GenericContainer)
        };
        let link_3_1_ignored = Node {
            children: vec![STATIC_TEXT_3_1_0_ID],
            ignored: true,
            linked: true,
            ..Node::new(LINK_3_1_IGNORED_ID, Role::Link)
        };
        let static_text_3_1_0 = Node {
            name: Some("static_text_3_1_0".into()),
            ..Node::new(STATIC_TEXT_3_1_0_ID, Role::StaticText)
        };
        let button_3_2 = Node {
            name: Some("button_3_2".into()),
            ..Node::new(BUTTON_3_2_ID, Role::Button)
        };
        let empty_container_3_3_ignored = Node {
            ignored: true,
            ..Node::new(EMPTY_CONTAINER_3_3_IGNORED_ID, Role::GenericContainer)
        };
        let initial_update = TreeUpdate {
            nodes: vec![
                root,
                paragraph_0,
                static_text_0_0_ignored,
                paragraph_1_ignored,
                static_text_1_0,
                paragraph_2,
                static_text_2_0,
                paragraph_3_ignored,
                empty_container_3_0_ignored,
                link_3_1_ignored,
                static_text_3_1_0,
                button_3_2,
                empty_container_3_3_ignored,
            ],
            tree: Some(Tree::new(ROOT_ID, StringEncoding::Utf8)),
            focus: None,
        };
        crate::tree::Tree::new(initial_update, Box::new(NullActionHandler {}))
    }
}
