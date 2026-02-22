// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

//! Generator for the committed `accesskit_l10n` lookup code.
//!
//! Reads the translation files in `l10n/l10n/*.txt` and writes the generated
//! Rust source to `l10n/src/generated.rs`. That file is committed to the
//! repository so the `accesskit_l10n` crate builds without a build script.
//!
//! Usage:
//! - `cargo run -p accesskit_l10n_build` — regenerate and write the file.
//! - `cargo run -p accesskit_l10n_build -- --check` — verify the committed file
//!   is up to date (used in CI); exits non-zero if it is stale.

use accesskit_l10n_build::generate_from_dir;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let l10n_crate = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("l10n_build should have a parent directory")
        .join("l10n");
    let input_dir = l10n_crate.join("l10n");
    let output_file = l10n_crate.join("src").join("generated.rs");

    let (code, warnings) = match generate_from_dir(&input_dir) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    for warning in &warnings {
        eprintln!("warning: {warning}");
    }

    if std::env::args().any(|arg| arg == "--check") {
        let existing = std::fs::read_to_string(&output_file).unwrap_or_default();
        if existing != code {
            eprintln!("error: {} is out of date.", output_file.display());
            eprintln!("Run `cargo run -p accesskit_l10n_build` and commit the result.");
            return ExitCode::FAILURE;
        }
        println!("{} is up to date.", output_file.display());
    } else {
        if let Err(e) = std::fs::write(&output_file, &code) {
            eprintln!("error: failed to write {}: {e}", output_file.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {}", output_file.display());
    }

    ExitCode::SUCCESS
}
