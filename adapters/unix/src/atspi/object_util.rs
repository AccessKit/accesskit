// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use zbus::zvariant::ObjectPath;

const ACCESSIBLE_PATH_PREFIX: &str = "/org/a11y/atspi/accessible/";

pub fn object_path_components(path: &ObjectPath) -> Option<(usize, u32, NodeId)> {
    let stripped_path = path.as_str().strip_prefix(ACCESSIBLE_PATH_PREFIX)?;

    if let Some((adapter_str, node_str)) = stripped_path.split_once('/') {
        let adapter_usize = adapter_str.parse::<usize>().ok()?;
        let node_tree_u128 = node_str.parse::<u128>().ok()?;
        let tree_index = node_tree_u128 as u32;
        let node_id = NodeId((node_tree_u128 >> 64) as u64);
        Some((adapter_usize, tree_index, node_id))
    } else {
        None
    }
}