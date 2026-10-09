package dev.accesskit.showcase;

import android.content.Context;
import android.graphics.Insets;
import android.os.Bundle;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.View;
import android.view.WindowInsets;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeProvider;

public final class ShowcaseView extends View {
    static {
        System.loadLibrary("showcase");
    }

    private static final int DARK_GRAY = 0xff181818;

    private long nativeHandle;

    private Insets safeArea = Insets.NONE;

    private final Runnable processDueMessages =
            new Runnable() {
                @Override
                public void run() {
                    if (nativeHandle != 0) {
                        nativeProcessDueMessages(nativeHandle, ShowcaseView.this);
                    }
                }
            };

    private final AccessibilityNodeProvider nodeProvider =
            new AccessibilityNodeProvider() {
                @Override
                public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualViewId) {
                    if (nativeHandle == 0) {
                        return null;
                    }
                    return nativeCreateAccessibilityNodeInfo(
                            nativeHandle, ShowcaseView.this, virtualViewId);
                }

                @Override
                public AccessibilityNodeInfo findFocus(int focusType) {
                    if (nativeHandle == 0) {
                        return null;
                    }
                    return nativeFindFocus(nativeHandle, ShowcaseView.this, focusType);
                }

                @Override
                public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                    if (nativeHandle == 0) {
                        return false;
                    }
                    return nativePerformAction(
                            nativeHandle, ShowcaseView.this, virtualViewId, action, arguments);
                }
            };

    private static native long nativeCreate();

    private static native void nativeDestroy(long handle);

    private static native void nativeSetViewport(
            long handle,
            View host,
            float scaleFactor,
            float safeAreaInsetX,
            float safeAreaInsetY,
            float safeAreaWidth,
            float safeAreaHeight);

    private static native AccessibilityNodeInfo nativeCreateAccessibilityNodeInfo(
            long handle, View host, int virtualViewId);

    private static native AccessibilityNodeInfo nativeFindFocus(
            long handle, View host, int focusType);

    private static native boolean nativePerformAction(
            long handle, View host, int virtualViewId, int action, Bundle arguments);

    private static native boolean nativeOnHoverEvent(
            long handle, View host, int action, float x, float y);

    private static native boolean nativeOnKeyEvent(
            long handle, View host, int action, int keyCode, boolean shiftPressed);

    private static native void nativeProcessDueMessages(long handle, View host);

    public ShowcaseView(Context context) {
        super(context);
        setBackgroundColor(DARK_GRAY);
        setFocusable(true);
        setFocusableInTouchMode(true);
    }

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        nativeHandle = nativeCreate();
    }

    @Override
    protected void onDetachedFromWindow() {
        removeCallbacks(processDueMessages);
        nativeDestroy(nativeHandle);
        nativeHandle = 0;
        super.onDetachedFromWindow();
    }

    @Override
    public AccessibilityNodeProvider getAccessibilityNodeProvider() {
        return nodeProvider;
    }

    @Override
    public WindowInsets onApplyWindowInsets(WindowInsets insets) {
        safeArea =
                insets.getInsets(
                        WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
        updateViewport();
        return super.onApplyWindowInsets(insets);
    }

    @Override
    protected void onSizeChanged(int width, int height, int oldWidth, int oldHeight) {
        super.onSizeChanged(width, height, oldWidth, oldHeight);
        updateViewport();
    }

    private void updateViewport() {
        if (nativeHandle == 0) {
            return;
        }
        float scaleFactor = getResources().getDisplayMetrics().density;
        nativeSetViewport(
                nativeHandle,
                this,
                scaleFactor,
                safeArea.left,
                safeArea.top,
                getWidth() - safeArea.left - safeArea.right,
                getHeight() - safeArea.top - safeArea.bottom);
    }

    @Override
    public boolean onHoverEvent(MotionEvent event) {
        if (nativeHandle != 0
                && nativeOnHoverEvent(
                        nativeHandle, this, event.getAction(), event.getX(), event.getY())) {
            return true;
        }
        return super.onHoverEvent(event);
    }

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        return handleKeyEvent(event) || super.onKeyDown(keyCode, event);
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        return handleKeyEvent(event) || super.onKeyUp(keyCode, event);
    }

    private boolean handleKeyEvent(KeyEvent event) {
        if (nativeHandle == 0) {
            return false;
        }
        return nativeOnKeyEvent(
                nativeHandle, this, event.getAction(), event.getKeyCode(), event.isShiftPressed());
    }

    /** Called from Rust. */
    private void scheduleDelayedMessage(long delayMillis) {
        removeCallbacks(processDueMessages);
        postDelayed(processDueMessages, delayMillis);
    }
}
