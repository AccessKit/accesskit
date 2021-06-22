# AccessKit: UI accessibility infrastructure across platforms and programming languages

## Motivation

There are numerous UI toolkits, and new ones continue to proliferate. So far, only the largest UI toolkit projects, with corporate backing, implement accessibility for users who require assistive technologies, such as blind people using screen readers. If the long tail of UI toolkits are ever going to be made fully accessible, then we must pool as much of the required effort as possible across these toolkits. Many of these toolkits are cross-platform, but each platform has its own accessibility API. These toolkits are also written in many different programming languages, so the shared infrastructure must be usable across languages. This is the problem that AccessKit aims to solve.

## Project plan

AccessKit is currently in the design phase. The plan is to break the project down into the following components:

### Data schema

It has often been said that data structures are more important than code. This is certainly the case with AccessKit. The heart of AccessKit will be a schema that defines all of the data required to make a UI accessible to screen readers and other assistive technologies. This schema will represent a tree structure, in which each node is either a UI element such as a button or text box, or a grouping of elements such as a document, pane, or window. Each node will have an integer ID, a role (e.g. button or window), and a variety of optional attributes. The schema will also define actions that can be requested by assistive technologies, such as moving the keyboard focus, invoking a button, or selecting text. The schema will be based largely on Chromium's cross-platform accessibility abstraction.

The schema will be defined using the Protocol Buffers schema language (a .proto file). This schema definition language was chosen because Protocol Buffers is broadly supported across many programming languages. In some cases, where the UI toolkit and the platform adapter (see below) are written in different programming languages, the data will actually be serialized using the Protocol Buffers binary encoding. This will minimize the overhead of cross-language interfaces such as the Java Native Interface (JNI). However, serialization will not always be needed. For example, some of the platform adapters will be written in Rust. When the UI toolkit is also written in Rust, data can be passed back and forth in standard Rust structs, with no serialization step. We also expect to be able to eliminate the serialization step for other language combinations. Finally, we plan to offer JSON as an alternate serialization option.

### Platform adapters

These are the libraries that implement platform accessibility APIs. The following platform adapters are planned:

* Windows (UI Automation API), written in Rust
* macOS, written in Rust
* iOS/tvOS, written in Rust
* Unix desktop environments (e.g. GNOME) based on AT-SPI, written in Rust
* Android, written in Java or Kotlin
* Web (creating a hidden HTML DOM), written in TypeScript, likely using a virtual DOM library such as Preact

The interaction between the toolkit and the platform adapter is also inspired by Chromium. Because Chromium has a multi-process architecture and does not allow synchronous IPC from the browser process to the sandboxed renderer processes, the browser process cannot pull accessibility information from the renderer on demand. Instead, the renderer process pushes data to the browser process. The renderer process initially pushes a complete accessibility tree, then it pushes incremental updates. The browser process only needs to send a request to the renderer process when an assistive technology requests one of the actions mentioned above. In AccessKit, the platform adapter will be like the Chromium browser process, and the UI toolkit will be like the Chromium renderer process, except that both components will run in the same process and will communicate through normal function calls rather than IPC.

One notable consequence of this design is that only the platform adapter needs to retain a complete accessibility tree in memory. That means that this design is suitable for immediate-mode GUI toolkits, as long as they can provide a stable ID for each UI element.

As mentioned above, many of the platform adapters will be written in Rust. We've chosen Rust for its combination of reliability and efficiency, including safe concurrency, which is especially important in modern software. However, this doesn't mean that UI toolkit developers who merely want to use AccessKit will need to work directly with Rust. In addition to a direct Rust API, the Rust-based platform adapters will also provide a C API, which can be used from a variety of languages. The AccessKit project will provide pre-built binaries, including both dynamic and static libraries, for these platform adapters using the C API, so toolkit developers won't need to deal with Rust at all.

### Language support libraries

While many languages can use a C API, we also plan to provide libraries that make it easier to use AccessKit from languages other than Rust and C. In particular, we're planning to provide such a library for Java and other JVM-based languages.

### Documentation

We realize that most developers who might use AccessKit are not experts in accessibility. So this project will need to include comprehensive documentation, including a conceptual overview for developers that are learning about accessibility for the first time.

## Who is behind this project?

The founder of the AccessKit project is Matt Campbell, a legally blind software developer who has specialized in accessibility for over 20 years. Matt developed the System Access screen reader for Windows starting in 2004. He was a former member of the Windows accessibility team at Microsoft, where he worked on the Narrator screen reader and the UI Automation API. He is now the chief technology officer of [Pneuma Solutions](https://pneumasolutions.com/).
