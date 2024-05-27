# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.11.0 to 0.12.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.12.0 to 0.12.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.0 to 0.10.1
    * accesskit_consumer bumped from 0.14.0 to 0.14.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.0 to 0.12.1
    * accesskit_consumer bumped from 0.16.0 to 0.16.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.2 to 0.12.3
    * accesskit_consumer bumped from 0.17.0 to 0.17.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.19.0 to 0.19.1

## [0.15.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.14.0...accesskit_macos-v0.15.0) (2024-05-27)


### Features

* Expose the `orientation` property ([#421](https://github.com/AccessKit/accesskit/issues/421)) ([590aada](https://github.com/AccessKit/accesskit/commit/590aada070dc812f9b8f171fb9e43ac984fad2a1))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.21.0 to 0.22.0

## [0.14.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.13.2...accesskit_macos-v0.14.0) (2024-05-26)


### Features

* Expose the `placeholder` property ([#417](https://github.com/AccessKit/accesskit/issues/417)) ([8f4a0a1](https://github.com/AccessKit/accesskit/commit/8f4a0a1c10f83fcc8580a37d8013fec2d110865b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.20.0 to 0.21.0

## [0.13.2](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.13.1...accesskit_macos-v0.13.2) (2024-05-13)


### Bug Fixes

* Fix platform adapters to support copy-on-write tree snapshots again ([#411](https://github.com/AccessKit/accesskit/issues/411)) ([d3a130a](https://github.com/AccessKit/accesskit/commit/d3a130a5ec8ae1d9edf0bf85a44f35f0e365242c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.19.1 to 0.20.0

## [0.13.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.12.0...accesskit_macos-v0.13.0) (2024-04-30)


### ⚠ BREAKING CHANGES

* Clean up table roles and properties ([#393](https://github.com/AccessKit/accesskit/issues/393))
* Drop `NodeClassSet` ([#389](https://github.com/AccessKit/accesskit/issues/389))
* Rename `Checked` to `Toggled`; drop `ToggleButton` role ([#388](https://github.com/AccessKit/accesskit/issues/388))

### Features

* Implement the `description` property ([#382](https://github.com/AccessKit/accesskit/issues/382)) ([d49f406](https://github.com/AccessKit/accesskit/commit/d49f40660b5dc23ed074cd72a91e511b130756ae))


### Bug Fixes

* Increase minimum supported Rust version to `1.70` ([#396](https://github.com/AccessKit/accesskit/issues/396)) ([a8398b8](https://github.com/AccessKit/accesskit/commit/a8398b847aa003de91042ac45e33126fc2cae053))
* Use new objc2 crates ([#384](https://github.com/AccessKit/accesskit/issues/384)) ([b3484c0](https://github.com/AccessKit/accesskit/commit/b3484c0fb1fef3ecd41ff9592978336c20b8b4f8))


### Code Refactoring

* Clean up table roles and properties ([#393](https://github.com/AccessKit/accesskit/issues/393)) ([e34dad9](https://github.com/AccessKit/accesskit/commit/e34dad94448a5321ece9def3f2e054aa5f62dd79))
* Drop `NodeClassSet` ([#389](https://github.com/AccessKit/accesskit/issues/389)) ([1b153ed](https://github.com/AccessKit/accesskit/commit/1b153ed51f8421cdba2dc98beca2e8f5f8c781bc))
* Rename `Checked` to `Toggled`; drop `ToggleButton` role ([#388](https://github.com/AccessKit/accesskit/issues/388)) ([6bc040b](https://github.com/AccessKit/accesskit/commit/6bc040b7cf75cdbd6a019cc380d8dbce804b3c81))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.13.0 to 0.14.0
    * accesskit_consumer bumped from 0.18.0 to 0.19.0

## [0.12.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.11.1...accesskit_macos-v0.12.0) (2024-04-14)


### ⚠ BREAKING CHANGES

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375))

### Bug Fixes

* Fix new compiler warning in Rust 1.77 ([#376](https://github.com/AccessKit/accesskit/issues/376)) ([1de7c63](https://github.com/AccessKit/accesskit/commit/1de7c63e7db12bc7eda57a191e913fef0e572f43))


### Code Refactoring

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375)) ([9baebdc](https://github.com/AccessKit/accesskit/commit/9baebdceed7300389b6768815d7ae48f1ce401e4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.3 to 0.13.0
    * accesskit_consumer bumped from 0.17.1 to 0.18.0

## [0.11.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.10.1...accesskit_macos-v0.11.0) (2024-01-03)


### Features

* Support custom role descriptions ([#316](https://github.com/AccessKit/accesskit/issues/316)) ([c8d1a56](https://github.com/AccessKit/accesskit/commit/c8d1a5638fa6c33adfa059815c04f7e043c56026))


### Bug Fixes

* Bump objc2 to 0.5.0; bring icrate 0.1.0 ([#323](https://github.com/AccessKit/accesskit/issues/323)) ([23b3f2f](https://github.com/AccessKit/accesskit/commit/23b3f2f93b9452c80374d1da3e9abeaec60ba9bf))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.1 to 0.12.2
    * accesskit_consumer bumped from 0.16.1 to 0.17.0

## [0.10.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.9.0...accesskit_macos-v0.10.0) (2023-09-27)


### ⚠ BREAKING CHANGES

* Make `ActionHandler::do_action` take `&mut self` ([#296](https://github.com/AccessKit/accesskit/issues/296))
* Clean up roles and properties ([#289](https://github.com/AccessKit/accesskit/issues/289))
* Decouple in-tree focus from host window/view focus ([#278](https://github.com/AccessKit/accesskit/issues/278))
* Switch to simple unsigned 64-bit integer for node IDs ([#276](https://github.com/AccessKit/accesskit/issues/276))

### Features

* Add role for terminals ([#282](https://github.com/AccessKit/accesskit/issues/282)) ([ddbef37](https://github.com/AccessKit/accesskit/commit/ddbef37158b57f56217317b480e40d58f83a9c24))


### Bug Fixes

* Support text fields without a value property ([#274](https://github.com/AccessKit/accesskit/issues/274)) ([5ae557b](https://github.com/AccessKit/accesskit/commit/5ae557b40d395b4a9966a90a2d80e7d97ad50bf9))
* Use common filters across platform adapters ([#287](https://github.com/AccessKit/accesskit/issues/287)) ([09c1204](https://github.com/AccessKit/accesskit/commit/09c12045ff4ccdb22f0cf643077a27465013572d))


### Code Refactoring

* Clean up roles and properties ([#289](https://github.com/AccessKit/accesskit/issues/289)) ([4fc9c55](https://github.com/AccessKit/accesskit/commit/4fc9c55c91812472593923d93ff89d75ff305ee4))
* Decouple in-tree focus from host window/view focus ([#278](https://github.com/AccessKit/accesskit/issues/278)) ([d360d20](https://github.com/AccessKit/accesskit/commit/d360d20cf951e7643b81a5303006c9f7daa5bd56))
* Make `ActionHandler::do_action` take `&mut self` ([#296](https://github.com/AccessKit/accesskit/issues/296)) ([4fc7846](https://github.com/AccessKit/accesskit/commit/4fc7846d732d61fb45c023060ebab96801a0053e))
* Switch to simple unsigned 64-bit integer for node IDs ([#276](https://github.com/AccessKit/accesskit/issues/276)) ([3eadd48](https://github.com/AccessKit/accesskit/commit/3eadd48ec47854faa94a94ebf910ec08f514642f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.2 to 0.12.0
    * accesskit_consumer bumped from 0.15.2 to 0.16.0

## [0.9.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.8.0...accesskit_macos-v0.9.0) (2023-08-08)


### Features

* Workaround for libraries that put the macOS keyboard focus on the window rather than the content view ([#266](https://github.com/AccessKit/accesskit/issues/266)) ([c2db1b0](https://github.com/AccessKit/accesskit/commit/c2db1b0424e905d87691f8148f28b77405f29926))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.1 to 0.11.2
    * accesskit_consumer bumped from 0.15.1 to 0.15.2

## [0.8.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.7.1...accesskit_macos-v0.8.0) (2023-07-30)


### Features

* Add window-based constructor to macOS subclassing adapter ([#253](https://github.com/AccessKit/accesskit/issues/253)) ([022ef04](https://github.com/AccessKit/accesskit/commit/022ef045b9f28262b738ee1ca29a4c7303061fb3))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.0 to 0.11.1
    * accesskit_consumer bumped from 0.15.0 to 0.15.1

## [0.7.1](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.7.0...accesskit_macos-v0.7.1) (2023-06-20)


### Bug Fixes

* Set proper target to build accesskit_macos documentation ([#226](https://github.com/AccessKit/accesskit/issues/226)) ([9cd6bb1](https://github.com/AccessKit/accesskit/commit/9cd6bb14d60bf85027b330a51afe912c37723902))

## [0.7.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.6.3...accesskit_macos-v0.7.0) (2023-03-30)


### ⚠ BREAKING CHANGES

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234))

### Bug Fixes

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234)) ([773389b](https://github.com/AccessKit/accesskit/commit/773389bff857fa18edf15de426e029251fc34591))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.14.2 to 0.15.0

## [0.6.3](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.6.2...accesskit_macos-v0.6.3) (2023-03-29)


### Bug Fixes

* Fix problems related to the root node ([#231](https://github.com/AccessKit/accesskit/issues/231)) ([7228494](https://github.com/AccessKit/accesskit/commit/7228494361c4f131af6a7fc2af8a98406cd9a63e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.1 to 0.11.0
    * accesskit_consumer bumped from 0.14.1 to 0.14.2

## [0.6.2](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.6.1...accesskit_macos-v0.6.2) (2023-03-14)


### Bug Fixes

* Fix macOS leaks ([e8537fc](https://github.com/AccessKit/accesskit/commit/e8537fcbdf4a68f39c9bc51cf9fe6960903e26f2))

## [0.6.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.5.0...accesskit_macos-v0.6.0) (2023-02-12)


### ⚠ BREAKING CHANGES

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212))

### Code Refactoring

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212)) ([5df52e5](https://github.com/AccessKit/accesskit/commit/5df52e5545faddf6a51905409013c2f5be23981e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.9.0 to 0.10.0
    * accesskit_consumer bumped from 0.13.0 to 0.14.0

## [0.5.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.4.2...accesskit_macos-v0.5.0) (2023-02-05)


### ⚠ BREAKING CHANGES

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205))

### Code Refactoring

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205)) ([4811152](https://github.com/AccessKit/accesskit/commit/48111521439b76c1a8687418a4b20f9b705eac6d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.1 to 0.9.0
    * accesskit_consumer bumped from 0.12.1 to 0.13.0

## [0.4.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.3.0...accesskit_macos-v0.4.0) (2022-12-27)


### Features

* Live regions on macOS ([#196](https://github.com/AccessKit/accesskit/issues/196)) ([47d8d9f](https://github.com/AccessKit/accesskit/commit/47d8d9f6a567dfe909aa4065886cace07084efb7))


### Bug Fixes

* Pin objc2 dependency to 0.3.0-beta.3 ([#201](https://github.com/AccessKit/accesskit/issues/201)) ([0adfed1](https://github.com/AccessKit/accesskit/commit/0adfed1192ee255fba34ad82e8483ab9296ac2df))

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.2.1...accesskit_macos-v0.3.0) (2022-12-17)


### Features

* Text support on macOS ([#191](https://github.com/AccessKit/accesskit/issues/191)) ([3a35dbe](https://github.com/AccessKit/accesskit/commit/3a35dbe02122c789fe682995c5b7e022aef5cc36))


### Bug Fixes

* Don't expose the window title in our root element on macOS ([#187](https://github.com/AccessKit/accesskit/issues/187)) ([9739b74](https://github.com/AccessKit/accesskit/commit/9739b7424328da45c1c43b6db49af142a8789aa5))
* Expose which accessibility selectors are actually allowed for a particular node ([#181](https://github.com/AccessKit/accesskit/issues/181)) ([c4cbb23](https://github.com/AccessKit/accesskit/commit/c4cbb23156749d513df4e520dcb9be0a74c697d3))
* More reliable handling of the edge case for wrapped lines ([#192](https://github.com/AccessKit/accesskit/issues/192)) ([c626d2c](https://github.com/AccessKit/accesskit/commit/c626d2c3028085b076ada7dd31242cf3ca3c0f08))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.10.0 to 0.11.0

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
