// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use libtest_mimic::{Arguments, Trial};
use std::process::ExitCode;

pub struct Test {
    pub name: &'static str,
    pub run: fn(),
}

macro_rules! tests {
    ($($name:ident),* $(,)?) => {
        &[$($crate::harness::Test { name: stringify!($name), run: $name }),*]
    };
}
pub(crate) use tests;

pub fn run(suites: &[(&str, &[Test])]) -> ExitCode {
    let mut args = Arguments::from_args();
    // libtest-mimic only runs tests on the main thread when it uses a single thread.
    args.test_threads = Some(1);
    let trials = suites
        .iter()
        .flat_map(|(suite, tests)| {
            tests.iter().map(move |test| {
                let run = test.run;
                Trial::test(format!("{suite}::{}", test.name), move || {
                    run();
                    Ok(())
                })
            })
        })
        .collect();
    libtest_mimic::run(&args, trials).exit_code()
}
