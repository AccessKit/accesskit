# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.12.0 to 0.12.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.0 to 0.10.1
    * accesskit_consumer bumped from 0.14.0 to 0.14.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.1 to 0.11.0
    * accesskit_consumer bumped from 0.14.1 to 0.14.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.0 to 0.11.1
    * accesskit_consumer bumped from 0.15.0 to 0.15.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.1 to 0.11.2
    * accesskit_consumer bumped from 0.15.1 to 0.15.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.0 to 0.12.1
    * accesskit_consumer bumped from 0.16.0 to 0.16.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_atspi_common bumped from 0.1.0 to 0.1.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.2 to 0.12.3
    * accesskit_atspi_common bumped from 0.1.1 to 0.1.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit_atspi_common bumped from 0.1.2 to 0.2.0

## [0.7.2](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.7.1...accesskit_unix-v0.7.2) (2024-02-24)


### Bug Fixes

* Don't emit focus event twice on Unix ([#354](https://github.com/AccessKit/accesskit/issues/354)) ([b39216c](https://github.com/AccessKit/accesskit/commit/b39216cb31df692fef377f9b3c3c718fd225cc3c))
* Use the new accesskit_atspi_common crate in the Unix adapter ([#356](https://github.com/AccessKit/accesskit/issues/356)) ([b2a468c](https://github.com/AccessKit/accesskit/commit/b2a468ccb91ee4e6d3435e73eb00c65cbe75060a))

## [0.7.1](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.7.0...accesskit_unix-v0.7.1) (2024-01-11)


### Bug Fixes

* Make full use of tokio ecosystem if the tokio feature is enabled on Unix ([#336](https://github.com/AccessKit/accesskit/issues/336)) ([c034802](https://github.com/AccessKit/accesskit/commit/c0348024665a615a30fd8fe2f02e8c93cf9c6332))
* Run our own async executor on Unix ([#337](https://github.com/AccessKit/accesskit/issues/337)) ([8f937ba](https://github.com/AccessKit/accesskit/commit/8f937baaa510dd96da196501822b82f75f05b595))

## [0.7.0](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.6.2...accesskit_unix-v0.7.0) (2024-01-03)


### ⚠ BREAKING CHANGES

* Lazily activate Unix adapters ([#324](https://github.com/AccessKit/accesskit/issues/324))

### Features

* Support custom role descriptions ([#316](https://github.com/AccessKit/accesskit/issues/316)) ([c8d1a56](https://github.com/AccessKit/accesskit/commit/c8d1a5638fa6c33adfa059815c04f7e043c56026))


### Bug Fixes

* Lazily activate Unix adapters ([#324](https://github.com/AccessKit/accesskit/issues/324)) ([54ed036](https://github.com/AccessKit/accesskit/commit/54ed036c99d87428a8eb5bb03fd77e9e31562d4c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.1 to 0.12.2
    * accesskit_consumer bumped from 0.16.1 to 0.17.0

## [0.6.2](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.6.1...accesskit_unix-v0.6.2) (2023-12-14)


### Bug Fixes

* Bump async-channel dependency to `2.1.1` ([#321](https://github.com/AccessKit/accesskit/issues/321)) ([99120b8](https://github.com/AccessKit/accesskit/commit/99120b828d65306ab71d41f71979dc67e8b0bf6b))

## [0.6.0](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.5.2...accesskit_unix-v0.6.0) (2023-09-27)


### ⚠ BREAKING CHANGES

* Allow providing app_name, toolkit_name and toolkit_version in Tree, remove parameters from unix adapter constructor ([#291](https://github.com/AccessKit/accesskit/issues/291))
* Make `ActionHandler::do_action` take `&mut self` ([#296](https://github.com/AccessKit/accesskit/issues/296))
* Clean up roles and properties ([#289](https://github.com/AccessKit/accesskit/issues/289))
* Decouple in-tree focus from host window/view focus ([#278](https://github.com/AccessKit/accesskit/issues/278))
* Switch to simple unsigned 64-bit integer for node IDs ([#276](https://github.com/AccessKit/accesskit/issues/276))

### Features

* Add role for terminals ([#282](https://github.com/AccessKit/accesskit/issues/282)) ([ddbef37](https://github.com/AccessKit/accesskit/commit/ddbef37158b57f56217317b480e40d58f83a9c24))
* Allow providing app_name, toolkit_name and toolkit_version in Tree, remove parameters from unix adapter constructor ([#291](https://github.com/AccessKit/accesskit/issues/291)) ([5313860](https://github.com/AccessKit/accesskit/commit/531386023257150f49b5e4be942f359855fb7cb6))
* Support live regions on Unix ([#299](https://github.com/AccessKit/accesskit/issues/299)) ([8d52a5f](https://github.com/AccessKit/accesskit/commit/8d52a5fc4271a3b5edcc602b23fd7b920446eab0))
* Support multiple top-level windows on Unix ([#292](https://github.com/AccessKit/accesskit/issues/292)) ([43ecf4b](https://github.com/AccessKit/accesskit/commit/43ecf4b3ab96d9e8f7d2c2222c7e664c4f4f4abf))


### Bug Fixes

* Don't require tokio rt-multi-thread feature ([#290](https://github.com/AccessKit/accesskit/issues/290)) ([cf61e47](https://github.com/AccessKit/accesskit/commit/cf61e477adff26b032fa0b24502c0ae0a96c1987))
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

## [0.5.0](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.4.0...accesskit_unix-v0.5.0) (2023-05-21)


### Features

* Add features for async runtimes on Unix ([#248](https://github.com/AccessKit/accesskit/issues/248)) ([b56b4ea](https://github.com/AccessKit/accesskit/commit/b56b4ea7c967ee5a1dae21a2fa0dcd385346031e))

## [0.4.0](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.3.3...accesskit_unix-v0.4.0) (2023-03-30)


### ⚠ BREAKING CHANGES

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234))

### Bug Fixes

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234)) ([773389b](https://github.com/AccessKit/accesskit/commit/773389bff857fa18edf15de426e029251fc34591))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.14.2 to 0.15.0

## [0.3.1](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.3.0...accesskit_unix-v0.3.1) (2023-02-20)


### Bug Fixes

* Update atspi dependency ([#217](https://github.com/AccessKit/accesskit/issues/217)) ([93f2dc9](https://github.com/AccessKit/accesskit/commit/93f2dc9bf0a57a8b7592c3a4cf4aa3885a3356f2))

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.2.0...accesskit_unix-v0.3.0) (2023-02-12)


### ⚠ BREAKING CHANGES

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212))

### Code Refactoring

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212)) ([5df52e5](https://github.com/AccessKit/accesskit/commit/5df52e5545faddf6a51905409013c2f5be23981e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.9.0 to 0.10.0
    * accesskit_consumer bumped from 0.13.0 to 0.14.0

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_unix-v0.1.1...accesskit_unix-v0.2.0) (2023-02-05)


### ⚠ BREAKING CHANGES

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205))

### Code Refactoring

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205)) ([4811152](https://github.com/AccessKit/accesskit/commit/48111521439b76c1a8687418a4b20f9b705eac6d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.1 to 0.9.0
    * accesskit_consumer bumped from 0.12.1 to 0.13.0

## 0.1.0 (2023-01-05)


### Features

* Basic Unix platform adapter ([#198](https://github.com/AccessKit/accesskit/issues/198)) ([1cea32e](https://github.com/AccessKit/accesskit/commit/1cea32e44ee743b778ac941ceff9087ae745cb37))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.11.0 to 0.12.0
