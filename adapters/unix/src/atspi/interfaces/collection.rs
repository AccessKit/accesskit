// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::collections::HashMap;

use accesskit_atspi_common::PlatformNode;
use atspi::{InterfaceSet, MatchType, ObjectMatchRule, RoleSet, SortOrder, StateSet, TreeTraversalType};
use serde::{Deserialize, Serialize};
use zbus::{fdo, interface, names::{BusName, OwnedUniqueName}, zvariant::ObjectPath};

use crate::atspi::{ObjectId, OwnedObjectAddress, object_util::object_path_components};

pub(crate) struct CollectionInterface {
    bus_name: OwnedUniqueName,
    node: PlatformNode,
}

impl CollectionInterface {
    pub fn new(bus_name: OwnedUniqueName, node: PlatformNode) -> Self {
        Self { bus_name, node }
    }

    fn map_error(&self) -> impl '_ + FnOnce(accesskit_atspi_common::Error) -> fdo::Error {
        |error| crate::util::map_error_from_node(&self.node, error)
    }
}

#[interface(name = "org.a11y.atspi.Collection")]
impl CollectionInterface {

    fn get_active_descendant(&self) -> fdo::Result<(OwnedObjectAddress,)> {
        let child = self
            .node
            .active_descendant()
            .map_err(self.map_error())?
            .map(|child| ObjectId::Node {
                adapter: self.node.adapter_id(),
                node: child,
            });
        Ok(super::optional_object_address(&self.bus_name, child))
    }

    fn get_matches(&self, fixed_rule: FixedObjectMatchRule, sortby: SortOrder, count: i32, traverse: bool) -> fdo::Result<Vec<OwnedObjectAddress>> {
        let rule: ObjectMatchRule = fixed_rule.try_into()
            .map_err(|err| fdo::Error::Failed(err))?;
        let is_reverse = sortby == SortOrder::ReverseCanonical;
        let matched_children = self.node.get_matches(&rule, is_reverse, count as u32, traverse)
            .map_err(self.map_error())?;
        let matched_children_add = matched_children
            .into_iter()
            .map(|child| ObjectId::Node {
                adapter: self.node.adapter_id(),
                node: child,
            })
            .map(|child| super::optional_object_address(&self.bus_name, Some(child)).0)
            .collect();
        Ok(matched_children_add)
    }

    fn get_matches_to(&self, current_object_t: (BusName, ObjectPath), fixed_rule: FixedObjectMatchRule, sortby: SortOrder, tree: TreeTraversalType, count: i32, traverse: bool) -> fdo::Result<Vec<OwnedObjectAddress>> {
        let rule: ObjectMatchRule = fixed_rule.try_into()
            .map_err(|err| fdo::Error::Failed(err))?;

        let current_object_components = object_path_components(&current_object_t.1)
            .ok_or_else(|| fdo::Error::UnknownObject("Invalid Object Path: ".to_string() + current_object_t.1.as_str())
        )?;
        let current_tree_index = current_object_components.1;
        let current_node_id = current_object_components.2;

        let is_reverse = sortby == SortOrder::ReverseCanonical;
        let matched_children = self.node.get_matches_to_or_from(
            false, current_tree_index, current_node_id, &rule, is_reverse, tree, count as u32, traverse
        ).map_err(self.map_error())?;
        let matched_children_add = matched_children
            .into_iter()
            .map(|child| ObjectId::Node {
                adapter: self.node.adapter_id(),
                node: child,
            })
            .map(|child| super::optional_object_address(&self.bus_name, Some(child)).0)
            .collect();
        Ok(matched_children_add)
    }

    fn get_matches_from(&self, current_object_t: (BusName, ObjectPath), fixed_rule: FixedObjectMatchRule, sortby: SortOrder, tree: TreeTraversalType, count: i32, traverse: bool) -> fdo::Result<Vec<OwnedObjectAddress>> {
        let rule: ObjectMatchRule = fixed_rule.try_into()
            .map_err(|err| fdo::Error::Failed(err))?;

        let current_object_components = object_path_components(&current_object_t.1)
            .ok_or_else(|| fdo::Error::UnknownObject("Invalid Object Path: ".to_string() + current_object_t.1.as_str())
        )?;
        let current_tree_index = current_object_components.1;
        let current_node_id = current_object_components.2;

        let is_reverse = sortby == SortOrder::ReverseCanonical;
        let matched_children = self.node.get_matches_to_or_from(
            true, current_tree_index, current_node_id, &rule, is_reverse, tree, count as u32, traverse
        ).map_err(self.map_error())?;
        let matched_children_add = matched_children
            .into_iter()
            .map(|child| ObjectId::Node {
                adapter: self.node.adapter_id(),
                node: child,
            })
            .map(|child| super::optional_object_address(&self.bus_name, Some(child)).0)
            .collect();
        Ok(matched_children_add)
    }
    
}

// Because atspi's default rule is broken, even in 0.31
// MatchType's deserializer expects a string
// RoleSet breaks if there are less than 5 elements
// And then StateSet just broke when I copied it here

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, zbus::zvariant::Type)]
pub struct FixedObjectMatchRule {
	pub states: Vec<i32>,
	pub states_mt: i32,
	pub attr: HashMap<String, String>,
	pub attr_mt: i32,
	pub roles: Vec<i32>,
	pub roles_mt: i32,
	pub ifaces: InterfaceSet,
	pub ifaces_mt: i32,
	pub invert: bool
}

impl TryInto<ObjectMatchRule> for FixedObjectMatchRule {
    type Error = String;

    fn try_into(self) -> Result<ObjectMatchRule, Self::Error> {
        if self.states.len() != 2 {
            return Err("Expected states vector with length 2".to_string())
        }

        let states_bits = ((self.states[0] as u32 as u64) << 32) | (self.states[1] as u32 as u64);
        let state_set = StateSet::from_bits(states_bits)
            .map_err(|e| e.to_string())?;
        let role_set = to_role_set(self.roles)?;
        
        let result = ObjectMatchRule::builder()
            .states(state_set, to_match_type(self.states_mt)?)
            .attributes(self.attr, to_match_type(self.attr_mt)?)
            .roles(&role_set.into_iter().collect::<Vec<_>>(), to_match_type(self.roles_mt)?)
            .interfaces(self.ifaces, to_match_type(self.ifaces_mt)?)
            .build();

        Ok(result)
    }
}

fn to_match_type(ordinal: i32) -> Result<MatchType, String> {
    match ordinal {
        0 => Ok(MatchType::Invalid),
        1 => Ok(MatchType::All),
        2 => Ok(MatchType::Any),
        3 => Ok(MatchType::None),
        4 => Ok(MatchType::Empty),
        _ => Err("Could not deserialize match type".to_string()),
    }
}

fn to_role_set(v: Vec<i32>) -> Result<RoleSet, String> {
    let backing_arr: [i32;5] = std::array::from_fn(
        |i| if i < v.len() { v[i] } else { 0 });

    Ok(unsafe { std::mem::transmute(backing_arr) })
}