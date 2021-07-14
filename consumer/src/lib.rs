// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub use accesskit_schema::{Node as NodeData, Tree as TreeData};

pub(crate) mod tree;
pub use tree::{Reader as TreeReader, Tree};

pub(crate) mod node;
pub use node::{Node, WeakNode};
