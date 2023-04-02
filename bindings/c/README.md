# AccessKit C bindings

These are the bindings to use AccessKit from other languages through FFI such as in C.

## Prerequisites

- [Rust](https://rustup.rs/)
- [CMake](https://cmake.org/), version 3.20 or higher
- A nightly Rust toolchain: `rustup install nightly`
- [cbindgen](https://github.com/eqrion/cbindgen): `cargo install cbindgen`
- [clang-format](https://releases.llvm.org/14.0.0/tools/clang/docs/ClangFormat.html), version 14 or higher

## Building

Once inside the `bindings/c` directory, CMake can be used like so:

```
cmake -S . -B build # See below for available configuration options.
cd build
cmake --build .
```

Available configuration options:

- `BUILD_WINDOWS_EXAMPLE`: Builds the Windows example program
