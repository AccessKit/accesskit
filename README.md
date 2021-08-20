# AccessKit: UI accessibility infrastructure across platforms and programming languages

[![Build Status](https://github.com/AccessKit/accesskit/actions/workflows/ci.yml/badge.svg)](https://github.com/AccessKit/accesskit/actions)

## Motivation

There are numerous UI toolkits, and new ones continue to proliferate. So far, only the largest UI toolkit projects, with corporate backing, implement accessibility for users who require assistive technologies, such as blind people using screen readers. If the long tail of UI toolkits are ever going to be made fully accessible, then we must pool as much of the required effort as possible across these toolkits. Many of these toolkits are cross-platform, but each platform has its own accessibility API. These toolkits are also written in many different programming languages, so the shared infrastructure must be usable across languages.

## Project plan

AccessKit is currently in the design phase. The plan is to break the project down into the following components:

### Data schema

It has often been said that data structures are more important than code. This is certainly the case with AccessKit. The heart of AccessKit is a schema that defines all of the data required to make a UI accessible to screen readers and other assistive technologies. This schema represents a tree structure, in which each node is either a UI element such as a button or text box, or a grouping of elements such as a document, pane, or window. Each node has an integer ID, a role (e.g. button or window), and a variety of optional attributes. The schema also defines actions that can be requested by assistive technologies, such as moving the keyboard focus, invoking a button, or selecting text. The schema is based largely on Chromium's cross-platform accessibility abstraction.

The canonical definition of the schema is in the Rust programming language. Rust was chosen for its combination of efficiency and an expressive type system. Representations of the schema in other programming languages and schema definition languages (such as JSON Schema or Protocol Buffers) can be generated from the Rust code.

When both the toolkit and the platform adapter (see below) are written in Rust or another language that can efficiently access Rust data structures (such as C++), the data defined in the schema can be passed back and forth with no serialization overhead. In other cases, serialization will be required to minimize the overhead of language interoperability. The plan is to use JSON serialization for now, and possibly add other serialization options later, for better performance or to please toolkit developers who would rather not use JSON.

A draft of the schema is defined in [the `schema` directory](https://github.com/AccessKit/accesskit/tree/main/schema). It has the following known limitations:

* While a list of action types is defined, the `ActionRequest` struct that will represent a request from an assistive technology to perform an action, with parameters for some types of actions, has not yet been defined.
* he schema doesn't yet define any events. While some events in platform accessibility APIs, such as focus change and property changes, can be implied from tree updates, others cannot.
* The in-memory representation of a node has not yet been optimized.

### Platform adapters

These are the libraries that implement platform accessibility APIs. The following platform adapters are planned:

* Windows (UI Automation API)
* macOS
* iOS/tvOS
* Unix desktop environments (e.g. GNOME) based on AT-SPI
* Android
* Web (creating a hidden HTML DOM)

Most of these libraries will certainly be written in Rust. The best course of action for the Android and web libraries isn't yet clear. The original plan was to write these in Kotlin and TypeScript, respectively. However, that would prevent us from reusing the consumer library (described below) for these platform adapters. So it may be best to write these in Rust as well.

The interaction between the provider (toolkit or application) and the platform adapter is also inspired by Chromium. Because Chromium has a multi-process architecture and does not allow synchronous IPC from the browser process to the sandboxed renderer processes, the browser process cannot pull accessibility information from the renderer on demand. Instead, the renderer process pushes data to the browser process. The renderer process initially pushes a complete accessibility tree, then it pushes incremental updates. The browser process only needs to send a request to the renderer process when an assistive technology requests one of the actions mentioned above. In AccessKit, the platform adapter will be like the Chromium browser process, and the UI toolkit will be like the Chromium renderer process, except that both components will run in the same process and will communicate through normal function calls rather than IPC.

One notable consequence of this design is that only the platform adapter needs to retain a complete accessibility tree in memory. That means that this design is suitable for immediate-mode GUI toolkits, as long as they can provide a stable ID for each UI element.

As mentioned above, many (if not all) of the platform adapters will be written in Rust. We've chosen Rust for its combination of reliability and efficiency, including safe concurrency, which is especially important in modern software.

A prototype platform adapter for macOS is in [the `platforms/mac` directory](https://github.com/AccessKit/accesskit/tree/main/platforms/mac). So far, it only implements a small subset of the translation from the AccessKit schema to the macOS accessibility API -- just enough to traverse the nodes of a simple tree and get their names and roles. One especially glaring limitation of the current implementation is that the Cocoa object for a node is not cleaned up if the node is removed from the AccessKit tree. It would probably be simplest to take care of this while implementing events, since one of those events will be the removal of a node.

### Consumer library

Some of the code required by the platform adapters is platform-independent. This code is in what we currently call the consumer library, in [the `consumer` directory](https://github.com/AccessKit/accesskit/tree/main/consumer).

The consumer library has these known limitations:

* `Tree::update` needs to raise events when nodes are added, removed, or updated. In the last case, an event should only be raised if the node was actually updated.
* The `Tree::update` function doesn't handle the schema's `TreeUpdate::clear` field. It's not clear whether this field will actually be needed in any anticipated use case of AccessKit; the field was simply copied from Chromium. If we don't need it, we should drop it.
* The `Node` struct needs traversal functions that skip ignored children, as in Chromium's `AXNode` class.
* `AXNode::bounds` needs to account for the container offset and/or transform.
* `Node` needs many more functions for getting computed data. For example, the position of an item in the list and the number of items in the list are often computed, not specified directly. The same is true for some table attributes. As is often the case, we'll follow Chromium's lead here unless we have a reason to do otherwise.
* This library needs tests.

### Language bindings

UI toolkit developers who merely want to use AccessKit should not be required to use Rust directly. In addition to a direct Rust API, the Rust-based platform adapters will also provide a C API, which can be used from a variety of languages. The AccessKit project will provide pre-built binaries, including both dynamic and static libraries, for these platform adapters using the C API, so toolkit developers won't need to deal with Rust at all.

While many languages can use a C API, we also plan to provide libraries that make it easier to use AccessKit from languages other than Rust and C. In particular, we're planning to provide such a library for Java and other JVM-based languages. A prototype of such a library for the macOS platform adapter is in [the `platforms/mac/jni` directory](https://github.com/AccessKit/accesskit/tree/main/platforms/mac/jni). This library should probably be refactored into platform-independent and platform-specific parts.

### Documentation

We realize that most developers who might use AccessKit are not experts in accessibility. So this project will need to include comprehensive documentation, including a conceptual overview for developers that are learning about accessibility for the first time.
