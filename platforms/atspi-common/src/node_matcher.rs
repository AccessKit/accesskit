// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::collections::HashMap;

use accesskit_consumer::{FullNodeId, NodeRef};
use atspi_common::{InterfaceSet, MatchType, ObjectMatchRule, RoleSet, StateSet} ;

use crate::node::NodeWrapper;

pub fn recurse_scan_children(
    node: NodeRef<'_>,
    current_match_list: &mut Vec<FullNodeId>,
    match_rule: &ObjectMatchRule,
    traverse: bool,
    target_node: Option<FullNodeId>,
    is_after: bool,
    is_reverse: bool,
    count: u32,
    // Needed to get node states for comparison
    has_window_focus: bool
) -> u32 {
    let use_count = count != 0;
    let mut count_left = count;
    let mut is_scan_mode =
        target_node.is_none()
        || !is_after;
    maybe_reverse_it_children(node, is_reverse, |child| {
        if use_count && count_left == 0 {
            return;
        }

        let is_target_match = target_node == Some(child.id());
        if is_scan_mode && is_target_match {
            return
        }

        let is_node_match = is_scan_mode && is_match(child, match_rule, has_window_focus);
        if is_node_match {
            current_match_list.push(child.id());
            if use_count {
                count_left -= 1;
            }
        }

        if is_scan_mode && traverse {
            count_left = recurse_scan_children(
                child, current_match_list, match_rule,
                traverse, target_node, is_after, is_reverse,
                count_left, has_window_focus);
        }

        if is_target_match {
            is_scan_mode = !is_scan_mode
        }
    });

    count_left
}

fn maybe_reverse_it_children<F>(node: NodeRef<'_>, is_reverse: bool, f: F)
where F: FnMut(NodeRef<'_>),
{
    if is_reverse {
        node.children().rev().for_each(f);
    } else {
        node.children().for_each(f);
    }
}

fn is_match(node: NodeRef<'_>, match_rule: &ObjectMatchRule, has_window_focus: bool) -> bool {
    let wrapper = NodeWrapper(&node);
    let states_mr = match_states(&wrapper, &match_rule.states, has_window_focus);
    let is_match_states = cmp_match_type(states_mr, match_rule.states_mt);
    let attributes_mr = match_attributes(&wrapper, &match_rule.attr);
    let is_match_attributes = cmp_match_type(attributes_mr, match_rule.attr_mt);
    let roles_mr = match_roles(&wrapper, &match_rule.roles);
    let is_match_roles = cmp_match_type(roles_mr, match_rule.roles_mt);
    let interfaces_mr = match_interfaces(&wrapper, &match_rule.ifaces);
    let is_match_interfaces = cmp_match_type(interfaces_mr, match_rule.ifaces_mt);
    let is_match_all =
        is_match_states
        && is_match_attributes
        && is_match_roles
        && is_match_interfaces;

    is_match_all != match_rule.invert
}

fn match_states(node_wrapper: &NodeWrapper<'_>, states: &StateSet, has_window_focus: bool) -> MatchResult {
    let node_states = node_wrapper.state(has_window_focus);

    let states_1_bits = node_states.bits();
    let states_2_bits = states.bits();

    let all_matched = states_1_bits & states_2_bits == states_2_bits;
    let none_matched = states_1_bits & states_2_bits == 0;

    to_match_result(all_matched, none_matched)
}

fn match_attributes(node_wrapper: &NodeWrapper<'_>, attributes: &HashMap<String, String>) -> MatchResult {
    let node_attributes = node_wrapper.attributes();
    
    let all_matched = attributes.iter().all(
        |(k,v)| node_attributes.get(k.as_str()) == Some(v) );
    let none_matched = !all_matched && !attributes.iter().any(
        |(k,v)| node_attributes.get(k.as_str()) == Some(v) );

    to_match_result(all_matched, none_matched)
}

fn match_roles(node_wrapper: &NodeWrapper<'_>, roles: &RoleSet) -> MatchResult {
    let role = node_wrapper.role();

    let all_matched =
        if roles.is_empty() { true }
        else if roles.len() == 1 { roles.contains(role) }
        else { false };
    let none_matched = !roles.contains(role);

    to_match_result(all_matched, none_matched)
}

fn match_interfaces(node_wrapper: &NodeWrapper<'_>, interfaces: &InterfaceSet) -> MatchResult {
    let node_interfaces = node_wrapper.interfaces();

    let interfaces_1_bits = node_interfaces.bits();
    let interfaces_2_bits = interfaces.bits();

    let all_matched = interfaces_1_bits & interfaces_2_bits == interfaces_2_bits;
    let none_matched = interfaces_1_bits & interfaces_2_bits == 0;

    to_match_result(all_matched, none_matched)
}

fn to_match_result(all_matched: bool, none_matched: bool) -> MatchResult {
    if all_matched {
        MatchResult::All
    } else if none_matched {
        MatchResult::None
    } else {
        MatchResult::Some
    }
}

fn cmp_match_type(match_result: MatchResult, match_type: MatchType) -> bool {
    match match_type {
        MatchType::Invalid => false,
        MatchType::All => match_result == MatchResult::All,
        MatchType::Any => match_result != MatchResult::None,
        MatchType::None => match_result == MatchResult::None,
        MatchType::Empty => false,
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum MatchResult {
    All, Some, None
}