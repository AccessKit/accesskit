// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public final class TreeUpdate {
    public TreeUpdate() {
        ptr = nativeNew();
    }

    /**
     * Releases resources associated with this object. In normal usage,
     * you don't need to call this method, since the platform adapter
     * takes ownership of the tree update once you push it.
     */
    public void drop() {
        if (ptr != 0) {
            nativeDrop(ptr);
            ptr = 0;
        }
    }

    private long ptr;
    private static native long nativeNew();
    private static native void nativeDrop(long ptr);
}
