// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::num::NonZeroU64;

use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use windows::{runtime::*, Win32::UI::Accessibility::*};

use super::*;

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });

fn get_button_1(name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(BUTTON_1_ID, Role::Button)
    }
}

fn get_button_2(name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(BUTTON_2_ID, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = get_button_1("Button 1");
    let button_2 = get_button_2("Button 2");
    TreeUpdate {
        clear: None,
        nodes: vec![root, button_1, button_2],
        tree: Some(Tree::new(
            TreeId("test".into()),
            WINDOW_ID,
            StringEncoding::Utf8,
        )),
        focus: None,
    }
}

#[test]
fn has_native_uia() -> Result<()> {
    scope(WINDOW_TITLE, get_initial_state(), BUTTON_1_ID, |s| {
        let has_native_uia: bool = unsafe { UiaHasServerSideProvider(s.window) }.into();
        assert!(has_native_uia);
        Ok(())
    })
}
