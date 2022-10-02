# AccessKit

This is the shared cross-platform crate for [AccessKit](https://accesskit.dev/). It defines the data structures that represent an accessibility tree, and the trait for handling action requests from assistive technologies.

To use AccessKit in your application or toolkit, you will also need a platform adapter. The following platform adapters are currently available:

* [accesskit_windows](https://crates.io/crates/accesskit_windows): exposes an AccessKit tree on Windows using the UI Automation API
* [accesskit_winit](https://crates.io/crates/accesskit_winit): wraps other platform adapters for use with the [winit](https://crates.io/crates/winit) windowing library

All platform adapters include simple examples.
