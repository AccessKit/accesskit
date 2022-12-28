// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, OwnedObjectAddress};
use accesskit::NodeId;
use zbus::{names::OwnedUniqueName, zvariant::Value};

#[derive(Debug, PartialEq)]
pub(crate) enum ObjectRef {
    Managed(ObjectId<'static>),
    Unmanaged(OwnedObjectAddress),
}

impl<'a> ObjectRef {
    pub(crate) fn into_value(self, name: OwnedUniqueName) -> Value<'a> {
        match self {
            Self::Managed(object) => OwnedObjectAddress::accessible(name, object),
            Self::Unmanaged(object) => object,
        }
        .into()
    }
}

impl From<NodeId> for ObjectRef {
    fn from(value: NodeId) -> ObjectRef {
        ObjectRef::Managed(value.into())
    }
}

impl From<ObjectId<'static>> for ObjectRef {
    fn from(value: ObjectId<'static>) -> ObjectRef {
        ObjectRef::Managed(value)
    }
}

impl From<OwnedObjectAddress> for ObjectRef {
    fn from(value: OwnedObjectAddress) -> ObjectRef {
        ObjectRef::Unmanaged(value)
    }
}
