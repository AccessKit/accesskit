# AccessKit Android adapter

This is the Android adapter for [AccessKit](https://accesskit.dev/).

This adapter is implemented in two layers:

* The `Adapter` struct is the core low-level adapter. It provides maximum flexibility in the application threading model, the interface between Java and native code, and the implementation of action callbacks, at the expense of requiring its caller to provide glue code.
* The `InjectingAdapter` struct injects accessibility into an arbitrary Android view without requiring the view class to be modified, at the expense of depending on a specific Java class and providing less flexibility in the aspects listed above.

The most convenient way to use `InjectingAdapter` is to embed a precompiled `.dex` file containing the associated Java class and its inner classes into the native code. This approach requires the `embedded-dex` Cargo feature.

## Example

The `examples/` directory contains a runnable example built on the low-level `Adapter`:

### Running the example

Install [cargo-ndk](https://github.com/bbqsrc/cargo-ndk) (version 4 or later) and the Rust target for your device, e.g. `rustup target add aarch64-linux-android`. The Android SDK must be discoverable by Gradle (through `ANDROID_HOME` or a `local.properties` file), and cargo-ndk needs an NDK, which it finds inside the SDK or through `ANDROID_NDK_HOME`. Then, with a device connected or an emulator running:

```sh
cd examples/hello_world_app
./gradlew installDebug
```

By default, the Rust library is only built for `arm64-v8a`. Pass `-PrustAbis=arm64-v8a,x86_64` to build for additional ABIs, such as for an emulator on an x86-64 host. The project can also be opened in Android Studio.
