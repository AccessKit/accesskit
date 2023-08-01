// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

/**
 * The stable identity of a node, unique within the node's tree.
 *
 * This is a 128-bit integer, but Java doesn't have a 128-bit integer type,
 * so we represent it as 64-bit low and high parts. The full integer must be
 * non-zero.
 */
public final class NodeId {
    public NodeId(long low, long high) {
        if (low == 0 && high == 0) {
            throw new IllegalArgumentException("node ID must be non-zero");
        }
        this.low = low;
        this.high = high;
    }

    public NodeId(long low) {
        this(low, 0);
    }

    public final long low;
    public final long high;

    @Override
    public boolean equals(Object other) {
        if (!(other instanceof NodeId)) {
            return false;
        }
        NodeId otherNodeId = (NodeId)other;
        return this.low == otherNodeId.low && this.high == otherNodeId.high;
    }

    @Override
    public int hashCode() {
        // TODO: better way of combining hash codes?
        return Long.hashCode(low) | Long.hashCode(high);
    }

    @Override
    public String toString() {
        if (high == 0) {
            return "NodeId(" + Long.toString(low) + ")";
        }
        // TODO: do we want to try to format as a single number?
        return "NodeId(" + Long.toString(high) + " " + Long.toString(low) + ")";
    }
}
