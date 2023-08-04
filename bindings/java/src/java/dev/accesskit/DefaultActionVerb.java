// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

public enum DefaultActionVerb {
    CLICK,
    FOCUS,
    CHECK,
    UNCHECK,
    /**
     * A click will be performed on one of the node's ancestors.
     * This happens when the node itself is not clickable, but one of its
     * ancestors has click handlers attached which are able to capture the click
     * as it bubbles up.
     */
    CLICK_ANCESTOR,
    JUMP,
    OPEN,
    PRESS,
    SELECT
}
