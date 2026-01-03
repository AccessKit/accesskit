# Changelog

## [0.5.0](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.4.2...accesskit_android-v0.5.0) (2025-12-23)


### ⚠ BREAKING CHANGES

* Drop deprecated roles ([#642](https://github.com/AccessKit/accesskit/issues/642))
* Infrastructure for supporting text formatting changes ([#626](https://github.com/AccessKit/accesskit/issues/626))

### Features

* Add GridCell role ([#643](https://github.com/AccessKit/accesskit/issues/643)) ([1e5abca](https://github.com/AccessKit/accesskit/commit/1e5abca737d1ee942c0804fec2c06d3cb08faa94))
* Implement BrailleLabel and BrailleRoleDescription roles ([#638](https://github.com/AccessKit/accesskit/issues/638)) ([0fdcebb](https://github.com/AccessKit/accesskit/commit/0fdcebb55e308e039ec99fbc31e94e8087a69f2d))


### Code Refactoring

* Drop deprecated roles ([#642](https://github.com/AccessKit/accesskit/issues/642)) ([4d46c27](https://github.com/AccessKit/accesskit/commit/4d46c2740631c5fe4f057707b949d12b26931d0b))
* Infrastructure for supporting text formatting changes ([#626](https://github.com/AccessKit/accesskit/issues/626)) ([ea23ec4](https://github.com/AccessKit/accesskit/commit/ea23ec424c7dbb8841e03d71b6a15b74264850a9))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.21.1 to 0.22.0
    * accesskit_consumer bumped from 0.31.0 to 0.32.0

## [0.4.2](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.4.1...accesskit_android-v0.4.2) (2025-10-20)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.30.1 to 0.31.0

## [0.4.1](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.4.0...accesskit_android-v0.4.1) (2025-10-02)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.21.0 to 0.21.1
    * accesskit_consumer bumped from 0.30.0 to 0.30.1

## [0.4.0](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.3.0...accesskit_android-v0.4.0) (2025-07-16)


### Features

* Let parents declare actions supported on their children ([#593](https://github.com/AccessKit/accesskit/issues/593)) ([70b534b](https://github.com/AccessKit/accesskit/commit/70b534bed168a84b84cc35199588aa8ab784fb43))
* Scrolling on Android ([#586](https://github.com/AccessKit/accesskit/issues/586)) ([62f193a](https://github.com/AccessKit/accesskit/commit/62f193a50e6c11c6726629410548244e486ce940))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.20.0 to 0.21.0
    * accesskit_consumer bumped from 0.29.0 to 0.30.0

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.2.0...accesskit_android-v0.3.0) (2025-06-26)


### ⚠ BREAKING CHANGES

* Force a semver-breaking release ([#589](https://github.com/AccessKit/accesskit/issues/589))

### Bug Fixes

* Force a semver-breaking release ([#589](https://github.com/AccessKit/accesskit/issues/589)) ([2887cdd](https://github.com/AccessKit/accesskit/commit/2887cddde817ba3851688068d8d10de5cef7c624))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.19.0 to 0.20.0
    * accesskit_consumer bumped from 0.28.0 to 0.29.0

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.1.1...accesskit_android-v0.2.0) (2025-05-06)


### ⚠ BREAKING CHANGES

* Simplify the core Android adapter API ([#558](https://github.com/AccessKit/accesskit/issues/558))
* Use the queued-events pattern in the Android adapter ([#555](https://github.com/AccessKit/accesskit/issues/555))
* Drop redundant `HasPopup::True` ([#550](https://github.com/AccessKit/accesskit/issues/550))

### Bug Fixes

* Fix Android adapter after dropping `FrozenNode` ([#553](https://github.com/AccessKit/accesskit/issues/553)) ([735cb7e](https://github.com/AccessKit/accesskit/commit/735cb7e292b87e7660586a924954689e4894dcea))
* Return text content from multiline inputs ([#552](https://github.com/AccessKit/accesskit/issues/552)) ([4b74090](https://github.com/AccessKit/accesskit/commit/4b74090dc0b848747296b4a66d3bbe3cef96fc56))


### Code Refactoring

* Drop redundant `HasPopup::True` ([#550](https://github.com/AccessKit/accesskit/issues/550)) ([56abf17](https://github.com/AccessKit/accesskit/commit/56abf17356e4c7f13f64aaeaca6a63c8f7ede553))
* Simplify the core Android adapter API ([#558](https://github.com/AccessKit/accesskit/issues/558)) ([7ac5911](https://github.com/AccessKit/accesskit/commit/7ac5911b11f3d6b8b777b91e6476e7073f6b0e4a))
* Use the queued-events pattern in the Android adapter ([#555](https://github.com/AccessKit/accesskit/issues/555)) ([0316518](https://github.com/AccessKit/accesskit/commit/0316518b94cf1bc9755e67f0cf48e37c096975fa))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.18.0 to 0.19.0
    * accesskit_consumer bumped from 0.27.0 to 0.28.0

## [0.1.1](https://github.com/AccessKit/accesskit/compare/accesskit_android-v0.1.0...accesskit_android-v0.1.1) (2025-03-17)


### Bug Fixes

* Eliminate the dependency on `paste` ([#528](https://github.com/AccessKit/accesskit/issues/528)) ([4aef05d](https://github.com/AccessKit/accesskit/commit/4aef05d0b34b434c0f0ce2e7583adef3e73bda4d))

## 0.1.0 (2025-03-06)


### Features

* Android adapter ([#500](https://github.com/AccessKit/accesskit/issues/500)) ([7e65ac7](https://github.com/AccessKit/accesskit/commit/7e65ac77d7e108ac5b9f3722f488a2fdf2e3b3e0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.17.1 to 0.18.0
    * accesskit_consumer bumped from 0.26.0 to 0.27.0
