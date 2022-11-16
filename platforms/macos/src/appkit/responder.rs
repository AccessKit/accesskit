// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use objc2::{extern_class, foundation::NSObject, ClassType};

extern_class!(
    #[derive(Debug)]
    pub struct NSResponder;

    unsafe impl ClassType for NSResponder {
        type Super = NSObject;
    }
);
