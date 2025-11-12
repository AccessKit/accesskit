# AccessKit

**Accessibility infrastructure for UI toolkits**

[![Build Status](https://github.com/AccessKit/accesskit/actions/workflows/ci.yml/badge.svg)](https://github.com/AccessKit/accesskit/actions)
[![crates.io](https://img.shields.io/crates/v/accesskit)](https://crates.io/crates/accesskit)
[![Documentation](https://docs.rs/accesskit/badge.svg)](https://docs.rs/accesskit/)

AccessKit makes it easier to implement accessibility, for screen readers and other assistive technologies, in toolkits that render their own user interface elements. It provides a cross-platform, cross-language abstraction over accessibility APIs, so toolkit developers only have to implement accessibility once.

## Documentation

* [How AccessKit works (architecture + Quick Start)](docs/how-it-works.md): canonical walkthrough of the schema, adapters, and a runnable winit example.
* [Tips for application developers](README-APPLICATION-DEVELOPERS.md): practical screen reader testing guidance.
* [Crate documentation on docs.rs](https://docs.rs/accesskit/): API reference for each published crate.

## Platform adapters

AccessKit ships adapters for the major desktop platforms plus Android, all written primarily in Rust:

* [Android adapter](https://crates.io/crates/accesskit_android): Implements the Java-based Android accessibility API.
* [macOS adapter](https://crates.io/crates/accesskit_macos): Bridges to AppKit's `NSAccessibility`.
* [Unix adapter](https://crates.io/crates/accesskit_unix): Talks to the AT-SPI D-Bus interfaces through [`zbus`](https://github.com/dbus2/zbus).
* [Windows adapter](https://crates.io/crates/accesskit_windows): Implements UI Automation.

Planned adapters include iOS and a web/canvas bridge. Each adapter retains the full accessibility tree while toolkits keep only the state they need.

| Platform | Adapter crate | Support status | Notes |
| --- | --- | --- | --- |
| Windows 10+ | `accesskit_windows` | Stable | Exposes UI Automation. |
| macOS 12+ | `accesskit_macos` | Stable | Known issue: ListBox item selection state not reported yet (`platforms/macos/README.md:5-7`). |
| Linux / *BSD (AT-SPI) | `accesskit_unix` | Stable | Requires choosing an async runtime feature (`platforms/unix/README.md:3-10`). |
| Android (GameActivity) | `accesskit_android` | Stable | `InjectingAdapter` currently depends on GameActivity (`platforms/winit/README.md:14-16`). |
| iOS | _Planned_ | – | Adapter not yet implemented. |
| Web/canvas | _Planned_ | – | Adapter not yet implemented. |

### Adapters for cross-platform windowing layers

* [AccessKit winit adapter](https://crates.io/crates/accesskit_winit): Recommended entry point for Rust applications built on [winit](https://crates.io/crates/winit). It wraps the platform-specific adapters and exposes a single API surface for delivering tree updates and handling action requests.

### Language bindings

UI toolkit developers who don't want to call Rust directly can use:

* [C bindings](https://github.com/AccessKit/accesskit-c): C header generated with [`cbindgen`](https://crates.io/crates/cbindgen) plus Win32 and SDL samples.
* [Python bindings](https://pypi.org/project/accesskit/): Implemented via [PyO3](https://pyo3.rs/) with a cross-platform Pygame sample.

### Consumer library

Shared logic that every adapter needs lives in [the AccessKit Consumer Crate](https://crates.io/crates/accesskit_consumer). It can also power embedded assistive technologies, such as a screen reader that runs directly inside a game engine or appliance that lacks platform-level accessibility APIs.

## Contributing

Contributions to AccessKit are welcome. Please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

AccessKit is licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in AccessKit by you, as defined in the Apache License, shall be dual-licensed as above, without any additional terms or conditions.

The list of authors for copyright purposes is in the [AUTHORS](AUTHORS) file.

Significant portions of AccessKit are derived from Chromium and are covered by its [BSD-style license](LICENSE.chromium).
