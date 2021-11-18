// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_schema::TreeUpdate;

pub trait InitTree {
    fn init_accesskit_tree(self) -> TreeUpdate;
}

impl InitTree for TreeUpdate {
    fn init_accesskit_tree(self) -> Self {
        self
    }
}
