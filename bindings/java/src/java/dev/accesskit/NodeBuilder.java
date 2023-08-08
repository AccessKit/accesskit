// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public final class NodeBuilder {
    public NodeBuilder(Role role) {
        ptr = nativeNew(role.ordinal());
    }

    /**
     * Releases resources associated with this object. In normal usage,
     * you don't need to call this method, since the node builder
     * is consumed when you build the node.
     */
    public void drop() {
        if (ptr != 0) {
            nativeDrop(ptr);
            ptr = 0;
        }
    }

    public void addAction(Action action) {
        checkActive();
        nativeAddAction(ptr, action.ordinal());
    }

    public void setDefaultActionVerb(DefaultActionVerb value) {
        checkActive();
        nativeSetDefaultActionVerb(ptr, value.ordinal());
    }

    public void setName(String value) {
        checkActive();
        nativeSetName(ptr, Util.bytesFromString(value));
    }

    public void setValue(String value) {
        checkActive();
        nativeSetValue(ptr, Util.bytesFromString(value));
    }

    public void setBounds(Rect value) {
        checkActive();
        nativeSetBounds(ptr, value.x0, value.y0, value.x1, value.y1);
    }

    public void addChild(NodeId id) {
        checkActive();
        nativeAddChild(ptr, id.low, id.high);
    }

    public void clearChildren() {
        checkActive();
        nativeClearChildren(ptr);
    }

    public void setChildren(Iterable<NodeId> ids) {
        clearChildren();
        for (NodeId id : ids) {
            addChild(id);
        }
    }

    public void setMultiline() {
        checkActive();
        nativeSetMultiline(ptr);
    }

    public void clearMultiline() {
        checkActive();
        nativeClearMultiline(ptr);
    }

    public void setCheckedState(CheckedState value) {
        checkActive();
        nativeSetCheckedState(ptr, value.ordinal());
    }

    public void setLive(Live value) {
        checkActive();
        nativeSetLive(ptr, value.ordinal());
    }

    public void setTextDirection(TextDirection value) {
        checkActive();
        nativeSetTextDirection(ptr, value.ordinal());
    }

    public void setNumericValue(double value) {
        checkActive();
        nativeSetNumericValue(ptr, value);
    }

    public void setMinNumericValue(double value) {
        checkActive();
        nativeSetMinNumericValue(ptr, value);
    }

    public void setMaxNumericValue(double value) {
        checkActive();
        nativeSetMaxNumericValue(ptr, value);
    }

    public void setNumericValueStep(double value) {
        checkActive();
        nativeSetNumericValueStep(ptr, value);
    }

    public void setNumericValueJump(double value) {
        checkActive();
        nativeSetNumericValueJump(ptr, value);
    }

    public void setTextSelection(TextSelection value) {
        checkActive();
        nativeSetTextSelection(ptr, value.anchor.node.low, value.anchor.node.high, value.anchor.characterIndex, value.focus.node.low, value.focus.node.high, value.focus.characterIndex);
    }

    public Node build() {
        checkActive();
        long nodePtr = nativeBuild(ptr);
        ptr = 0;
        return new Node(nodePtr);
    }

    long ptr;
    private static native long nativeNew(int role);
    private static native void nativeDrop(long ptr);
    private static native void nativeAddAction(long ptr, int action);
    private static native void nativeSetDefaultActionVerb(long ptr, int value);
    private static native void nativeSetName(long ptr, byte[] value);
    private static native void nativeSetValue(long ptr, byte[] value);
    private static native void nativeSetBounds(long ptr, double x0, double y0, double x1, double y1);
    private static native void nativeAddChild(long ptr, long idLow, long idHigh);
    private static native void nativeClearChildren(long ptr);
    private static native void nativeSetMultiline(long ptr);
    private static native void nativeClearMultiline(long ptr);
    private static native void nativeSetCheckedState(long ptr, int value);
    private static native void nativeSetLive(long ptr, int value);
    private static native void nativeSetTextDirection(long ptr, int value);
    private static native void nativeSetNumericValue(long ptr, double value);
    private static native void nativeSetMinNumericValue(long ptr, double value);
    private static native void nativeSetMaxNumericValue(long ptr, double value);
    private static native void nativeSetNumericValueStep(long ptr, double value);
    private static native void nativeSetNumericValueJump(long ptr, double value);
    private static native void nativeSetTextSelection(long ptr, long anchorNodeIdLow, long anchorNodeIdHigh, int anchorCharacterIndex, long focusNodeIdLow, long focusNodeIdHigh, int focusCharacterIndex);
    private static native long nativeBuild(long ptr);

    void checkActive() {
        Util.checkActive(ptr);
    }
}
