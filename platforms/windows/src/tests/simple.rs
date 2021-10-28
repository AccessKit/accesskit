// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{convert::TryInto, num::NonZeroU64};

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

#[test]
fn navigation() -> Result<()> {
    scope(WINDOW_TITLE, get_initial_state(), BUTTON_1_ID, |s| {
        let root = unsafe { s.uia.ElementFromHandle(s.window) }?;
        let walker = unsafe { s.uia.ControlViewWalker() }?;

        // The children of the window include the children that we provide,
        // but also the title bar provided by the OS. We know that our own
        // children are in the order we specified, but we don't know
        // their position relative to the title bar. In fact, Windows
        // has changed this in the past.
        //
        // Note that a UIA client would normally use the UIA condition feature
        // to traverse the tree looking for an element that meets
        // some condition. But we want to be explicit about navigating
        // forward (and backward below) through only the immediate children.
        // We'll accept the performance hit of multiple cross-thread calls
        // (insignificant in this case) to achieve that.
        let mut button_1_forward: Option<IUIAutomationElement> = None;
        let mut wrapped_child = unsafe { walker.GetFirstChildElement(&root) };
        while let Ok(child) = wrapped_child {
            let name = unsafe { child.CurrentName() }?;
            let name: String = name.try_into().unwrap();
            if name == "Button 1" {
                button_1_forward = Some(child.clone());
                break;
            }
            wrapped_child = unsafe { walker.GetNextSiblingElement(&child) };
        }
        let button_1_forward = button_1_forward.unwrap();
        let control_type = unsafe { button_1_forward.CurrentControlType() }?;
        assert_eq!(UIA_ButtonControlTypeId, control_type);

        let mut button_2_forward: Option<IUIAutomationElement> = None;
        let wrapped_child = unsafe { walker.GetNextSiblingElement(&button_1_forward) };
        if let Ok(child) = wrapped_child {
            let name = unsafe { child.CurrentName() }?;
            let name: String = name.try_into().unwrap();
            if name == "Button 2" {
                button_2_forward = Some(child.clone());
            }
        }
        let button_2_forward = button_2_forward.unwrap();
        let control_type = unsafe { button_2_forward.CurrentControlType() }?;
        assert_eq!(UIA_ButtonControlTypeId, control_type);

        let mut button_2_backward: Option<IUIAutomationElement> = None;
        let mut wrapped_child = unsafe { walker.GetLastChildElement(&root) };
        while let Ok(child) = wrapped_child {
            let name = unsafe { child.CurrentName() }?;
            let name: String = name.try_into().unwrap();
            if name == "Button 2" {
                button_2_backward = Some(child.clone());
                break;
            }
            wrapped_child = unsafe { walker.GetPreviousSiblingElement(&child) };
        }
        let button_2_backward = button_2_backward.unwrap();
        let control_type = unsafe { button_2_backward.CurrentControlType() }?;
        assert_eq!(UIA_ButtonControlTypeId, control_type);

        let mut button_1_backward: Option<IUIAutomationElement> = None;
        let wrapped_child = unsafe { walker.GetPreviousSiblingElement(&button_2_backward) };
        if let Ok(child) = wrapped_child {
            let name = unsafe { child.CurrentName() }?;
            let name: String = name.try_into().unwrap();
            if name == "Button 1" {
                button_1_backward = Some(child.clone());
            }
        }
        let button_1_backward = button_1_backward.unwrap();
        let control_type = unsafe { button_1_backward.CurrentControlType() }?;
        assert_eq!(UIA_ButtonControlTypeId, control_type);

        let equal: bool =
            unsafe { s.uia.CompareElements(&button_1_forward, &button_1_backward) }?.into();
        assert!(equal);

        let parent = unsafe { walker.GetParentElement(&button_1_forward) }?;
        let equal: bool = unsafe { s.uia.CompareElements(&parent, &root) }?.into();
        assert!(equal);

        let desktop_root = unsafe { s.uia.GetRootElement() }?;
        let parent = unsafe { walker.GetParentElement(&root) }?;
        let equal: bool = unsafe { s.uia.CompareElements(&parent, &desktop_root) }?.into();
        assert!(equal);

        let wrapped_child = unsafe { walker.GetFirstChildElement(&button_1_forward) };
        assert_eq!(Err(Error::OK), wrapped_child);

        let wrapped_child = unsafe { walker.GetLastChildElement(&button_1_forward) };
        assert_eq!(Err(Error::OK), wrapped_child);

        Ok(())
    })
}
