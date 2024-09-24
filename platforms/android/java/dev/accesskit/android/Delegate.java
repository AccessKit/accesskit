// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

package dev.accesskit.android;

import android.os.Bundle;
import android.view.View;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeInfo.AccessibilityAction;
import android.view.accessibility.AccessibilityNodeProvider;

public final class Delegate extends View.AccessibilityDelegate {
    private final long adapterHandle;
    private int accessibilityFocus = AccessibilityNodeProvider.HOST_VIEW_ID;

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

    private static AccessibilityEvent newEvent(View host, int virtualViewId, int type) {
        AccessibilityEvent e = AccessibilityEvent.obtain(type);
        e.setPackageName(host.getContext().getPackageName());
        if (virtualViewId == AccessibilityNodeProvider.HOST_VIEW_ID) {
            e.setSource(host);
        } else {
            e.setSource(host, virtualViewId);
        }
        return e;
    }

    private static void sendCompletedEvent(View host, AccessibilityEvent e) {
        host.getParent().requestSendAccessibilityEvent(host, e);
    }

    private static void sendEventInternal(View host, int virtualViewId, int type) {
        AccessibilityEvent e = newEvent(host, virtualViewId, type);
        if (type == AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED) {
            e.setContentChangeTypes(AccessibilityEvent.CONTENT_CHANGE_TYPE_SUBTREE);
        }
        sendCompletedEvent(host, e);
    }

    public static void sendEvent(final View host, final int virtualViewId, final int type) {
        host.post(new Runnable() {
            @Override
            public void run() {
                sendEventInternal(host, virtualViewId, type);
            }
        });
    }

    private static void sendTextChangedInternal(View host, int virtualViewId, String oldValue, String newValue) {
        int i;
        for (i = 0; i < oldValue.length() && i < newValue.length(); ++i) {
            if (oldValue.charAt(i) != newValue.charAt(i)) {
                break;
            }
        }
        if (i >= oldValue.length() && i >= newValue.length()) {
            return; // Text did not change
        }
        AccessibilityEvent e = newEvent(host, virtualViewId, AccessibilityEvent.TYPE_VIEW_TEXT_CHANGED);
        e.setBeforeText(oldValue);
        e.getText().add(newValue);
        int firstDifference = i;
        e.setFromIndex(firstDifference);
        int oldIndex = oldValue.length() - 1;
        int newIndex = newValue.length() - 1;
        while (oldIndex >= firstDifference && newIndex >= firstDifference) {
            if (oldValue.charAt(oldIndex) != newValue.charAt(newIndex)) {
                break;
            }
            --oldIndex;
            --newIndex;
        }
        e.setRemovedCount(oldIndex - firstDifference + 1);
        e.setAddedCount(newIndex - firstDifference + 1);
        sendCompletedEvent(host, e);
    }

    public static void sendTextChanged(final View host, final int virtualViewId, final String oldValue, final String newValue) {
        host.post(new Runnable() {
            @Override
            public void run() {
                sendTextChangedInternal(host, virtualViewId, oldValue, newValue);
            }
        });
    }

    private static native boolean populateNodeInfo(long adapterHandle, View host, int screenX, int screenY, int virtualViewId, AccessibilityNodeInfo nodeInfo);
    private static native boolean performAction(long adapterHandle, View host, int virtualViewId, int action, Bundle arguments);

    @Override
    public AccessibilityNodeProvider getAccessibilityNodeProvider(final View host) {
        return new AccessibilityNodeProvider() {
            @Override
            public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualViewId) {
                int[] location = new int[2];
                host.getLocationOnScreen(location);
                AccessibilityNodeInfo nodeInfo;
                if (virtualViewId == HOST_VIEW_ID) {
                    nodeInfo = AccessibilityNodeInfo.obtain(host);
                } else {
                    nodeInfo = AccessibilityNodeInfo.obtain(host, virtualViewId);
                }
                nodeInfo.setPackageName(host.getContext().getPackageName());
                nodeInfo.setVisibleToUser(true);
                if (!populateNodeInfo(adapterHandle, host, location[0], location[1], virtualViewId, nodeInfo)) {
                    nodeInfo.recycle();
                    return null;
                }
                if (virtualViewId == accessibilityFocus) {
                    nodeInfo.setAccessibilityFocused(true);
                    nodeInfo.addAction(AccessibilityAction.ACTION_CLEAR_ACCESSIBILITY_FOCUS);
                } else {
                    nodeInfo.setAccessibilityFocused(false);
                    nodeInfo.addAction(AccessibilityAction.ACTION_ACCESSIBILITY_FOCUS);
                }
                return nodeInfo;
            }

            @Override
            public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                switch (action) {
                case AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS:
                    accessibilityFocus = virtualViewId;
                    host.invalidate();
                    sendEventInternal(host, virtualViewId, AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUSED);
                    return true;
                }
                if (!Delegate.performAction(adapterHandle, host, virtualViewId, action, arguments)) {
                    return false;
                }
                switch (action) {
                case AccessibilityNodeInfo.ACTION_CLICK:
                    sendEventInternal(host, virtualViewId, AccessibilityEvent.TYPE_VIEW_CLICKED);
                    break;
                }
                return true;
            }

            @Override
            public AccessibilityNodeInfo findFocus(int focusType) {
                if (focusType != AccessibilityNodeInfo.FOCUS_ACCESSIBILITY
                    || accessibilityFocus == HOST_VIEW_ID) {
                    return null;
                }
                return createAccessibilityNodeInfo(accessibilityFocus);
            }
        };
    }
}
