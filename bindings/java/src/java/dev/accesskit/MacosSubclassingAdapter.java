// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public final class MacosSubclassingAdapter implements AutoCloseable {
    // TODO: action handler
    public MacosSubclassingAdapter(long view, TreeUpdate.Supplier initialStateSupplier) {
        NativePointerSupplier nativeSupplier = TreeUpdate.makeNativeSupplier(initialStateSupplier);
        ptr = nativeNew(view, nativeSupplier);
    }

    MacosSubclassingAdapter(long ptr) {
        this.ptr = ptr;
    }

    // TODO: action handler
    public static MacosSubclassingAdapter forWindow(long window, TreeUpdate.Supplier initialStateSupplier) {
        NativePointerSupplier nativeSupplier = TreeUpdate.makeNativeSupplier(initialStateSupplier);
        return new MacosSubclassingAdapter(nativeForWindow(window, nativeSupplier));
    }

    @Override
    public void close() {
        if (ptr != 0) {
            nativeDrop(ptr);
            ptr = 0;
        }
    }

    public void updateIfActive(TreeUpdate.Supplier updateSupplier) {
        checkActive();
        NativePointerSupplier nativeSupplier = TreeUpdate.makeNativeSupplier(updateSupplier);
        nativeUpdateIfActive(ptr, nativeSupplier);
    }

    long ptr;
    private static native long nativeNew(long view, NativePointerSupplier initialStateSupplier);
    private static native long nativeForWindow(long window, NativePointerSupplier initialStateSupplier);
    private static native void nativeDrop(long ptr);
    private static native void nativeUpdateIfActive(long ptr, NativePointerSupplier updateSupplier);

    void checkActive() {
        Util.checkActive(ptr);
    }
}
