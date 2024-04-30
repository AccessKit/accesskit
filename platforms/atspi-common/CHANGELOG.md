# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.2 to 0.12.3
    * accesskit_consumer bumped from 0.17.0 to 0.17.1

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
