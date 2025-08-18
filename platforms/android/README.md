# AccessKit Android adapter

This is the Android adapter for [AccessKit](https://accesskit.dev/).

This adapter is implemented in two layers:

* The `Adapter` struct is the core low-level adapter. It provides maximum flexibility in the application threading model, the interface between Java and native code, and the implementation of action callbacks, at the expense of requiring its caller to provide glue code.
* The `InjectingAdapter` struct injects accessibility into an arbitrary Android view without requiring the view class to be modified, at the expense of depending on a specific Java class and providing less flexibility in the aspects listed above.

The most convenient way to use `InjectingAdapter` is to embed a `.dex` file containing the associated Java class and its inner classes into the native code. This approach requires the `embedded-dex` Cargo feature.
