# AccessKit C bindings

These are the bindings to use AccessKit from other languages through FFI such as in C.

## Prerequisites

- [Rust](https://rustup.rs/)
- A nightly Rust toolchain: `rustup install nightly`
- [cbindgen](https://github.com/eqrion/cbindgen): `cargo install cbindgen`
- [clang-format](https://releases.llvm.org/14.0.0/tools/clang/docs/ClangFormat.html), version 14 or higher

## Building

Inside the `bindings/c` directory, some of the available make recipes include:

- `make include/accesskit.h`: generate the header files
- `make ../../target/release/libaccesskit.a`: build static and dynamic libraries on Linux
- `make clean`: remove generated artifacts
