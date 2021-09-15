// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

pub use accesskit_schema::{Node as NodeData, Tree as TreeData};

pub(crate) mod tree;
pub use tree::{Reader as TreeReader, Tree};

pub(crate) mod node;
pub use node::{Node, WeakNode};
