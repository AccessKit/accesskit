// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public final class Node {
    Node(long ptr) {
        this.ptr = ptr;
    }

    /**
     * Releases resources associated with this object. In normal usage,
     * you don't need to call this method, since the tree update
     * takes ownership of each node that is added.
     */
    public void drop() {
        if (ptr != 0) {
            nativeDrop(ptr);
            ptr = 0;
        }
    }

    long ptr;
    private static native void nativeDrop(long ptr);

    void checkActive() {
        Util.checkActive(ptr);
    }
}
