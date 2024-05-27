# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.7.0 to 0.7.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.16.0 to 0.16.1
    * accesskit_unix bumped from 0.7.1 to 0.7.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.7.2 to 0.7.3

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.16.1 to 0.16.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.2 to 0.12.3
    * accesskit_windows bumped from 0.16.2 to 0.16.3
    * accesskit_macos bumped from 0.11.0 to 0.11.1
    * accesskit_unix bumped from 0.7.3 to 0.7.4

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.7.4 to 0.7.5

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.16.3 to 0.16.4

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.18.1 to 0.18.2
    * accesskit_macos bumped from 0.13.1 to 0.13.2
    * accesskit_unix bumped from 0.9.1 to 0.9.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.18.2 to 0.19.0
    * accesskit_macos bumped from 0.13.2 to 0.14.0
    * accesskit_unix bumped from 0.9.2 to 0.10.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.19.0 to 0.20.0
    * accesskit_macos bumped from 0.14.0 to 0.15.0
    * accesskit_unix bumped from 0.10.0 to 0.10.1

## [0.3.1](https://github.com/AccessKit/accesskit/compare/accesskit_python-v0.3.0...accesskit_python-v0.3.1) (2024-05-11)


### Bug Fixes

* Fix dead code warning on Unix platforms ([#403](https://github.com/AccessKit/accesskit/issues/403)) ([09d9157](https://github.com/AccessKit/accesskit/commit/09d91577dd88743e379a1fdea34b25a94726d0fb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.18.0 to 0.18.1
    * accesskit_macos bumped from 0.13.0 to 0.13.1
    * accesskit_unix bumped from 0.9.0 to 0.9.1

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_python-v0.2.0...accesskit_python-v0.3.0) (2024-04-30)


### ⚠ BREAKING CHANGES

* Clean up table roles and properties ([#393](https://github.com/AccessKit/accesskit/issues/393))
* Rename `hierarchical_level` to `level` ([#390](https://github.com/AccessKit/accesskit/issues/390))
* Drop `NodeClassSet` ([#389](https://github.com/AccessKit/accesskit/issues/389))
* Rename `Checked` to `Toggled`; drop `ToggleButton` role ([#388](https://github.com/AccessKit/accesskit/issues/388))

### Features

* Add the `owns` relation ([#392](https://github.com/AccessKit/accesskit/issues/392)) ([fd668dd](https://github.com/AccessKit/accesskit/commit/fd668ddc4b64cb05ab3600972b3d3823a037f2d5))


### Bug Fixes

* Increase minimum supported Rust version to `1.70` ([#396](https://github.com/AccessKit/accesskit/issues/396)) ([a8398b8](https://github.com/AccessKit/accesskit/commit/a8398b847aa003de91042ac45e33126fc2cae053))


### Code Refactoring

* Clean up table roles and properties ([#393](https://github.com/AccessKit/accesskit/issues/393)) ([e34dad9](https://github.com/AccessKit/accesskit/commit/e34dad94448a5321ece9def3f2e054aa5f62dd79))
* Drop `NodeClassSet` ([#389](https://github.com/AccessKit/accesskit/issues/389)) ([1b153ed](https://github.com/AccessKit/accesskit/commit/1b153ed51f8421cdba2dc98beca2e8f5f8c781bc))
* Rename `Checked` to `Toggled`; drop `ToggleButton` role ([#388](https://github.com/AccessKit/accesskit/issues/388)) ([6bc040b](https://github.com/AccessKit/accesskit/commit/6bc040b7cf75cdbd6a019cc380d8dbce804b3c81))
* Rename `hierarchical_level` to `level` ([#390](https://github.com/AccessKit/accesskit/issues/390)) ([2d61e01](https://github.com/AccessKit/accesskit/commit/2d61e01fffff1265b348c141715f6f9b6fe4081b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.13.0 to 0.14.0
    * accesskit_windows bumped from 0.17.0 to 0.18.0
    * accesskit_macos bumped from 0.12.0 to 0.13.0
    * accesskit_unix bumped from 0.8.0 to 0.9.0

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_python-v0.1.8...accesskit_python-v0.2.0) (2024-04-14)


### ⚠ BREAKING CHANGES

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375))

### Code Refactoring

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375)) ([9baebdc](https://github.com/AccessKit/accesskit/commit/9baebdceed7300389b6768815d7ae48f1ce401e4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.3 to 0.13.0
    * accesskit_windows bumped from 0.16.4 to 0.17.0
    * accesskit_macos bumped from 0.11.1 to 0.12.0
    * accesskit_unix bumped from 0.7.5 to 0.8.0

## [0.1.1](https://github.com/AccessKit/accesskit/compare/accesskit_python-v0.1.0...accesskit_python-v0.1.1) (2024-01-06)


### Bug Fixes

* Decrease minimum Python version to 3.8 for Python bindings ([#334](https://github.com/AccessKit/accesskit/issues/334)) ([3725373](https://github.com/AccessKit/accesskit/commit/3725373658bf2475cf3e1341b2e5fcefada576bd))

## 0.1.0 (2024-01-03)


### Features

* Add Python bindings ([#269](https://github.com/AccessKit/accesskit/issues/269)) ([52560da](https://github.com/AccessKit/accesskit/commit/52560da1c1480f1a37a27906b24b518a5fa03249))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.1 to 0.12.2
    * accesskit_windows bumped from 0.15.1 to 0.16.0
    * accesskit_macos bumped from 0.10.1 to 0.11.0
    * accesskit_unix bumped from 0.6.2 to 0.7.0
