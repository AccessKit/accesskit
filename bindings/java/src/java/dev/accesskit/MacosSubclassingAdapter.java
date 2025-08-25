// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public final class MacosSubclassingAdapter extends Adapter {
    // TODO: action handler
    public MacosSubclassingAdapter(long view, TreeUpdateSupplier initialStateSupplier) {
        NativePointerSupplier nativeSupplier = TreeUpdate.makeNativeSupplier(initialStateSupplier);
        ptr = nativeNew(view, nativeSupplier);
    }

    MacosSubclassingAdapter(long ptr) {
        this.ptr = ptr;
    }

    // TODO: action handler
    public static MacosSubclassingAdapter forWindow(long window, TreeUpdateSupplier initialStateSupplier) {
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

    @Override
    public void updateIfActive(TreeUpdateSupplier updateSupplier) {
        checkActive();
        NativePointerSupplier nativeSupplier = TreeUpdate.makeNativeSupplier(updateSupplier);
        nativeUpdateIfActive(ptr, nativeSupplier);
    }

    @Override
    public void updateViewFocusState(boolean isFocused) {
        checkActive();
        nativeUpdateViewFocusState(ptr, isFocused);
    }

    long ptr;
    private static native long nativeNew(long view, NativePointerSupplier initialStateSupplier);
    private static native long nativeForWindow(long window, NativePointerSupplier initialStateSupplier);
    private static native void nativeDrop(long ptr);
    private static native void nativeUpdateIfActive(long ptr, NativePointerSupplier updateSupplier);
    private static native void nativeUpdateViewFocusState(long ptr, boolean isFocused);

    void checkActive() {
        Util.checkActive(ptr);
    }
}
