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

// Based on the standard library's impl of Error for Box<T: Error>
impl<T: InitTree> InitTree for Box<T> {
    fn init_accesskit_tree(self) -> TreeUpdate {
        InitTree::init_accesskit_tree(*self)
    }
}

// Based on the standard library's impl of From<E: Error> for Box<dyn Error>
impl<'a, T: InitTree + 'a> From<T> for Box<dyn InitTree + 'a> {
    fn from(init: T) -> Box<dyn InitTree + 'a> {
        Box::new(init)
    }
}
