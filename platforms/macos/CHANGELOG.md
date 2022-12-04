# Changelog

## [0.2.1](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.2.0...accesskit_macos-v0.2.1) (2022-12-04)


### Bug Fixes

* Correctly apply the DPI scale factor to coordinates ([#185](https://github.com/AccessKit/accesskit/issues/185)) ([d263938](https://github.com/AccessKit/accesskit/commit/d263938d68bb63567853a340d3466ff27e076d87))
* Expose static text as the value rather than the title on macOS ([#186](https://github.com/AccessKit/accesskit/issues/186)) ([e3720c8](https://github.com/AccessKit/accesskit/commit/e3720c8e2d7c5e8c8601c52ad620dcfcacebc570))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.0 to 0.8.1
    * accesskit_consumer bumped from 0.9.1 to 0.10.0

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.5...accesskit_macos-v0.2.0) (2022-11-29)


### ⚠ BREAKING CHANGES

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179))

### Code Refactoring

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179)) ([f35c941](https://github.com/AccessKit/accesskit/commit/f35c941f395f3162db376a69cfaaaf770d376267))

## [0.1.5](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.4...accesskit_macos-v0.1.5) (2022-11-27)


### Bug Fixes

* Handle views with flipped coordinates ([#174](https://github.com/AccessKit/accesskit/issues/174)) ([d14484c](https://github.com/AccessKit/accesskit/commit/d14484cdcfdd99a497354aa3e012a0e130cc3d64))
* Make VoiceOver move through nodes in logical order ([#176](https://github.com/AccessKit/accesskit/issues/176)) ([f060be4](https://github.com/AccessKit/accesskit/commit/f060be409945296ed100cd63ecb3d2bb6bbad89e))

### [0.1.4](https://www.github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.3...accesskit_macos-v0.1.4) (2022-11-25)


### Bug Fixes

* Re-export types from objc2 ([#172](https://www.github.com/AccessKit/accesskit/issues/172)) ([1ac67ad](https://www.github.com/AccessKit/accesskit/commit/1ac67ad17587d79b5338cb71e2bc07612fc10c44))

### [0.1.3](https://www.github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.2...accesskit_macos-v0.1.3) (2022-11-25)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.9.0 to 0.9.1

### [0.1.2](https://www.github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.1...accesskit_macos-v0.1.2) (2022-11-24)


### Bug Fixes

* **platforms/macos:** Add the macOS crate to the release-please configuration ([#164](https://www.github.com/AccessKit/accesskit/issues/164)) ([da83f63](https://www.github.com/AccessKit/accesskit/commit/da83f63d279a10c5a7199a9145ca9eb9e27d7b56))
