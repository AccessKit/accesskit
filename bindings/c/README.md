# AccessKit C bindings

These are the bindings to use AccessKit from other languages through FFI such as in C.

## How to use in a CMake project

Decompress an accesskit_c release and place the resulting folder somewhere either:

- already known by CMake,
- listed in your `CMAKE_PREFIX_PATH`,
- indicated by the `ACCESSKIT_DIR` option.

You can then require AccessKit as a dependency by adding this to your `CMakeLists.txt`:

```cmake
find_package(ACCESSKIT REQUIRED)
```

Finally, link the library to your executable:

```cmake
target_link_libraries(hello_world PUBLIC accesskit)
```

See [the `examples` directory](https://github.com/AccessKit/accesskit/tree/main/bindings/c/examples) for project integration examples.

## Building from source

Prerequisites:

- [Rust](https://rustup.rs/)
- [CMake](https://cmake.org/), version 3.20 or higher

Once inside the `bindings/c` directory, CMake can be used like this to build the project:

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
cmake --install build
```

### Notes on cross-compiling

On Windows, you will need to pass the `-A` flag when configuring the project. For instance, to target ARM64:

```bash
cmake -S . -B build -A ARM64
```

On other platforms you will have to specify which Rust target to use, as well as tell CMake for which architecture to compile. Here is how you would cross-compile for Linux X86 from Linux AMD64:

```bash
cmake -S . -B build -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=x86 -DRust_CARGO_TARGET=i686-unknown-linux-gnu
```

### Regenerating the header file

If you modify the C bindings, you need to regenerate the header file and commit it. To do this, in addition to the above requirements, you will need:

- A nightly Rust toolchain: `rustup install nightly`
- [cbindgen](https://github.com/mozilla/cbindgen): `cargo install cbindgen`
- [clang-format](https://releases.llvm.org/14.0.0/tools/clang/docs/ClangFormat.html), version 14 or higher

Once you have these requirements, the process of regenerating the header file is similar to building and installing from source with CMake, but using different configuration options:

```bash
cmake -S . -B build -DACCESSKIT_BUILD_HEADERS=ON -DACCESSKIT_BUILD_LIBRARIES=OFF
cmake --build build
cmake --install build
```
