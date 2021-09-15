// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit.mac;

public final class AccessKitMacManager implements AutoCloseable {
    public static AccessKitMacManager forNSWindow(long nsWindow, String initialStateJson) {
        long ptr = nativeNewForNSWindow(nsWindow, initialStateJson);
        return new AccessKitMacManager(ptr);
    }

    public static AccessKitMacManager forNSView(long nsView, String initialStateJson) {
        long ptr = nativeNewForNSView(nsView, initialStateJson);
        return new AccessKitMacManager(ptr);
    }

    @Override
    public void close() {
        if (ptr != 0) {
            nativeDestroy(ptr);
            ptr = 0;
        }
    }

    @Override
    public void finalize() {
        close();
    }

    public void update(String updateJson) {
        checkActive();
        nativeUpdate(ptr, updateJson);
    }

    /**
     * Inject the accessibility tree into the specified window or view.
     */
    public void inject() {
        checkActive();
        nativeInject(ptr);
    }

    private long ptr;

    private AccessKitMacManager(long ptr) {
        this.ptr = ptr;
    }

    private void checkActive() {
        if (ptr == 0) {
            throw new IllegalStateException("already closed");
        }
    }

    private static native long nativeNewForNSWindow(long nsWindow, String initialStateJson);
    private static native long nativeNewForNSView(long nsView, String initialStateJson);
    private static native void nativeDestroy(long ptr);
    private static native void nativeUpdate(long ptr, String updateJson);
    private static native void nativeInject(long ptr);

    static {
        System.loadLibrary("accesskit_mac_jni");
    }
}
