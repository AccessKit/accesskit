// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

mod harness;
mod subclassing_adapter;
mod util;

fn main() -> std::process::ExitCode {
    harness::run(&[("subclassing_adapter", subclassing_adapter::TESTS)])
}
