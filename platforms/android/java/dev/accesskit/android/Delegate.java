// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit.android;

import android.view.View;

public final class Delegate extends View.AccessibilityDelegate {
    private final long adapterHandle;

    private Delegate(long adapterHandle) {
        super();
        this.adapterHandle = adapterHandle;
    }

    public static void inject(final View host, final long adapterHandle) {
        host.post(new Runnable() {
            @Override
            public void run() {
                if (host.getAccessibilityDelegate() != null) {
                    throw new IllegalStateException("host already has an accessibility delegate");
                }
                Delegate delegate = new Delegate(adapterHandle);
                host.setAccessibilityDelegate(delegate);
            }
        });
    }

    public static void remove(final View host) {
        host.post(new Runnable() {
            @Override
            public void run() {
                View.AccessibilityDelegate delegate = host.getAccessibilityDelegate();
                if (delegate != null && delegate instanceof Delegate) {
                    host.setAccessibilityDelegate(null);
                }
            }
        });
    }
}
