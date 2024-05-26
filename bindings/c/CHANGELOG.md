# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.13.3 to 0.14.0
    * accesskit_macos bumped from 0.6.3 to 0.7.0
    * accesskit_unix bumped from 0.3.3 to 0.4.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.4.0 to 0.5.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_macos bumped from 0.7.0 to 0.7.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.14.2 to 0.14.3

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.0 to 0.12.1
    * accesskit_windows bumped from 0.15.0 to 0.15.1
    * accesskit_macos bumped from 0.10.0 to 0.10.1
    * accesskit_unix bumped from 0.6.0 to 0.6.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.6.1 to 0.6.2

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

## [0.10.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.9.0...accesskit_c-v0.10.0) (2024-05-11)


### ⚠ BREAKING CHANGES

* Rename `accesskit_builder_set_text_selection` to `accesskit_node_builder_set_text_selection` ([#407](https://github.com/AccessKit/accesskit/issues/407))

### Bug Fixes

* Explicitly set macOS CI build architecture for C bindings ([#406](https://github.com/AccessKit/accesskit/issues/406)) ([64857e8](https://github.com/AccessKit/accesskit/commit/64857e8747a8eb7b51d321b0f1b98ac76292a8b6))
* Fix dead code warning on Unix platforms ([#403](https://github.com/AccessKit/accesskit/issues/403)) ([09d9157](https://github.com/AccessKit/accesskit/commit/09d91577dd88743e379a1fdea34b25a94726d0fb))
* Rename `accesskit_builder_set_text_selection` to `accesskit_node_builder_set_text_selection` ([#407](https://github.com/AccessKit/accesskit/issues/407)) ([79eb0a4](https://github.com/AccessKit/accesskit/commit/79eb0a4019c4ef7e4b5b7509be68b960ec17b9e0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.18.0 to 0.18.1
    * accesskit_macos bumped from 0.13.0 to 0.13.1
    * accesskit_unix bumped from 0.9.0 to 0.9.1

## [0.9.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.8.0...accesskit_c-v0.9.0) (2024-04-30)


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

## [0.8.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.7.7...accesskit_c-v0.8.0) (2024-04-14)


### ⚠ BREAKING CHANGES

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375))

### Bug Fixes

* Remove unused `BoxCastPtr::set_mut_ptr` in the C bindings ([#378](https://github.com/AccessKit/accesskit/issues/378)) ([3db0fdb](https://github.com/AccessKit/accesskit/commit/3db0fdb6fcd0078c5480fb3a5bf7d5332aad82e5))
* Set INTERFACE_INCLUDE_DIRECTORIES in CMake config ([#379](https://github.com/AccessKit/accesskit/issues/379)) ([2b52701](https://github.com/AccessKit/accesskit/commit/2b52701995816ba45dc9d91e401a6548e77bbb2a))
* Update Corrosion to 0.4.7 ([#380](https://github.com/AccessKit/accesskit/issues/380)) ([0efcfb1](https://github.com/AccessKit/accesskit/commit/0efcfb140cef4c98ccc943f0ab692045d49bbb40))


### Code Refactoring

* New approach to lazy initialization ([#375](https://github.com/AccessKit/accesskit/issues/375)) ([9baebdc](https://github.com/AccessKit/accesskit/commit/9baebdceed7300389b6768815d7ae48f1ce401e4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.3 to 0.13.0
    * accesskit_windows bumped from 0.16.4 to 0.17.0
    * accesskit_macos bumped from 0.11.1 to 0.12.0
    * accesskit_unix bumped from 0.7.5 to 0.8.0

## [0.7.5](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.7.4...accesskit_c-v0.7.5) (2024-03-07)


### Bug Fixes

* Spelling of `transferred` in C bindings documentation ([#361](https://github.com/AccessKit/accesskit/issues/361)) ([f27ce99](https://github.com/AccessKit/accesskit/commit/f27ce99860b02c64367065c6b7dad6e9cbd11f74))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.2 to 0.12.3
    * accesskit_windows bumped from 0.16.2 to 0.16.3
    * accesskit_macos bumped from 0.11.0 to 0.11.1
    * accesskit_unix bumped from 0.7.3 to 0.7.4

## [0.7.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.6.2...accesskit_c-v0.7.0) (2024-01-03)


### ⚠ BREAKING CHANGES

* Lazily activate Unix adapters ([#324](https://github.com/AccessKit/accesskit/issues/324))

### Bug Fixes

* Lazily activate Unix adapters ([#324](https://github.com/AccessKit/accesskit/issues/324)) ([54ed036](https://github.com/AccessKit/accesskit/commit/54ed036c99d87428a8eb5bb03fd77e9e31562d4c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.12.1 to 0.12.2
    * accesskit_windows bumped from 0.15.1 to 0.16.0
    * accesskit_macos bumped from 0.10.1 to 0.11.0
    * accesskit_unix bumped from 0.6.2 to 0.7.0

## [0.6.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.5.1...accesskit_c-v0.6.0) (2023-09-27)


### ⚠ BREAKING CHANGES

* Allow providing app_name, toolkit_name and toolkit_version in Tree, remove parameters from unix adapter constructor ([#291](https://github.com/AccessKit/accesskit/issues/291))
* Make `ActionHandler::do_action` take `&mut self` ([#296](https://github.com/AccessKit/accesskit/issues/296))
* Clean up roles and properties ([#289](https://github.com/AccessKit/accesskit/issues/289))
* Drop next/previous focus properties ([#288](https://github.com/AccessKit/accesskit/issues/288))
* Drop `Tree::root_scroller` ([#279](https://github.com/AccessKit/accesskit/issues/279))
* Decouple in-tree focus from host window/view focus ([#278](https://github.com/AccessKit/accesskit/issues/278))
* Switch to simple unsigned 64-bit integer for node IDs ([#276](https://github.com/AccessKit/accesskit/issues/276))

### Features

* Allow providing app_name, toolkit_name and toolkit_version in Tree, remove parameters from unix adapter constructor ([#291](https://github.com/AccessKit/accesskit/issues/291)) ([5313860](https://github.com/AccessKit/accesskit/commit/531386023257150f49b5e4be942f359855fb7cb6))


### Bug Fixes

* Drop `Tree::root_scroller` ([#279](https://github.com/AccessKit/accesskit/issues/279)) ([fc6c4e0](https://github.com/AccessKit/accesskit/commit/fc6c4e0091d5b257a3869a468fca144a1453cebc))
* Drop next/previous focus properties ([#288](https://github.com/AccessKit/accesskit/issues/288)) ([d35c7c1](https://github.com/AccessKit/accesskit/commit/d35c7c149a650dfedf1b031c0668adad585659fa))


### Code Refactoring

* Clean up roles and properties ([#289](https://github.com/AccessKit/accesskit/issues/289)) ([4fc9c55](https://github.com/AccessKit/accesskit/commit/4fc9c55c91812472593923d93ff89d75ff305ee4))
* Decouple in-tree focus from host window/view focus ([#278](https://github.com/AccessKit/accesskit/issues/278)) ([d360d20](https://github.com/AccessKit/accesskit/commit/d360d20cf951e7643b81a5303006c9f7daa5bd56))
* Make `ActionHandler::do_action` take `&mut self` ([#296](https://github.com/AccessKit/accesskit/issues/296)) ([4fc7846](https://github.com/AccessKit/accesskit/commit/4fc7846d732d61fb45c023060ebab96801a0053e))
* Switch to simple unsigned 64-bit integer for node IDs ([#276](https://github.com/AccessKit/accesskit/issues/276)) ([3eadd48](https://github.com/AccessKit/accesskit/commit/3eadd48ec47854faa94a94ebf910ec08f514642f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.2 to 0.12.0
    * accesskit_windows bumped from 0.14.3 to 0.15.0
    * accesskit_macos bumped from 0.9.0 to 0.10.0
    * accesskit_unix bumped from 0.5.2 to 0.6.0

## [0.5.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.4.0...accesskit_c-v0.5.0) (2023-08-08)


### Features

* Add an SDL example to the C bindings ([#250](https://github.com/AccessKit/accesskit/issues/250)) ([1f5cd1f](https://github.com/AccessKit/accesskit/commit/1f5cd1f7a94a762edeb73188f0ab4fd352c36b3d))
* Workaround for libraries that put the macOS keyboard focus on the window rather than the content view ([#266](https://github.com/AccessKit/accesskit/issues/266)) ([c2db1b0](https://github.com/AccessKit/accesskit/commit/c2db1b0424e905d87691f8148f28b77405f29926))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.1 to 0.11.2
    * accesskit_windows bumped from 0.14.1 to 0.14.2
    * accesskit_macos bumped from 0.8.0 to 0.9.0
    * accesskit_unix bumped from 0.5.1 to 0.5.2

## [0.4.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.3.2...accesskit_c-v0.4.0) (2023-07-30)


### ⚠ BREAKING CHANGES

* New C API for tree updates ([#263](https://github.com/AccessKit/accesskit/issues/263))

### Features

* Add CMake support to C bindings ([#247](https://github.com/AccessKit/accesskit/issues/247)) ([3f556c9](https://github.com/AccessKit/accesskit/commit/3f556c995e8c5eae6831a89b0173809134c1b4e7))
* Add window-based constructor to macOS subclassing adapter ([#253](https://github.com/AccessKit/accesskit/issues/253)) ([022ef04](https://github.com/AccessKit/accesskit/commit/022ef045b9f28262b738ee1ca29a4c7303061fb3))


### Code Refactoring

* New C API for tree updates ([#263](https://github.com/AccessKit/accesskit/issues/263)) ([b260a86](https://github.com/AccessKit/accesskit/commit/b260a860e6f47cf7ef4e10c407123d91c5b35297))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.0 to 0.11.1
    * accesskit_windows bumped from 0.14.0 to 0.14.1
    * accesskit_macos bumped from 0.7.1 to 0.8.0
    * accesskit_unix bumped from 0.5.0 to 0.5.1

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.2.0...accesskit_c-v0.3.0) (2023-04-25)


### ⚠ BREAKING CHANGES

* Remove objc and Core Foundation types from C bindings public API ([#243](https://github.com/AccessKit/accesskit/issues/243))

### Bug Fixes

* Remove objc and Core Foundation types from C bindings public API ([#243](https://github.com/AccessKit/accesskit/issues/243)) ([3ae1c11](https://github.com/AccessKit/accesskit/commit/3ae1c116abcf4593c8540f0d25d154828a69a388))

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_c-v0.1.1...accesskit_c-v0.2.0) (2023-04-01)


### ⚠ BREAKING CHANGES

* Improve C bindings package directory structure ([#239](https://github.com/AccessKit/accesskit/issues/239))

### Bug Fixes

* Improve C bindings package directory structure ([#239](https://github.com/AccessKit/accesskit/issues/239)) ([44c27e7](https://github.com/AccessKit/accesskit/commit/44c27e76f242154a44d907ac4ca0a35bf807caaf))

## 0.1.0 (2023-03-29)


### Features

* Add C bindings ([#230](https://github.com/AccessKit/accesskit/issues/230)) ([7f7f4c7](https://github.com/AccessKit/accesskit/commit/7f7f4c755890ab8210a5a8bf8e237ba6a51dd205))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.1 to 0.11.0
    * accesskit_windows bumped from 0.13.2 to 0.13.3
    * accesskit_macos bumped from 0.6.2 to 0.6.3
    * accesskit_unix bumped from 0.3.2 to 0.3.3
