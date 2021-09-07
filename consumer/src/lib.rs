// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub use accesskit_schema::{Node as NodeData, Tree as TreeData};

pub(crate) mod tree;
pub use tree::{Reader as TreeReader, Tree};

pub(crate) mod node;
pub use node::{Node, WeakNode};

pub(crate) mod iterators;
pub use iterators::{
    FollowingSiblings, FollowingUnignoredSiblings, PrecedingSiblings, PrecedingUnignoredSiblings,
    UnignoredChildren,
};

#[cfg(test)]
mod tests {
    use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
    use std::num::NonZeroU64;
    use std::sync::Arc;

    pub const ROOT_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
    pub const PARAGRAPH_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
    pub const STATIC_TEXT_0_0_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
    pub const PARAGRAPH_1_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(4) });
    pub const STATIC_TEXT_1_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(5) });
    pub const PARAGRAPH_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(6) });
    pub const STATIC_TEXT_2_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(7) });
    pub const PARAGRAPH_3_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(8) });
    pub const LINK_3_0_IGNORED_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(9) });
    pub const STATIC_TEXT_3_0_0_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(10) });
    pub const BUTTON_3_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(11) });

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
            name: Some("static_text_0_0_ignored".to_string()),
            ..Node::new(STATIC_TEXT_0_0_IGNORED_ID, Role::StaticText)
        };
        let paragraph_1_ignored = Node {
            children: vec![STATIC_TEXT_1_0_ID],
            ignored: true,
            ..Node::new(PARAGRAPH_1_IGNORED_ID, Role::Paragraph)
        };
        let static_text_1_0 = Node {
            name: Some("static_text_1_0".to_string()),
            ..Node::new(STATIC_TEXT_1_0_ID, Role::StaticText)
        };
        let paragraph_2 = Node {
            children: vec![STATIC_TEXT_2_0_ID],
            ..Node::new(PARAGRAPH_2_ID, Role::Paragraph)
        };
        let static_text_2_0 = Node {
            name: Some("static_text_2_0".to_string()),
            ..Node::new(STATIC_TEXT_2_0_ID, Role::StaticText)
        };
        let paragraph_3_ignored = Node {
            children: vec![LINK_3_0_IGNORED_ID, BUTTON_3_1_ID],
            ignored: true,
            ..Node::new(PARAGRAPH_3_IGNORED_ID, Role::Paragraph)
        };
        let link_3_0_ignored = Node {
            children: vec![STATIC_TEXT_3_0_0_ID],
            ignored: true,
            linked: true,
            ..Node::new(LINK_3_0_IGNORED_ID, Role::Link)
        };
        let static_text_3_0_0 = Node {
            name: Some("static_text_3_0_0".to_string()),
            ..Node::new(STATIC_TEXT_3_0_0_ID, Role::StaticText)
        };
        let button_3_1 = Node {
            name: Some("button_3_1".to_string()),
            ..Node::new(BUTTON_3_1_ID, Role::Button)
        };
        let initial_update = TreeUpdate {
            clear: None,
            nodes: vec![
                root,
                paragraph_0,
                static_text_0_0_ignored,
                paragraph_1_ignored,
                static_text_1_0,
                paragraph_2,
                static_text_2_0,
                paragraph_3_ignored,
                link_3_0_ignored,
                static_text_3_0_0,
                button_3_1,
            ],
            tree: Some(Tree::new(
                TreeId("test_tree".to_string()),
                StringEncoding::Utf8,
            )),
            root: Some(ROOT_ID),
        };
        crate::tree::Tree::new(initial_update)
    }
}
