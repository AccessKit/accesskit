# AccessKit

## UI accessibility infrastructure across platforms and programming languages

[![Build Status](https://github.com/AccessKit/accesskit/actions/workflows/ci.yml/badge.svg)](https://github.com/AccessKit/accesskit/actions)
[![crates.io](https://img.shields.io/crates/v/accesskit)](https://crates.io/crates/accesskit)
[![docs.rs](https://docs.rs/accesskit/badge.svg)](https://docs.rs/accesskit/)

## Motivation

There are numerous UI toolkits, and new ones continue to proliferate. So far, only the largest UI toolkit projects, with corporate backing, implement accessibility for users who require assistive technologies, such as blind people using screen readers. If the long tail of UI toolkits are ever going to be made fully accessible, then we must pool as much of the required effort as possible across these toolkits. Many of these toolkits are cross-platform, but each platform has its own accessibility API. These toolkits are also written in many different programming languages, so the shared infrastructure must be usable across languages.

## Project components

### Data schema

It has often been said that data structures are more important than code. This is certainly the case with AccessKit. The heart of AccessKit is a schema that defines all of the data required to make a UI accessible to screen readers and other assistive technologies. This schema represents a tree structure, in which each node is either a UI element such as a button or text box, or a grouping of elements such as a document, pane, or window. Each node has an integer ID, a role (e.g. button or window), and a variety of optional attributes. The schema also defines actions that can be requested by assistive technologies, such as moving the keyboard focus, invoking a button, or selecting text. The schema is based largely on Chromium's cross-platform accessibility abstraction.

The canonical definition of the schema is in the Rust programming language. Rust was chosen for its combination of efficiency and an expressive type system.

When both the toolkit and the platform adapter (see below) are written in Rust or another language that can efficiently call functions defined in Rust (such as C or C++), the data defined in the schema can be passed back and forth with no serialization overhead. In other cases, serialization will be required to minimize the overhead of language interoperability. Because the schema supports [serde](https://serde.rs/), each language binding of this type can choose its own serialization format.

The schema is defined in [the `accesskit` crate](https://crates.io/crates/accesskit), in [the `common` directory](https://github.com/AccessKit/accesskit/tree/main/common). It has not yet been stabilized, but it is reasonably complete. At this point there is just one known issue: the schema doesn't yet define any events. While some events in platform accessibility APIs, such as focus change and property changes, can be implied from tree updates, other events, such as ad-hoc announcements meant for screen reader users, cannot.

### Platform adapters

These are the libraries that implement platform accessibility APIs. The following platform adapters are currently implemented:

* The Windows adapter, which implements the UI Automation API, is available in [the `accesskit_windows` crate](https://crates.io/crates/accesskit_windows), in [the `platforms/windows` directory](https://github.com/AccessKit/accesskit/tree/main/platforms/windows). It doesn't yet support all possible widget types, but it can now be used to make real, non-trivial applications accessible. In particular, it supports both single-line and multi-line text edit controls, but not yet rich text.
* The macOS adapter, which implements the Cocoa NSAccessibility protocol, is available in [the `accesskit_macos` crate](https://crates.io/crates/accesskit_macos), in [the `platforms/macos` directory](https://github.com/AccessKit/accesskit/tree/main/platforms/macos). It is roughly at feature parity with the Windows adapter, including support for text edit controls.
* The Unix adapter, which implements the D-Bus-based AT-SPI protocol, is available in [the `accesskit_unix` crate](https://crates.io/crates/accesskit_unix), in [the `platforms/unix` directory](https://github.com/AccessKit/accesskit/tree/main/platforms/unix). This adapter also supports text edit controls. It is not yet fully usable with the Orca screen reader, due to a keyboard input handling issue that we are working with the appropriate GNOME development teams to solve.

The following adapters are planned:

* Android
* iOS/tvOS
* Web (creating a hidden HTML DOM)

The interaction between the provider (toolkit or application) and the platform adapter is also inspired by Chromium. Because Chromium has a multi-process architecture and does not allow synchronous IPC from the browser process to the sandboxed renderer processes, the browser process cannot pull accessibility information from the renderer on demand. Instead, the renderer process pushes data to the browser process. The renderer process initially pushes a complete accessibility tree, then it pushes incremental updates. The browser process only needs to send a request to the renderer process when an assistive technology requests one of the actions mentioned above. In AccessKit, the platform adapter is like the Chromium browser process, and the UI toolkit is like the Chromium renderer process, except that both components run in the same process and communicate through normal function calls rather than IPC.

One notable consequence of this design is that only the platform adapter needs to retain a complete accessibility tree in memory. That means that this design is suitable for immediate-mode GUI toolkits, as long as they can provide a stable ID for each UI element.

The platform adapters are written primarily in Rust. We've chosen Rust for its combination of reliability and efficiency, including safe concurrency, which is especially important in modern software. Some future adapters may need to be partially written in another language, such as Java or Kotlin for the Android adapter.

### Consumer library

Some of the code required by the platform adapters is platform-independent. This code is in [the `accesskit_consumer` crate](https://crates.io/crates/accesskit_consumer), in [the `consumer` directory](https://github.com/AccessKit/accesskit/tree/main/consumer). In addition to platform adapters, this library may also be useful for implementing embedded assistive technologies, such as a screen reader running directly inside an application, for platforms that don't yet have an AccessKit platform adapter, or for devices that don't have platform support for accessibility at all, such as game consoles and appliances.

### Adapters for cross-platform windowing layers

In the Rust ecosystem, [the `winit` crate](https://crates.io/crates/winit) is a popular cross-platform abstraction for windowing and user input. [The `accesskit_winit` crate](https://crates.io/crates/accesskit_winit), in [the `platforms/winit` directory](https://github.com/AccessKit/accesskit/tree/main/platforms/winit), provides a cross-platform way of integrating AccessKit into `winit`-based toolkits and applications. AccessKit is also directly integrated into the similar [glazier](https://github.com/linebender/glazier) crate. We may later implement similar adapters for other cross-platform abstractions such as GLFW and SDL.

### GUI toolkit integrations

While we expect GUI toolkit developers to eventually integrate AccessKit into their own projects, we are directly working on integration with some GUI toolkits at this stage in the project, so we can test our work on AccessKit in realistic environments. So far, we have integrated AccessKit into [the `egui` toolkit for Rust](https://github.com/emilk/egui). We also have a [proof-of-concept integration with the Unity game engine](https://github.com/AccessKit/the-intercept), which demonstrates the ability to expose AccessKit to a language other than Rust.

### Language bindings

UI toolkit developers who merely want to use AccessKit should not be required to use Rust directly.

AccessKit provides a C API covering both the core data structures and all platform adapters. This C API can be used from a variety of languages. The Rust source for the C bindings is in the [accesskit-c](https://github.com/AccessKit/accesskit-c) repository. The AccessKit project also provides a pre-built package, including a header file, both dynamic and static libraries, and sample code, for the C API, so toolkit developers won't need to deal with Rust at all. The latest pre-built package can be found in [accesskit-c GitHub releases](https://github.com/AccessKit/accesskit-c/releases).

Bindings for the Python programming language are also available. Rust source code is in the [accesskit-python](https://github.com/AccessKit/accesskit-python) repository. Releases can be found on [PyPI](https://pypi.org/project/accesskit/) and can be included in your project using `pip`.

While many languages can use a C API, we also plan to provide libraries that make it easier to safely use AccessKit from languages other than Rust and C. In particular, we're planning to provide such a library for Java and other JVM-based languages.

### Documentation

We realize that most developers who might use AccessKit are not experts in accessibility. So this project will need to include comprehensive documentation, including a conceptual overview for developers that are learning about accessibility for the first time.

## Contributing

Contributions to AccessKit are welcome. Please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

AccessKit is licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in AccessKit by you, as defined in the Apache License, shall be dual-licensed as above, without any additional terms or conditions.

The list of authors for copyright purposes is in the [AUTHORS](AUTHORS) file.

Significant portions of AccessKit are derived from Chromium and are covered by its [BSD-style license](LICENSE.chromium).
