// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

/**
 * The data associated with an accessibility tree that's global to the
 * tree and not associated with any particular node.
 */
public final class Tree {
    public Tree(NodeId root) {
        this.root = root;
    }

    public NodeId root;

    /**
     * The node that's used as the root scroller, if any. On some platforms
     * like Android we need to ignore accessibility scroll offsets for
     * that node and get them from the viewport instead.
     */
    public NodeId rootScroller = null;
}
