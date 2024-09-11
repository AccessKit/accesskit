# AccessKit

**Accessibility infrastructure for UI toolkits**

[![Build Status](https://github.com/AccessKit/accesskit/actions/workflows/ci.yml/badge.svg)](https://github.com/AccessKit/accesskit/actions)
[![crates.io](https://img.shields.io/crates/v/accesskit)](https://crates.io/crates/accesskit)
[![Documentation](https://docs.rs/accesskit/badge.svg)](https://docs.rs/accesskit/)

AccessKit makes it easier to implement accessibility, for screen readers and other assistive technologies, in toolkits that render their own user interface elements. It provides a cross-platform, cross-language abstraction over accessibility APIs, so toolkit developers only have to implement accessibility once.

## How it works

### Data schema

The heart of AccessKit is a data schema that defines all the data required to render an accessible UI for screen readers and other assistive technologies. The schema represents a tree structure, in which each node is either a single UI element or an element cluster such as a window or document. Each node has an integer ID, a role (e.g. button, label, or text input), and a variety of optional attributes. The schema also defines actions that can be requested by assistive technologies, such as moving the keyboard focus, invoking a button, or selecting text. The schema is based largely on Chromium's cross-platform accessibility abstraction.

The canonical definition of the schema is in the Rust programming language. We chose Rust for its combination of efficiency and an expressive type system. Representations of the schema in other programming languages and schema definition languages (such as JSON Schema or Protocol Buffers) can be generated from the Rust code.

The schema is defined in the [accesskit crate](https://crates.io/crates/accesskit).

#### Serialization

When the toolkit is written in Rust or another language that can efficiently call functions implemented in Rust, such as C or C++, the data defined in the schema can be passed back and forth with no serialization overhead. In other cases, serialization may be used to minimize the overhead of language interoperability. Because the schema supports [serde](https://serde.rs/), each language binding can choose its own serialization format.

### Platform adapters

These are the libraries that implement the platform-specific accessibility APIs.

The interaction between the provider (toolkit or application) and the platform adapter is also inspired by Chromium. Because Chromium has a multi-process architecture and does not allow synchronous IPC from the browser process to the sandboxed renderer processes, the browser process cannot pull accessibility information from the renderer on demand. Instead, the renderer process pushes data to the browser process. The renderer process initially pushes a complete accessibility tree, then it pushes incremental updates. The browser process only needs to send a request to the renderer process when an assistive technology requests one of the actions mentioned above. In AccessKit, the platform adapter is like the Chromium browser process, and the UI toolkit is like the Chromium renderer process, except that both components run in the same process and communicate through normal function calls rather than IPC.

One notable consequence of this design is that only the platform adapter needs to retain a complete accessibility tree in memory. That means that this design is suitable for immediate-mode GUI toolkits, as long as they can provide a stable ID for each UI element.

The platform adapters are written primarily in Rust. We've chosen Rust for its combination of reliability and efficiency, including safe concurrency, which is especially important in modern software. Some future adapters will need to be partially written in another language, such as Java for the Android adapter.

The current released platform adapters are all at rough feature parity. They don't yet support all types of UI elements or all of the properties in the schema, but they have enough functionality to make non-trivial applications accessible, including support for both single-line and multi-line text input controls. They don't yet support rich text or hypertext.

The following platform adapters are currently available:

* [macOS adapter](https://crates.io/crates/accesskit_macos): This adapter implements the NSAccessibility protocols in the AppKit framework.
* [Unix adapter](https://crates.io/crates/accesskit_unix): This adapter implements the AT-SPI D-Bus interfaces, using [zbus](https://github.com/dbus2/zbus), a pure-Rust implementation of D-Bus.
* [Windows adapter](https://crates.io/crates/accesskit_windows): This adapter implements UI Automation, the current Windows accessibility API.

#### Planned adapters

* Android
* iOS
* web (for applications that render their own UI elements to a canvas)

### Adapters for cross-platform windowing layers

#### winit

The [AccessKit winit adapter](https://crates.io/crates/accesskit_winit) is the recommended way to use AccessKit with [winit](https://crates.io/crates/winit), the popular Rust windowing library.

### Language bindings

UI toolkit developers who merely want to use AccessKit should not be required to use Rust directly. In addition to a direct Rust API, AccessKit provides the following language bindings, covering both the data schema and the platform adapters:

* [C bindings](https://github.com/AccessKit/accesskit-c): The C API is implemented directly in Rust, and the header file is generated using [cbindgen](https://crates.io/crates/cbindgen). We provide both a Windows-specific example using the Win32 API directly and a cross-platform example using SDL.
* [Python bindings](https://pypi.org/project/accesskit/): These bindings are implemented in Rust using [PyO3](https://pyo3.rs/). We provide a cross-platform example using Pygame.

### Consumer library

Some of the code required by the platform adapters is platform-independent. This code is in [the AccessKit Consumer Crate](https://crates.io/crates/accesskit_consumer). In addition to platform adapters, this library may also be useful for implementing embedded assistive technologies, such as a screen reader running directly inside an application, for devices that don't have platform support for accessibility at all, such as game consoles and appliances.

### Documentation

We realize that most developers who might use AccessKit are not experts in accessibility. So this project will need to include comprehensive documentation, including a conceptual overview for developers that are learning about accessibility for the first time.

## Contributing

Contributions to AccessKit are welcome. Please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

AccessKit is licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in AccessKit by you, as defined in the Apache License, shall be dual-licensed as above, without any additional terms or conditions.

The list of authors for copyright purposes is in the [AUTHORS](AUTHORS) file.

Significant portions of AccessKit are derived from Chromium and are covered by its [BSD-style license](LICENSE.chromium).
