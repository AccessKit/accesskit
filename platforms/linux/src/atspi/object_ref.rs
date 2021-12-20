// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{ObjectId, OwnedObjectAddress};
use accesskit::NodeId;

pub enum ObjectRef {
    Managed(ObjectId<'static>),
    Unmanaged(OwnedObjectAddress),
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
