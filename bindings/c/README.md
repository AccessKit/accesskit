# AccessKit C bindings

These are the bindings to use AccessKit from other languages through FFI such as in C.

## How to use in a CMake project

Decompress an accesskit_c release and place the resulting folder somewhere:

- already known by CMake,
- listed in your `CMAKE_PREFIX_PATH`,
- indicated by the `ACCESSKIT_DIR` option.

You can then require AccessKit as a dependency by adding this to your `CMakeLists.txt`:

```cmake
find_package(ACCESSKIT REQUIRED)
```

The headers can be added like so:

```cmake
include_directories(YourProject ${ACCESSKIT_INCLUDE_DIR})
```

Finally, link the library to your executable:

```cmake
target_link_libraries(YourExecutable PUBLIC ${ACCESSKIT_LIBRARIES})
```

See [the `examples` directory](https://github.com/AccessKit/accesskit/tree/main/bindings/c/examples) for project integration examples.

## Building from source

Prerequisites:

- [Rust](https://rustup.rs/)
- [CMake](https://cmake.org/), version 3.20 or higher
- A nightly Rust toolchain: `rustup install nightly`
- [cbindgen](https://github.com/eqrion/cbindgen): `cargo install cbindgen`
- [clang-format](https://releases.llvm.org/14.0.0/tools/clang/docs/ClangFormat.html), version 14 or higher

Once inside the `bindings/c` directory, CMake can be used like this to build the project:

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```
