// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{convert::TryInto, num::NonZeroU64};

use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use windows::{core::*, Win32::UI::Accessibility::*};

use super::*;

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });

fn make_button(id: NodeId, name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(id, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = make_button(BUTTON_1_ID, "Button 1");
    let button_2 = make_button(BUTTON_2_ID, "Button 2");
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

fn is_button_named(element: &IUIAutomationElement, expected_name: &str) -> bool {
    let control_type = unsafe { element.CurrentControlType() }.unwrap();
    let name = unsafe { element.CurrentName() }.unwrap();
    let name: String = name.try_into().unwrap();
    control_type == UIA_ButtonControlTypeId && name == expected_name
}

fn is_button_1(element: &IUIAutomationElement) -> bool {
    is_button_named(element, "Button 1")
}

fn is_button_2(element: &IUIAutomationElement) -> bool {
    is_button_named(element, "Button 2")
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
            if is_button_1(&child) {
                button_1_forward = Some(child);
                break;
            }
            wrapped_child = unsafe { walker.GetNextSiblingElement(&child) };
        }
        let button_1_forward = button_1_forward.unwrap();

        let mut button_2_forward: Option<IUIAutomationElement> = None;
        let wrapped_child = unsafe { walker.GetNextSiblingElement(&button_1_forward) };
        if let Ok(child) = wrapped_child {
            if is_button_2(&child) {
                button_2_forward = Some(child);
            }
        }
        let _button_2_forward = button_2_forward.unwrap();

        let mut button_2_backward: Option<IUIAutomationElement> = None;
        let mut wrapped_child = unsafe { walker.GetLastChildElement(&root) };
        while let Ok(child) = wrapped_child {
            if is_button_2(&child) {
                button_2_backward = Some(child);
                break;
            }
            wrapped_child = unsafe { walker.GetPreviousSiblingElement(&child) };
        }
        let button_2_backward = button_2_backward.unwrap();

        let mut button_1_backward: Option<IUIAutomationElement> = None;
        let wrapped_child = unsafe { walker.GetPreviousSiblingElement(&button_2_backward) };
        if let Ok(child) = wrapped_child {
            if is_button_1(&child) {
                button_1_backward = Some(child);
            }
        }
        let button_1_backward = button_1_backward.unwrap();

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

#[test]
fn focus() -> Result<()> {
    scope(WINDOW_TITLE, get_initial_state(), BUTTON_1_ID, |s| {
        let (focus_event_handler, received_focus_event) = FocusEventHandler::new();
        unsafe { s.uia.AddFocusChangedEventHandler(None, focus_event_handler) }?;

        s.show_and_focus_window();
        let focus_from_event = received_focus_event.wait(is_button_1);
        let has_focus: bool = unsafe { focus_from_event.CurrentHasKeyboardFocus() }?.into();
        assert!(has_focus);
        let is_focusable: bool = unsafe { focus_from_event.CurrentIsKeyboardFocusable() }?.into();
        assert!(is_focusable);

        let focus_on_demand = unsafe { s.uia.GetFocusedElement() }?;
        let equal: bool =
            unsafe { s.uia.CompareElements(&focus_from_event, &focus_on_demand) }?.into();
        assert!(equal);

        Ok(())
    })
}
