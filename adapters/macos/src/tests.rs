// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

//! These tests don't touch AppKit, so they don't need the main thread.

use accesskit::{Node, NodeId, Rect, Role, TreeId, TreeInfo, TreeUpdate};
use accesskit_consumer::Tree;

use crate::{
    adapter::settle_focus_target,
    event::{EventGenerator, QueuedEvent, layout_changed_target},
};

const ROOT: NodeId = NodeId(0);
const BUTTON: NodeId = NodeId(1);
const OTHER_BUTTON: NodeId = NodeId(2);

fn rect(y0: f64, height: f64) -> Rect {
    Rect {
        x0: 0.0,
        y0,
        x1: 100.0,
        y1: y0 + height,
    }
}

fn root_node() -> Node {
    let mut node = Node::new(Role::Window);
    node.push_child(BUTTON);
    node.push_child(OTHER_BUTTON);
    node
}

fn button(label: &str, bounds: Rect) -> Node {
    let mut node = Node::new(Role::Button);
    node.set_label(label);
    node.set_bounds(bounds);
    node
}

/// A window with two buttons, the first of which has focus.
fn tree(is_host_focused: bool) -> Tree {
    Tree::new(
        TreeUpdate {
            nodes: vec![
                (ROOT, root_node()),
                (BUTTON, button("first", rect(0.0, 20.0))),
                (OTHER_BUTTON, button("second", rect(40.0, 20.0))),
            ],
            tree: Some(TreeInfo::new(ROOT)),
            tree_id: TreeId::ROOT,
            focus: BUTTON,
        },
        is_host_focused,
    )
}

fn update(nodes: Vec<(NodeId, Node)>) -> TreeUpdate {
    TreeUpdate {
        nodes,
        tree: None,
        tree_id: TreeId::ROOT,
        focus: BUTTON,
    }
}

fn apply(tree: &mut Tree, update: TreeUpdate) -> Vec<QueuedEvent> {
    let mut event_generator = EventGenerator::new();
    tree.update_and_process_changes(update, &mut event_generator);
    event_generator.into_events()
}

fn layout_changed_count(events: &[QueuedEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, QueuedEvent::LayoutChanged))
        .count()
}

#[test]
fn moving_a_node_queues_a_layout_changed_event() {
    let mut tree = tree(true);
    let events = apply(
        &mut tree,
        update(vec![(BUTTON, button("first", rect(-30.0, 20.0)))]),
    );
    assert_eq!(layout_changed_count(&events), 1);
}

#[test]
fn resizing_a_node_queues_a_layout_changed_event() {
    // Same origin, different size.
    let mut tree = tree(true);
    let events = apply(
        &mut tree,
        update(vec![(BUTTON, button("first", rect(0.0, 50.0)))]),
    );
    assert_eq!(layout_changed_count(&events), 1);
}

#[test]
fn moving_several_nodes_queues_only_one_layout_changed_event() {
    let mut tree = tree(true);
    let events = apply(
        &mut tree,
        update(vec![
            (BUTTON, button("first", rect(-30.0, 20.0))),
            (OTHER_BUTTON, button("second", rect(10.0, 20.0))),
        ]),
    );
    assert_eq!(layout_changed_count(&events), 1);
}

#[test]
fn an_update_that_moves_nothing_queues_no_layout_changed_event() {
    let mut tree = tree(true);
    let events = apply(
        &mut tree,
        update(vec![(BUTTON, button("renamed", rect(0.0, 20.0)))]),
    );
    assert_eq!(layout_changed_count(&events), 0);
    // Not vacuous: the update did change something.
    assert!(!events.is_empty());
}

#[test]
fn re_submitting_an_unchanged_tree_queues_no_layout_changed_event() {
    let mut tree = tree(true);
    let events = apply(
        &mut tree,
        update(vec![
            (BUTTON, button("first", rect(0.0, 20.0))),
            (OTHER_BUTTON, button("second", rect(40.0, 20.0))),
        ]),
    );
    assert_eq!(layout_changed_count(&events), 0);
}

#[test]
fn moving_a_node_that_is_not_exposed_queues_no_layout_changed_event() {
    let mut tree = tree(true);
    let hidden = |bounds| {
        let mut node = button("second", bounds);
        node.set_hidden();
        node
    };
    apply(
        &mut tree,
        update(vec![(OTHER_BUTTON, hidden(rect(40.0, 20.0)))]),
    );
    let events = apply(
        &mut tree,
        update(vec![(OTHER_BUTTON, hidden(rect(70.0, 20.0)))]),
    );
    assert_eq!(layout_changed_count(&events), 0);
}

#[test]
fn adding_and_removing_nodes_queues_no_layout_changed_event() {
    // Additions and removals have their own notifications.
    let mut tree = tree(true);
    let mut root_with_one_child = Node::new(Role::Window);
    root_with_one_child.push_child(BUTTON);
    let events = apply(&mut tree, update(vec![(ROOT, root_with_one_child)]));
    assert_eq!(layout_changed_count(&events), 0);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, QueuedEvent::NodeDestroyed(_)))
    );
}

#[test]
fn layout_changed_is_targeted_at_the_focused_node() {
    let tree = tree(true);
    let state = tree.state();
    assert_eq!(
        layout_changed_target(state),
        state.focus().unwrap().id(),
        "expected the focused node"
    );
    assert_ne!(layout_changed_target(state), state.root().id());
}

#[test]
fn layout_changed_falls_back_to_the_root_when_the_host_is_not_focused() {
    let tree = tree(false);
    let state = tree.state();
    assert!(state.focus().is_none());
    assert_eq!(layout_changed_target(state), state.root().id());
}

#[test]
fn scroll_settle_targets_the_focused_node() {
    let tree = tree(true);
    let state = tree.state();
    assert_eq!(
        settle_focus_target(state),
        Some(state.focus().unwrap().id())
    );
}

#[test]
fn scroll_settle_has_no_target_when_the_host_is_not_focused() {
    let tree = tree(false);
    assert_eq!(settle_focus_target(tree.state()), None);
}

#[test]
fn scroll_settle_has_no_target_when_focus_is_on_the_window_itself() {
    // The placeholder tree focuses the window node.
    let tree = Tree::new(
        TreeUpdate {
            nodes: vec![(ROOT, Node::new(Role::Window))],
            tree: Some(TreeInfo::new(ROOT)),
            tree_id: TreeId::ROOT,
            focus: ROOT,
        },
        true,
    );
    let state = tree.state();
    assert!(state.focus().is_some());
    assert_eq!(settle_focus_target(state), None);
}
