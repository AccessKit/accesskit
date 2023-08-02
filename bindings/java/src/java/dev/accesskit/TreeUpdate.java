// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

import java.util.function.Supplier;

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

    public void add(NodeId id, Node node) {
        checkActive();
        node.checkActive();
        nativeAddNode(ptr, id.low, id.high, node.ptr);
        node.ptr = 0;
    }

    public void setTree(Tree tree) {
        checkActive();
        long rootScrollerLow, rootScrollerHigh;
        if (tree.rootScroller == null) {
            rootScrollerLow = 0;
            rootScrollerHigh = 0;
        } else {
            rootScrollerLow = tree.rootScroller.low;
            rootScrollerHigh = tree.rootScroller.high;
        }
        nativeSetTree(ptr, tree.root.low, tree.root.high, rootScrollerLow, rootScrollerHigh);
    }

    public void clearTree() {
        checkActive();
        nativeClearTree(ptr);
    }

    public void setFocus(NodeId id) {
        checkActive();
        nativeSetFocus(ptr, id.low, id.high);
    }

    public void clearFocus() {
        checkActive();
        nativeClearFocus(ptr);
    }

    long ptr;
    private static native long nativeNew();
    private static native void nativeDrop(long ptr);
    private static native void nativeAddNode(long ptr, long idLow, long idHigh, long nodePtr);
    private static native void nativeSetTree(long ptr, long rootLow, long rootHigh, long rootScrollerLow, long rootScrollerHigh);
    private static native void nativeClearTree(long ptr);
    private static native void nativeSetFocus(long ptr, long idLow, long idHigh);
    private static native void nativeClearFocus(long ptr);

    void checkActive() {
        Util.checkActive(ptr);
    }

    static NativePointerSupplier makeNativeSupplier(Supplier<TreeUpdate> supplier) {
        return () -> {
            TreeUpdate update = supplier.get();
            update.checkActive();
            long ptr = update.ptr;
            update.ptr = 0;
            return ptr;
        };
    }
}
