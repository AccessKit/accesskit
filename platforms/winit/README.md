# AccessKit winit adapter

This is the winit adapter for [AccessKit](https://accesskit.dev/). It exposes an AccessKit accessibility tree through the platform-native accessibility API on any platform supported by AccessKit. On platforms not supported by AccessKit, this adapter does nothing, but still compiles.

## Compatibility with async runtimes

The following only applies on Linux/Unix:

While this crate's API is purely blocking, it internally spawns asynchronous tasks on an executor.

- If you use tokio, make sure to enable the `tokio` feature of this crate.
- If you use another async runtime or if you don't use one at all, the default feature will suit your needs.

## Android activity compatibility

The Android implementation of this adapter currently assumes that the Android activity Java class stores its content view in an instance variable called `mSurfaceView`. This is the case for [GameActivity](https://developer.android.com/games/agdk/game-activity), which is one of the two activity implementations that winit currently supports.
