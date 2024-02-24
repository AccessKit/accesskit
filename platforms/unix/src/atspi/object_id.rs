// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::OwnedObjectAddress;
use accesskit::NodeId;
use accesskit_atspi_common::PlatformNode;
use serde::{Serialize, Serializer};
use zbus::{
    names::OwnedUniqueName,
    zvariant::{ObjectPath, OwnedObjectPath, Signature, Structure, StructureBuilder, Type},
};

const ACCESSIBLE_PATH_PREFIX: &str = "/org/a11y/atspi/accessible/";
const ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";

#[derive(Debug, PartialEq)]
pub(crate) enum ObjectId {
    Root,
    Node { adapter: usize, node: NodeId },
}

impl ObjectId {
    pub(crate) fn to_address(&self, bus_name: OwnedUniqueName) -> OwnedObjectAddress {
        OwnedObjectAddress::new(bus_name, self.path())
    }

    pub(crate) fn path(&self) -> OwnedObjectPath {
        match self {
            Self::Root => ObjectPath::from_str_unchecked(ROOT_PATH),
            Self::Node { adapter, node } => ObjectPath::from_string_unchecked(format!(
                "{}{}/{}",
                ACCESSIBLE_PATH_PREFIX, adapter, node.0
            )),
        }
        .into()
    }
}

impl Serialize for ObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Root => serializer.serialize_str("root"),
            Self::Node { node, .. } => serializer.serialize_str(&node.0.to_string()),
        }
    }
}

impl Type for ObjectId {
    fn signature() -> Signature<'static> {
        <&str>::signature()
    }
}

impl From<ObjectId> for Structure<'_> {
    fn from(id: ObjectId) -> Self {
        StructureBuilder::new()
            .add_field(match id {
                ObjectId::Root => "root".into(),
                ObjectId::Node { node, .. } => node.0.to_string(),
            })
            .build()
    }
}

impl From<&PlatformNode> for ObjectId {
    fn from(node: &PlatformNode) -> Self {
        Self::Node {
            adapter: node.adapter_id(),
            node: node.id(),
        }
    }
}
