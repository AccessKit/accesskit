// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

/**
 * The stable identity of a node, unique within the node's tree.
 */
public final class NodeId {
    public NodeId(long value) {
        this.value = value;
    }

    public final long value;

    @Override
    public boolean equals(Object other) {
        if (!(other instanceof NodeId)) {
            return false;
        }
        NodeId otherNodeId = (NodeId)other;
        return this.value == otherNodeId.value;
    }

    @Override
    public int hashCode() {
        return Long.hashCode(value);
    }

    @Override
    public String toString() {
        return "NodeId(" + Long.toString(value) + ")";
    }
}
