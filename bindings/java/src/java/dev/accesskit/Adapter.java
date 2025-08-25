// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public abstract class Adapter implements AutoCloseable {
    public abstract void updateIfActive(TreeUpdateSupplier updateSupplier);

    public abstract void updateViewFocusState(boolean isFocused);
}
