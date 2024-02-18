// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use atspi_common::InterfaceSet;

use crate::{Adapter, Event, PlatformNode};

pub trait AdapterCallback {
    fn register_interfaces(&mut self, node: PlatformNode, interfaces: InterfaceSet);
    fn unregister_interfaces(&mut self, adapter: &Adapter, id: NodeId, interfaces: InterfaceSet);
    fn emit_event(&mut self, event: Event);
}
