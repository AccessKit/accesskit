// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public final class TreeUpdate {
    private TreeUpdate(long ptr) {
        this.ptr = ptr;
    }

    public static TreeUpdate withFocus(NodeId focus) {
        return new TreeUpdate(nativeWithFocus(focus.value));
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
        nativeAddNode(ptr, id.value, node.ptr);
        node.ptr = 0;
    }

    public void setTree(Tree tree) {
        checkActive();
        nativeSetTree(ptr, tree.root.value);
    }

    public void clearTree() {
        checkActive();
        nativeClearTree(ptr);
    }

    public void setFocus(NodeId id) {
        checkActive();
        nativeSetFocus(ptr, id.value);
    }

    long ptr;
    private static native long nativeWithFocus(long focus);
    private static native void nativeDrop(long ptr);
    private static native void nativeAddNode(long ptr, long id, long nodePtr);
    private static native void nativeSetTree(long ptr, long root);
    private static native void nativeClearTree(long ptr);
    private static native void nativeSetFocus(long ptr, long id);

    void checkActive() {
        Util.checkActive(ptr);
    }

    static NativePointerSupplier makeNativeSupplier(TreeUpdateSupplier supplier) {
        return () -> {
            try {
                TreeUpdate update = supplier.get();
                update.checkActive();
                long ptr = update.ptr;
                update.ptr = 0;
                return ptr;
            } catch (Exception e) {
                // Don't make the native side print the stack trace.
                e.printStackTrace();
                throw e;
            }
        };
    }
}
