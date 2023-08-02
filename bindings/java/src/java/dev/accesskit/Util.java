// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

import java.nio.charset.StandardCharsets;

final class Util {
    static void checkActive(long ptr) {
        if (ptr == 0) {
            throw new IllegalStateException("already dropped");
        }
    }

    static byte[] bytesFromString(String s) {
        return s.getBytes(StandardCharsets.UTF_8);
    }
}
