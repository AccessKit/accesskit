package dev.accesskit.showcase;

import android.content.Context;
import android.graphics.Insets;
import android.os.Bundle;
import android.os.Handler;
import android.text.TextUtils;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.View;
import android.view.WindowInsets;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeProvider;
import android.view.inputmethod.CompletionInfo;
import android.view.inputmethod.CorrectionInfo;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.ExtractedText;
import android.view.inputmethod.ExtractedTextRequest;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputContentInfo;
import android.view.inputmethod.InputMethodManager;

public final class ShowcaseView extends View {
    static {
        System.loadLibrary("showcase");
    }

    private static final int DARK_GRAY = 0xff181818;

    // The requests of `ShowcaseInputConnection`, as the Rust side expects them.
    private static final int IME_TEXT_BEFORE_CURSOR = 0;
    private static final int IME_TEXT_AFTER_CURSOR = 1;
    private static final int IME_SELECTED_TEXT = 2;
    private static final int IME_COMMIT_TEXT = 0;
    private static final int IME_SET_COMPOSING_TEXT = 1;
    private static final int IME_FINISH_COMPOSING_TEXT = 2;
    private static final int IME_SET_COMPOSING_REGION = 3;
    private static final int IME_DELETE_SURROUNDING_TEXT = 4;
    private static final int IME_DELETE_SURROUNDING_TEXT_IN_CODE_POINTS = 5;
    private static final int IME_SET_SELECTION = 6;
    private static final int IME_PERFORM_EDITOR_ACTION = 7;

    private long nativeHandle;

    private Insets safeArea = Insets.NONE;

    /** Whether a text input has focus. */
    private boolean isTextEditor;

    private final InputMethodManager inputMethodManager;

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
            long handle,
            View host,
            int action,
            int keyCode,
            int unicodeChar,
            boolean shiftPressed,
            boolean ctrlPressed);

    private static native void nativeProcessDueMessages(long handle, View host);

    private static native boolean nativeCreateInputConnection(
            long handle, View host, EditorInfo editorInfo);

    private static native String nativeImeGetText(long handle, View host, int request, int length);

    private static native boolean nativeImeEdit(
            long handle, View host, int request, String text, int a, int b);

    private static native boolean nativeImeBatchEdit(long handle, View host, boolean begin);

    public ShowcaseView(Context context) {
        super(context);
        setBackgroundColor(DARK_GRAY);
        setFocusable(true);
        setFocusableInTouchMode(true);
        inputMethodManager = context.getSystemService(InputMethodManager.class);
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
                        WindowInsets.Type.systemBars()
                                | WindowInsets.Type.displayCutout()
                                | WindowInsets.Type.ime());
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
        int unicodeChar = event.isCtrlPressed() ? 0 : event.getUnicodeChar();
        return nativeOnKeyEvent(
                nativeHandle,
                this,
                event.getAction(),
                event.getKeyCode(),
                unicodeChar,
                event.isShiftPressed(),
                event.isCtrlPressed());
    }

    /** Called from Rust. */
    private void scheduleDelayedMessage(long delayMillis) {
        removeCallbacks(processDueMessages);
        postDelayed(processDueMessages, delayMillis);
    }

    @Override
    public boolean onCheckIsTextEditor() {
        return isTextEditor;
    }

    @Override
    public InputConnection onCreateInputConnection(EditorInfo outAttrs) {
        if (nativeHandle == 0 || !nativeCreateInputConnection(nativeHandle, this, outAttrs)) {
            return null;
        }
        return new ShowcaseInputConnection();
    }

    /**
     * Called from Rust when focus moves to or from a text input, it changes type, or its text
     * changes without the input method editing it.
     */
    private void restartInput(boolean isTextEditor) {
        this.isTextEditor = isTextEditor;
        inputMethodManager.restartInput(this);
        if (!isTextEditor) {
            inputMethodManager.hideSoftInputFromWindow(getWindowToken(), 0);
        }
    }

    /** Called from Rust when a text input is activated. */
    private void showKeyboard() {
        requestFocus();
        inputMethodManager.showSoftInput(this, 0);
    }

    /** Called from Rust when the selection or composition of the focused input changes. */
    private void updateSelection(
            int selectionStart, int selectionEnd, int compositionStart, int compositionEnd) {
        inputMethodManager.updateSelection(
                this, selectionStart, selectionEnd, compositionStart, compositionEnd);
    }

    /** Forwards what the input method does to the focused text input. */
    private final class ShowcaseInputConnection implements InputConnection {
        private String getText(int request, int length) {
            if (nativeHandle == 0) {
                return null;
            }
            return nativeImeGetText(nativeHandle, ShowcaseView.this, request, length);
        }

        private boolean edit(int request, String text, int a, int b) {
            if (nativeHandle == 0) {
                return false;
            }
            return nativeImeEdit(nativeHandle, ShowcaseView.this, request, text, a, b);
        }

        @Override
        public CharSequence getTextBeforeCursor(int n, int flags) {
            return getText(IME_TEXT_BEFORE_CURSOR, n);
        }

        @Override
        public CharSequence getTextAfterCursor(int n, int flags) {
            return getText(IME_TEXT_AFTER_CURSOR, n);
        }

        @Override
        public CharSequence getSelectedText(int flags) {
            return getText(IME_SELECTED_TEXT, 0);
        }

        @Override
        public int getCursorCapsMode(int reqModes) {
            CharSequence before = getTextBeforeCursor(Integer.MAX_VALUE, 0);
            if (before == null) {
                return 0;
            }
            return TextUtils.getCapsMode(before, before.length(), reqModes);
        }

        @Override
        public ExtractedText getExtractedText(ExtractedTextRequest request, int flags) {
            return null;
        }

        @Override
        public boolean deleteSurroundingText(int beforeLength, int afterLength) {
            return edit(IME_DELETE_SURROUNDING_TEXT, null, beforeLength, afterLength);
        }

        @Override
        public boolean deleteSurroundingTextInCodePoints(int beforeLength, int afterLength) {
            return edit(
                    IME_DELETE_SURROUNDING_TEXT_IN_CODE_POINTS, null, beforeLength, afterLength);
        }

        @Override
        public boolean setComposingText(CharSequence text, int newCursorPosition) {
            return edit(IME_SET_COMPOSING_TEXT, text.toString(), newCursorPosition, 0);
        }

        @Override
        public boolean setComposingRegion(int start, int end) {
            return edit(IME_SET_COMPOSING_REGION, null, start, end);
        }

        @Override
        public boolean finishComposingText() {
            return edit(IME_FINISH_COMPOSING_TEXT, null, 0, 0);
        }

        @Override
        public boolean commitText(CharSequence text, int newCursorPosition) {
            return edit(IME_COMMIT_TEXT, text.toString(), newCursorPosition, 0);
        }

        @Override
        public boolean commitCompletion(CompletionInfo text) {
            return false;
        }

        @Override
        public boolean commitCorrection(CorrectionInfo correctionInfo) {
            return false;
        }

        @Override
        public boolean setSelection(int start, int end) {
            return edit(IME_SET_SELECTION, null, start, end);
        }

        @Override
        public boolean performEditorAction(int editorAction) {
            return edit(IME_PERFORM_EDITOR_ACTION, null, editorAction, 0);
        }

        @Override
        public boolean performContextMenuAction(int id) {
            return false;
        }

        @Override
        public boolean beginBatchEdit() {
            return nativeHandle != 0 && nativeImeBatchEdit(nativeHandle, ShowcaseView.this, true);
        }

        @Override
        public boolean endBatchEdit() {
            return nativeHandle != 0 && nativeImeBatchEdit(nativeHandle, ShowcaseView.this, false);
        }

        @Override
        public boolean sendKeyEvent(KeyEvent event) {
            return dispatchKeyEvent(event);
        }

        @Override
        public boolean clearMetaKeyStates(int states) {
            return false;
        }

        @Override
        public boolean reportFullscreenMode(boolean enabled) {
            return false;
        }

        @Override
        public boolean performPrivateCommand(String action, Bundle data) {
            return false;
        }

        @Override
        public boolean requestCursorUpdates(int cursorUpdateMode) {
            return false;
        }

        @Override
        public Handler getHandler() {
            return null;
        }

        @Override
        public void closeConnection() {}

        @Override
        public boolean commitContent(InputContentInfo inputContentInfo, int flags, Bundle opts) {
            return false;
        }
    }
}
