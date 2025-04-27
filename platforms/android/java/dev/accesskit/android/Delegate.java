// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit.android;

import android.os.Bundle;
import android.view.MotionEvent;
import android.view.View;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeProvider;

public final class Delegate extends View.AccessibilityDelegate implements View.OnHoverListener {
    private final long adapterHandle;

    private Delegate(long adapterHandle) {
        super();
        this.adapterHandle = adapterHandle;
    }

    public static void inject(final View host, final long adapterHandle) {
        host.post(
                new Runnable() {
                    @Override
                    public void run() {
                        if (host.getAccessibilityDelegate() != null) {
                            throw new IllegalStateException(
                                    "host already has an accessibility delegate");
                        }
                        Delegate delegate = new Delegate(adapterHandle);
                        host.setAccessibilityDelegate(delegate);
                        host.setOnHoverListener(delegate);
                    }
                });
    }

    public static void remove(final View host) {
        host.post(
                new Runnable() {
                    @Override
                    public void run() {
                        View.AccessibilityDelegate delegate = host.getAccessibilityDelegate();
                        if (delegate != null && delegate instanceof Delegate) {
                            host.setAccessibilityDelegate(null);
                            host.setOnHoverListener(null);
                        }
                    }
                });
    }

    private static native void runCallback(View host, long handle);

    private static native AccessibilityNodeInfo createAccessibilityNodeInfo(
            long adapterHandle, View host, int virtualViewId);

    private static native AccessibilityNodeInfo findFocus(
            long adapterHandle, View host, int focusType);

    private static native boolean performAction(
            long adapterHandle, View host, int virtualViewId, int action, Bundle arguments);

    private static native boolean onHoverEvent(
            long adapterHandle, View host, int action, float x, float y);

    public static Runnable newCallback(final View host, final long handle) {
        return new Runnable() {
            @Override
            public void run() {
                runCallback(host, handle);
            }
        };
    }

    @Override
    public AccessibilityNodeProvider getAccessibilityNodeProvider(final View host) {
        return new AccessibilityNodeProvider() {
            @Override
            public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualViewId) {
                return Delegate.createAccessibilityNodeInfo(adapterHandle, host, virtualViewId);
            }

            @Override
            public AccessibilityNodeInfo findFocus(int focusType) {
                return Delegate.findFocus(adapterHandle, host, focusType);
            }

            @Override
            public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                return Delegate.performAction(
                        adapterHandle, host, virtualViewId, action, arguments);
            }
        };
    }

    @Override
    public boolean onHover(View v, MotionEvent event) {
        return onHoverEvent(adapterHandle, v, event.getAction(), event.getX(), event.getY());
    }
}
