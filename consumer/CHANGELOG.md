# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.0 to 0.10.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.1 to 0.11.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.0 to 0.11.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.1 to 0.11.2

## [0.15.0](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.14.2...accesskit_consumer-v0.15.0) (2023-03-30)


### ⚠ BREAKING CHANGES

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234))

### Bug Fixes

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234)) ([773389b](https://github.com/AccessKit/accesskit/commit/773389bff857fa18edf15de426e029251fc34591))

## [0.14.0](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.13.0...accesskit_consumer-v0.14.0) (2023-02-12)


### ⚠ BREAKING CHANGES

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212))

### Code Refactoring

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212)) ([5df52e5](https://github.com/AccessKit/accesskit/commit/5df52e5545faddf6a51905409013c2f5be23981e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.9.0 to 0.10.0

## [0.13.0](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.12.1...accesskit_consumer-v0.13.0) (2023-02-05)


### ⚠ BREAKING CHANGES

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205))

### Code Refactoring

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205)) ([4811152](https://github.com/AccessKit/accesskit/commit/48111521439b76c1a8687418a4b20f9b705eac6d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.1 to 0.9.0

## [0.12.1](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.12.0...accesskit_consumer-v0.12.1) (2023-01-06)


### Bug Fixes

* Make `Node::filtered_parent` recursive as it was meant to be ([#203](https://github.com/AccessKit/accesskit/issues/203)) ([d2faef5](https://github.com/AccessKit/accesskit/commit/d2faef5a2ad61b9e4d3f3d5c89570cdeec6fe6e6))

## [0.12.0](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.11.0...accesskit_consumer-v0.12.0) (2023-01-05)


### Features

* Basic Unix platform adapter ([#198](https://github.com/AccessKit/accesskit/issues/198)) ([1cea32e](https://github.com/AccessKit/accesskit/commit/1cea32e44ee743b778ac941ceff9087ae745cb37))

## [0.11.0](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.10.0...accesskit_consumer-v0.11.0) (2022-12-17)


### Features

* Text support on macOS ([#191](https://github.com/AccessKit/accesskit/issues/191)) ([3a35dbe](https://github.com/AccessKit/accesskit/commit/3a35dbe02122c789fe682995c5b7e022aef5cc36))


### Bug Fixes

* More reliable handling of the edge case for wrapped lines ([#192](https://github.com/AccessKit/accesskit/issues/192)) ([c626d2c](https://github.com/AccessKit/accesskit/commit/c626d2c3028085b076ada7dd31242cf3ca3c0f08))

## [0.10.0](https://github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.9.1...accesskit_consumer-v0.10.0) (2022-12-04)


### Features

* Automatically get button and link labels from descendants ([#184](https://github.com/AccessKit/accesskit/issues/184)) ([ec5c38e](https://github.com/AccessKit/accesskit/commit/ec5c38ef3001a10b7a135df1438901246463f3e1))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.0 to 0.8.1

### [0.9.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.9.0...accesskit_consumer-v0.9.1) (2022-11-25)


### Bug Fixes

* **consumer:** Allow editable spin buttons ([#167](https://www.github.com/AccessKit/accesskit/issues/167)) ([65a7aa0](https://www.github.com/AccessKit/accesskit/commit/65a7aa0114bfc6e17189e834578e256945b84a98))
* Gracefully handle nodes that only support text ranges some of the time ([#169](https://www.github.com/AccessKit/accesskit/issues/169)) ([1f50df6](https://www.github.com/AccessKit/accesskit/commit/1f50df6820b9d23fe2e579f043f4981acf285de2))

## [0.9.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.8.0...accesskit_consumer-v0.9.0) (2022-11-23)


### Features

* **platforms/macos:** Basic macOS platform adapter ([#158](https://www.github.com/AccessKit/accesskit/issues/158)) ([a06725e](https://www.github.com/AccessKit/accesskit/commit/a06725e952e6041dbd366944fa793b746c9f195e))

## [0.8.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.7.1...accesskit_consumer-v0.8.0) (2022-11-17)


### ⚠ BREAKING CHANGES

* **consumer:** Eliminate the dependency on `im` due to licensing (#153)

### Code Refactoring

* **consumer:** Eliminate the dependency on `im` due to licensing ([#153](https://www.github.com/AccessKit/accesskit/issues/153)) ([b4c4cb5](https://www.github.com/AccessKit/accesskit/commit/b4c4cb5713d4833d8ee7979e4f4e39c7e96a3ed4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.7.0 to 0.8.0

### [0.7.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.7.0...accesskit_consumer-v0.7.1) (2022-11-12)


### Bug Fixes

* **consumer, platforms/windows, platforms/winit:** Update to parking_lot 0.12.1 ([#146](https://www.github.com/AccessKit/accesskit/issues/146)) ([6772855](https://www.github.com/AccessKit/accesskit/commit/6772855a7b540fd728faad15d8d208b05c1bbd8a))

## [0.7.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.6.1...accesskit_consumer-v0.7.0) (2022-11-11)


### ⚠ BREAKING CHANGES

* Text range support (#145)
* Drop the `ignored` field and implement generic filtered tree traversal (#143)

### Features

* Text range support ([#145](https://www.github.com/AccessKit/accesskit/issues/145)) ([455e6f7](https://www.github.com/AccessKit/accesskit/commit/455e6f73bc058644d299c06eeeda9cc4cbe8844f))


### Code Refactoring

* Drop the `ignored` field and implement generic filtered tree traversal ([#143](https://www.github.com/AccessKit/accesskit/issues/143)) ([a4befe6](https://www.github.com/AccessKit/accesskit/commit/a4befe6e8a5afbe4a52dfd09eb87fdf2078d6c1d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.6.1 to 0.7.0

### [0.6.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.6.0...accesskit_consumer-v0.6.1) (2022-10-10)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.6.0 to 0.6.1

## [0.6.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.5.1...accesskit_consumer-v0.6.0) (2022-10-09)


### ⚠ BREAKING CHANGES

* **consumer:** Optimize tree access and change handling (#134)
* Wrap `TreeUpdate` nodes in `Arc` (#135)
* **consumer:** Make `Node::data` private to the crate (#137)
* Store node ID in `TreeUpdate`, not `accesskit::Node` (#132)

### Bug Fixes

* **consumer:** Drop printing of detached nodes before panic ([#136](https://www.github.com/AccessKit/accesskit/issues/136)) ([2f20477](https://www.github.com/AccessKit/accesskit/commit/2f204772a97d4e21205609f31f3e84bc878554cd))
* Don't try to optimize tree updates with unchanged nodes ([#138](https://www.github.com/AccessKit/accesskit/issues/138)) ([7721719](https://www.github.com/AccessKit/accesskit/commit/7721719fb0ab90bf41cc30dd0469c7de90228fe9))


### Code Refactoring

* **consumer:** Make `Node::data` private to the crate ([#137](https://www.github.com/AccessKit/accesskit/issues/137)) ([adb372d](https://www.github.com/AccessKit/accesskit/commit/adb372dda78d183c7189966e3bbc2d3780070513))
* **consumer:** Optimize tree access and change handling ([#134](https://www.github.com/AccessKit/accesskit/issues/134)) ([765ab74](https://www.github.com/AccessKit/accesskit/commit/765ab74efcf10a3b3871dc901d28f3cf1ff6020c))
* Store node ID in `TreeUpdate`, not `accesskit::Node` ([#132](https://www.github.com/AccessKit/accesskit/issues/132)) ([0bb86dd](https://www.github.com/AccessKit/accesskit/commit/0bb86ddb298cb5a253a91f07be0bad8b84b2fda3))
* Wrap `TreeUpdate` nodes in `Arc` ([#135](https://www.github.com/AccessKit/accesskit/issues/135)) ([907bc18](https://www.github.com/AccessKit/accesskit/commit/907bc1820b80d95833b6c5c3acaa2a8a4e93a6c2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.5.1 to 0.6.0

### [0.5.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.5.0...accesskit_consumer-v0.5.1) (2022-10-03)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.5.0 to 0.5.1

## [0.5.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.4.0...accesskit_consumer-v0.5.0) (2022-09-23)


### ⚠ BREAKING CHANGES

* Basic live regions (#128)

### Features

* Basic live regions ([#128](https://www.github.com/AccessKit/accesskit/issues/128)) ([03d745b](https://www.github.com/AccessKit/accesskit/commit/03d745b891147175bde2693cc10b96a2f6e31f39))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.4.0 to 0.5.0

## [0.4.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.3.0...accesskit_consumer-v0.4.0) (2022-07-22)


### ⚠ BREAKING CHANGES

* String indices are always in UTF-8 code units (#114)
* Drop unused tree IDs (#113)
* Switch to NonZeroU128 for NodeIDs (#99)

### Bug Fixes

* **consumer, platforms/windows:** Resolve new clippy warning ([#100](https://www.github.com/AccessKit/accesskit/issues/100)) ([e8cd95c](https://www.github.com/AccessKit/accesskit/commit/e8cd95c3741b39b77e4ddc8ce82efdc20f93f096))
* Migrate to 2021 edition ([#115](https://www.github.com/AccessKit/accesskit/issues/115)) ([f2333c8](https://www.github.com/AccessKit/accesskit/commit/f2333c8ce17d46aab6fc190338ab4cfcf8569f9e))
* Switch to NonZeroU128 for NodeIDs ([#99](https://www.github.com/AccessKit/accesskit/issues/99)) ([25a1a52](https://www.github.com/AccessKit/accesskit/commit/25a1a52c4562b163bfcc8c625a233c00a41aacf2))


### Code Refactoring

* Drop unused tree IDs ([#113](https://www.github.com/AccessKit/accesskit/issues/113)) ([ca60770](https://www.github.com/AccessKit/accesskit/commit/ca607702cee13c93fe538d2faec88e474261f7ab))
* String indices are always in UTF-8 code units ([#114](https://www.github.com/AccessKit/accesskit/issues/114)) ([386ca0a](https://www.github.com/AccessKit/accesskit/commit/386ca0a89c42fd201843f617b2fd6b6d1de77f59))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.3.0 to 0.4.0

## [0.3.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_consumer-v0.2.0...accesskit_consumer-v0.3.0) (2021-12-29)


### ⚠ BREAKING CHANGES

* Drop `TreeUpdate::clear` (#96)

### Code Refactoring

* Drop `TreeUpdate::clear` ([#96](https://www.github.com/AccessKit/accesskit/issues/96)) ([38f520b](https://www.github.com/AccessKit/accesskit/commit/38f520b960c6db7b3927b369aee206ee6bc5e8aa))



### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.2.0 to 0.3.0
