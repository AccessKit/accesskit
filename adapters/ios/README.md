# AccessKit iOS adapter

This is the iOS adapter for [AccessKit](https://accesskit.dev/). It exposes an AccessKit accessibility tree through the UIKit `UIAccessibility` protocol.

## Example

The `examples/` directory contains one runnable example:

- `showcase` — a UIKit application that owns its `UIView` subclass and drives the low-level `Adapter` directly.

Install [XcodeGen](https://github.com/yonaskolb/XcodeGen) and the iOS Rust targets, then run `xcodegen` from `examples/apple/` to generate the Xcode project. Open it in Xcode and build/run the `Showcase` target on a device or simulator.

## Acknowledgements

This project was funded through the [NGI0 Commons Fund](https://nlnet.nl/commonsfund), a fund established by [NLnet](https://nlnet.nl/). See the [project page](https://nlnet.nl/project/AccessKit-iOS/) for more information.
