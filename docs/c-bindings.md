# AccessKit from C and other FFI Consumers

The [accesskit-c](https://github.com/AccessKit/accesskit-c) repository packages AccessKit as a C library so toolkits written in C, C++, or other FFI-friendly languages can expose accessibility trees without writing Rust.

## Getting the binaries

Download the latest release archive from GitHub and unpack it somewhere CMake or Meson can find it. Each release contains:

- `include/accesskit.h`: the generated header file.
- Prebuilt static and dynamic libraries for Windows, macOS, Linux, and Android (ARM64 + x86\_64 variants).
- CMake package metadata (`ACCESSKITConfig.cmake`) plus Meson wraps to simplify integration.

## Using the CMake package

Point CMake at the extracted directory (via `CMAKE_PREFIX_PATH` or the `ACCESSKIT_DIR` cache entry) and add:

```cmake
find_package(ACCESSKIT REQUIRED)
target_link_libraries(your_app PRIVATE accesskit)
```

The [examples directory](https://github.com/AccessKit/accesskit-c/tree/main/examples) contains working Win32 and SDL samples that create trees, forward events, and send updates to the platform adapters.

## Building from source

If the prebuilt binaries do not cover your platform, build the library yourself:

1. Install Rust (via [rustup.rs](https://rustup.rs/)) and CMake ≥ 3.20.
2. In the `accesskit-c` checkout, run:

   ```bash
   cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
   cmake --build build
   cmake --install build
   ```

3. For cross-compiling, pass `-A` on Windows or specify both `CMAKE_SYSTEM_*` and `Rust_CARGO_TARGET` on other systems (see the README in the C repo).

If you modify the bindings, regenerate `accesskit.h` with `cbindgen` (nightly Rust `rustup install nightly-2025-03-27` and `cargo install cbindgen`) and commit the updated header.

## High-level usage

At run time your UI toolkit is responsible for:

1. Including `accesskit.h` and initializing the chosen platform adapter (for example, `accesskit_windows_adapter_new`).
2. Implementing the `accesskit_action_handler` callbacks so AccessKit can tell you when a screen reader invokes a control.
3. Building `accesskit_tree_update` values whenever your UI changes and passing them to the adapter.

The SDL sample in the C repository shows how to wire these steps into an event loop that closely mirrors the Rust winit adapter.
