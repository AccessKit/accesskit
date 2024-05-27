# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.2 to 0.12.3
    * accesskit_consumer bumped from 0.17.0 to 0.17.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.19.0 to 0.19.1

## [0.6.0](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.5.0...accesskit_atspi_common-v0.6.0) (2024-05-27)


### Features

* Expose the `orientation` property ([#421](https://github.com/AccessKit/accesskit/issues/421)) ([590aada](https://github.com/AccessKit/accesskit/commit/590aada070dc812f9b8f171fb9e43ac984fad2a1))


### Bug Fixes

* Fix a logic error in suffix calculation for text changes ([#423](https://github.com/AccessKit/accesskit/issues/423)) ([1121723](https://github.com/AccessKit/accesskit/commit/1121723983cb2fa64b5053626ec64afb851ff6c4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.21.0 to 0.22.0

## [0.5.0](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.4.2...accesskit_atspi_common-v0.5.0) (2024-05-26)


### Features

* Add basic text support on Unix ([#362](https://github.com/AccessKit/accesskit/issues/362)) ([52540f8](https://github.com/AccessKit/accesskit/commit/52540f82cf9fc148358351ed486bab3e7e91f1d6))
* Expose the `placeholder` property ([#417](https://github.com/AccessKit/accesskit/issues/417)) ([8f4a0a1](https://github.com/AccessKit/accesskit/commit/8f4a0a1c10f83fcc8580a37d8013fec2d110865b))


### Bug Fixes

* Don't fire events for filtered children on Unix ([#414](https://github.com/AccessKit/accesskit/issues/414)) ([2bcb1b6](https://github.com/AccessKit/accesskit/commit/2bcb1b63e88b801b194a4db50059fa063efbee64))
* Improve how coordinates are computed on Unix ([#420](https://github.com/AccessKit/accesskit/issues/420)) ([fc5125e](https://github.com/AccessKit/accesskit/commit/fc5125e27f8f4f655e1de5049d0d53536284d9a0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.20.0 to 0.21.0

## [0.4.2](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.4.1...accesskit_atspi_common-v0.4.2) (2024-05-13)


### Bug Fixes

* Fix platform adapters to support copy-on-write tree snapshots again ([#411](https://github.com/AccessKit/accesskit/issues/411)) ([d3a130a](https://github.com/AccessKit/accesskit/commit/d3a130a5ec8ae1d9edf0bf85a44f35f0e365242c))
* Return to handling focus events directly, after generic node changes ([#409](https://github.com/AccessKit/accesskit/issues/409)) ([cd2e35e](https://github.com/AccessKit/accesskit/commit/cd2e35e43817405199ae6acd64ef90aee445be0b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.19.1 to 0.20.0

## [0.4.0](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.3.0...accesskit_atspi_common-v0.4.0) (2024-04-30)


### ⚠ BREAKING CHANGES

* Clean up table roles and properties ([#393](https://github.com/AccessKit/accesskit/issues/393))
* Rename `Checked` to `Toggled`; drop `ToggleButton` role ([#388](https://github.com/AccessKit/accesskit/issues/388))

### Features

* Implement the `description` property ([#382](https://github.com/AccessKit/accesskit/issues/382)) ([d49f406](https://github.com/AccessKit/accesskit/commit/d49f40660b5dc23ed074cd72a91e511b130756ae))


### Bug Fixes

* Increase minimum supported Rust version to `1.70` ([#396](https://github.com/AccessKit/accesskit/issues/396)) ([a8398b8](https://github.com/AccessKit/accesskit/commit/a8398b847aa003de91042ac45e33126fc2cae053))


### Code Refactoring

* Clean up table roles and properties ([#393](https://github.com/AccessKit/accesskit/issues/393)) ([e34dad9](https://github.com/AccessKit/accesskit/commit/e34dad94448a5321ece9def3f2e054aa5f62dd79))
* Rename `Checked` to `Toggled`; drop `ToggleButton` role ([#388](https://github.com/AccessKit/accesskit/issues/388)) ([6bc040b](https://github.com/AccessKit/accesskit/commit/6bc040b7cf75cdbd6a019cc380d8dbce804b3c81))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.13.0 to 0.14.0
    * accesskit_consumer bumped from 0.18.0 to 0.19.0

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.2.0...accesskit_atspi_common-v0.3.0) (2024-04-14)


### ⚠ BREAKING CHANGES

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375))

### Code Refactoring

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375)) ([9baebdc](https://github.com/AccessKit/accesskit/commit/9baebdceed7300389b6768815d7ae48f1ce401e4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.3 to 0.13.0
    * accesskit_consumer bumped from 0.17.1 to 0.18.0

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.1.2...accesskit_atspi_common-v0.2.0) (2024-03-11)


### Features

* Expose root node ID in `accesskit_atspi_common::Adapter` ([#370](https://github.com/AccessKit/accesskit/issues/370)) ([a43b497](https://github.com/AccessKit/accesskit/commit/a43b497afbbbcf90e9d15259635a329164d6a791))

## [0.1.1](https://github.com/AccessKit/accesskit/compare/accesskit_atspi_common-v0.1.0...accesskit_atspi_common-v0.1.1) (2024-02-24)


### Bug Fixes

* Add missing README ([#357](https://github.com/AccessKit/accesskit/issues/357)) ([e8cf48e](https://github.com/AccessKit/accesskit/commit/e8cf48e21be0146768b2d14289164d192823fd1f))

## 0.1.0 (2024-02-24)


### Features

* Factor out core AT-SPI translation layer ([#352](https://github.com/AccessKit/accesskit/issues/352)) ([8c0ab58](https://github.com/AccessKit/accesskit/commit/8c0ab58d441c0d4484e0bc31a554bdfb3f088cd6))
