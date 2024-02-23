// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("defunct")]
    Defunct,
    #[error("unsupported interface")]
    UnsupportedInterface,
    #[error("too many children")]
    TooManyChildren,
    #[error("index out of range")]
    IndexOutOfRange,
}

pub type Result<T> = std::result::Result<T, Error>;
