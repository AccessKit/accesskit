// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TextDirection,
    TextPosition, TextSelection, Tree, TreeId, TreeUpdate,
};
use windows::{
    core::*,
    Win32::{System::Variant::VARIANT, UI::Accessibility::*},
};

use super::*;

const WINDOW_TITLE: &str = "Simple test";

const WINDOW_ID: NodeId = NodeId(0);
const BUTTON_1_ID: NodeId = NodeId(1);
const BUTTON_2_ID: NodeId = NodeId(2);

fn make_button(label: &str) -> Node {
    let mut node = Node::new(Role::Button);
    node.set_label(label);
    node.add_action(Action::Focus);
    node
}

fn get_initial_state() -> TreeUpdate {
    let mut root = Node::new(Role::Window);
    root.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
    let button_1 = make_button("Button 1");
    let button_2 = make_button("Button 2");
    TreeUpdate {
        nodes: vec![
            (WINDOW_ID, root),
            (BUTTON_1_ID, button_1),
            (BUTTON_2_ID, button_2),
        ],
        tree: Some(Tree::new(WINDOW_ID)),
        tree_id: TreeId::ROOT,
        focus: BUTTON_1_ID,
    }
}

pub struct NullActionHandler;

impl ActionHandler for NullActionHandler {
    fn do_action(&mut self, _request: ActionRequest) {}
}

struct SimpleActivationHandler;

impl ActivationHandler for SimpleActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(get_initial_state())
    }
}

fn scope<F>(f: F) -> Result<()>
where
    F: FnOnce(&Scope) -> Result<()>,
{
    super::scope(
        WINDOW_TITLE,
        SimpleActivationHandler {},
        NullActionHandler {},
        f,
    )
}

#[test]
fn has_native_uia() -> Result<()> {
    scope(|s| {
        let has_native_uia: bool = unsafe { UiaHasServerSideProvider(s.window.0) }.into();
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
    scope(|s| {
        let root = unsafe { s.uia.ElementFromHandle(s.window.0) }?;
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
        assert_eq!(Err(Error::empty()), wrapped_child);

        let wrapped_child = unsafe { walker.GetLastChildElement(&button_1_forward) };
        assert_eq!(Err(Error::empty()), wrapped_child);

        Ok(())
    })
}

#[test]
fn focus() -> Result<()> {
    scope(|s| {
        let (focus_event_handler, received_focus_event) = FocusEventHandler::new();
        unsafe {
            s.uia
                .AddFocusChangedEventHandler(None, &focus_event_handler)
        }?;

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

const TEXT_INPUT_ID: NodeId = NodeId(10);
const TEXT_RUN_0_ID: NodeId = NodeId(20);
const TEXT_RUN_1_ID: NodeId = NodeId(21);

fn make_text_run(value: &str, character_lengths: &[u8], word_starts: &[u8]) -> Node {
    let mut node = Node::new(Role::TextRun);
    node.set_value(value);
    node.set_character_lengths(character_lengths.to_vec().into_boxed_slice());
    node.set_character_widths(vec![7.0; character_lengths.len()].into_boxed_slice());
    node.set_character_positions(
        (0..character_lengths.len())
            .map(|i| i as f32 * 7.0)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    node.set_word_starts(word_starts.to_vec().into_boxed_slice());
    node.set_text_direction(TextDirection::LeftToRight);
    node
}

fn two_line_text_tree() -> TreeUpdate {
    let mut root = Node::new(Role::Window);
    root.set_children(vec![TEXT_INPUT_ID]);

    let mut text_input = Node::new(Role::TextInput);
    text_input.add_action(Action::Focus);
    text_input.set_children(vec![TEXT_RUN_0_ID, TEXT_RUN_1_ID]);
    text_input.set_text_selection(TextSelection {
        anchor: TextPosition {
            node: TEXT_RUN_1_ID,
            character_index: 6,
        },
        focus: TextPosition {
            node: TEXT_RUN_1_ID,
            character_index: 6,
        },
    });

    let run_0 = make_text_run("Hello ", &[1; 6], &[0]);
    let run_1 = make_text_run("world!", &[1; 6], &[0]);

    TreeUpdate {
        nodes: vec![
            (WINDOW_ID, root),
            (TEXT_INPUT_ID, text_input),
            (TEXT_RUN_0_ID, run_0),
            (TEXT_RUN_1_ID, run_1),
        ],
        tree: Some(Tree::new(WINDOW_ID)),
        tree_id: TreeId::ROOT,
        focus: TEXT_INPUT_ID,
    }
}

fn one_line_text_update() -> TreeUpdate {
    let mut text_input = Node::new(Role::TextInput);
    text_input.add_action(Action::Focus);
    text_input.set_children(vec![TEXT_RUN_0_ID]);
    text_input.set_text_selection(TextSelection {
        anchor: TextPosition {
            node: TEXT_RUN_0_ID,
            character_index: 11,
        },
        focus: TextPosition {
            node: TEXT_RUN_0_ID,
            character_index: 11,
        },
    });

    let run_0 = make_text_run("Hello world", &[1; 11], &[0, 6]);

    TreeUpdate {
        nodes: vec![(TEXT_INPUT_ID, text_input), (TEXT_RUN_0_ID, run_0)],
        tree: None,
        tree_id: TreeId::ROOT,
        focus: TEXT_INPUT_ID,
    }
}

struct TextActivationHandler;

impl ActivationHandler for TextActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(two_line_text_tree())
    }
}

#[test]
fn compare_endpoints_after_text_run_removed() -> Result<()> {
    super::scope(
        "Text reflow test",
        TextActivationHandler {},
        NullActionHandler {},
        |s| {
            s.show_and_focus_window();

            let root = unsafe { s.uia.ElementFromHandle(s.window.0) }?;
            let condition = unsafe {
                s.uia.CreatePropertyCondition(
                    UIA_ControlTypePropertyId,
                    &VARIANT::from(UIA_EditControlTypeId.0),
                )
            }?;
            let text_element = unsafe { root.FindFirst(TreeScope_Descendants, &condition) }?;

            let pattern: IUIAutomationTextPattern =
                unsafe { text_element.GetCurrentPatternAs(UIA_TextPatternId) }?;
            let selection = unsafe { pattern.GetSelection() }?;
            let old_range: IUIAutomationTextRange = unsafe { selection.GetElement(0) }?;

            s.update_tree(one_line_text_update());

            let new_selection = unsafe { pattern.GetSelection() }?;
            let new_range: IUIAutomationTextRange = unsafe { new_selection.GetElement(0) }?;

            let result = unsafe {
                new_range.CompareEndpoints(
                    TextPatternRangeEndpoint_Start,
                    &old_range,
                    TextPatternRangeEndpoint_Start,
                )
            };
            assert!(
                result.is_ok(),
                "CompareEndpoints failed after text run removal: {:?}",
                result.err()
            );

            Ok(())
        },
    )
}
