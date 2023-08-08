# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.0 to 0.8.1
    * accesskit_consumer bumped from 0.9.1 to 0.10.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.11.0 to 0.12.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.12.0 to 0.12.1

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

## [0.14.3](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.14.2...accesskit_windows-v0.14.3) (2023-08-08)


### Bug Fixes

* Update windows crate to v0.48 ([#257](https://github.com/AccessKit/accesskit/issues/257)) ([cc703ed](https://github.com/AccessKit/accesskit/commit/cc703ed33d535aa1803e423a53beff9354b5b0df))

## [0.14.0](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.13.3...accesskit_windows-v0.14.0) (2023-03-30)


### ⚠ BREAKING CHANGES

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234))

### Bug Fixes

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234)) ([773389b](https://github.com/AccessKit/accesskit/commit/773389bff857fa18edf15de426e029251fc34591))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.14.2 to 0.15.0

## [0.13.2](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.13.1...accesskit_windows-v0.13.2) (2023-02-23)


### Bug Fixes

* Fix Windows 32-bit build errors ([#223](https://github.com/AccessKit/accesskit/issues/223)) ([41f28b6](https://github.com/AccessKit/accesskit/commit/41f28b670ac457b2d067bbc4ba40aa0fc8842e4d))

## [0.13.1](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.13.0...accesskit_windows-v0.13.1) (2023-02-20)


### Bug Fixes

* Update windows-rs to 0.44 ([#220](https://github.com/AccessKit/accesskit/issues/220)) ([a6b0a12](https://github.com/AccessKit/accesskit/commit/a6b0a124e7511e37760d769b517fd5fc9050160b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.0 to 0.10.1
    * accesskit_consumer bumped from 0.14.0 to 0.14.1

## [0.13.0](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.12.0...accesskit_windows-v0.13.0) (2023-02-12)


### ⚠ BREAKING CHANGES

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212))

### Code Refactoring

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212)) ([5df52e5](https://github.com/AccessKit/accesskit/commit/5df52e5545faddf6a51905409013c2f5be23981e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.9.0 to 0.10.0
    * accesskit_consumer bumped from 0.13.0 to 0.14.0

## [0.12.0](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.11.0...accesskit_windows-v0.12.0) (2023-02-05)


### ⚠ BREAKING CHANGES

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205))

### Code Refactoring

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205)) ([4811152](https://github.com/AccessKit/accesskit/commit/48111521439b76c1a8687418a4b20f9b705eac6d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.1 to 0.9.0
    * accesskit_consumer bumped from 0.12.1 to 0.13.0

## [0.11.0](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.10.4...accesskit_windows-v0.11.0) (2023-02-02)


### ⚠ BREAKING CHANGES

* Update winit to 0.28 ([#207](https://github.com/AccessKit/accesskit/issues/207))

### Miscellaneous Chores

* Update winit to 0.28 ([#207](https://github.com/AccessKit/accesskit/issues/207)) ([3ff0cf5](https://github.com/AccessKit/accesskit/commit/3ff0cf59f982af504499142a3804f7aeeb4defe0))

## [0.10.2](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.10.1...accesskit_windows-v0.10.2) (2022-12-17)


### Bug Fixes

* Correct broken UIA method implementation that was incompatible with Windows 11 ATs ([#193](https://github.com/AccessKit/accesskit/issues/193)) ([3c527c7](https://github.com/AccessKit/accesskit/commit/3c527c76cb4139402d2b5550d2eb1ad12e07ebe5))
* More reliable handling of the edge case for wrapped lines ([#192](https://github.com/AccessKit/accesskit/issues/192)) ([c626d2c](https://github.com/AccessKit/accesskit/commit/c626d2c3028085b076ada7dd31242cf3ca3c0f08))
* Provide fallback property implementations for the window root ([#194](https://github.com/AccessKit/accesskit/issues/194)) ([f3d30b9](https://github.com/AccessKit/accesskit/commit/f3d30b9ba2f66e08fb7f78c304ab8a9e83e1aeca))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.10.0 to 0.11.0

## [0.10.0](https://github.com/AccessKit/accesskit/compare/accesskit_windows-v0.9.3...accesskit_windows-v0.10.0) (2022-11-29)


### ⚠ BREAKING CHANGES

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179))

### Code Refactoring

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179)) ([f35c941](https://github.com/AccessKit/accesskit/commit/f35c941f395f3162db376a69cfaaaf770d376267))

### [0.9.3](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.9.2...accesskit_windows-v0.9.3) (2022-11-25)


### Bug Fixes

* Gracefully handle nodes that only support text ranges some of the time ([#169](https://www.github.com/AccessKit/accesskit/issues/169)) ([1f50df6](https://www.github.com/AccessKit/accesskit/commit/1f50df6820b9d23fe2e579f043f4981acf285de2))
* **platforms/windows:** Change default minimum and maximum for range value pattern ([#166](https://www.github.com/AccessKit/accesskit/issues/166)) ([176ec58](https://www.github.com/AccessKit/accesskit/commit/176ec5853ca127b1e12f9a992b75478177a4acce))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.9.0 to 0.9.1

### [0.9.2](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.9.1...accesskit_windows-v0.9.2) (2022-11-24)


### Bug Fixes

* **platforms/windows:** Re-export more types from windows-rs ([#162](https://www.github.com/AccessKit/accesskit/issues/162)) ([eed692b](https://www.github.com/AccessKit/accesskit/commit/eed692b27407e1ddd5f200464f0e2d52a272b958))

### [0.9.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.9.0...accesskit_windows-v0.9.1) (2022-11-23)


### Bug Fixes

* **platforms/windows:** Re-export the windows-rs HWND type ([#159](https://www.github.com/AccessKit/accesskit/issues/159)) ([389187a](https://www.github.com/AccessKit/accesskit/commit/389187ac5e96895ed1763d14d315d2f8f4256460))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.8.0 to 0.9.0

## [0.9.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.8.0...accesskit_windows-v0.9.0) (2022-11-17)


### ⚠ BREAKING CHANGES

* **consumer:** Eliminate the dependency on `im` due to licensing (#153)

### Code Refactoring

* **consumer:** Eliminate the dependency on `im` due to licensing ([#153](https://www.github.com/AccessKit/accesskit/issues/153)) ([b4c4cb5](https://www.github.com/AccessKit/accesskit/commit/b4c4cb5713d4833d8ee7979e4f4e39c7e96a3ed4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.7.0 to 0.8.0
    * accesskit_consumer bumped from 0.7.1 to 0.8.0

## [0.8.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.7.0...accesskit_windows-v0.8.0) (2022-11-12)


### ⚠ BREAKING CHANGES

* **platforms/windows:** Update to windows-rs 0.42.0 (#148)

### Bug Fixes

* **consumer, platforms/windows, platforms/winit:** Update to parking_lot 0.12.1 ([#146](https://www.github.com/AccessKit/accesskit/issues/146)) ([6772855](https://www.github.com/AccessKit/accesskit/commit/6772855a7b540fd728faad15d8d208b05c1bbd8a))
* **platforms/windows:** Update to windows-rs 0.42.0 ([#148](https://www.github.com/AccessKit/accesskit/issues/148)) ([70d1a89](https://www.github.com/AccessKit/accesskit/commit/70d1a89f51fd6c3a32b7192d9d7f3937db09d196))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.7.0 to 0.7.1

## [0.7.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.6.1...accesskit_windows-v0.7.0) (2022-11-11)


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
    * accesskit_consumer bumped from 0.6.1 to 0.7.0

### [0.6.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.6.0...accesskit_windows-v0.6.1) (2022-10-10)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.6.0 to 0.6.1
    * accesskit_consumer bumped from 0.6.0 to 0.6.1

## [0.6.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.5.1...accesskit_windows-v0.6.0) (2022-10-09)


### ⚠ BREAKING CHANGES

* **consumer:** Optimize tree access and change handling (#134)
* Wrap `TreeUpdate` nodes in `Arc` (#135)
* **consumer:** Make `Node::data` private to the crate (#137)
* Store node ID in `TreeUpdate`, not `accesskit::Node` (#132)

### Code Refactoring

* **consumer:** Make `Node::data` private to the crate ([#137](https://www.github.com/AccessKit/accesskit/issues/137)) ([adb372d](https://www.github.com/AccessKit/accesskit/commit/adb372dda78d183c7189966e3bbc2d3780070513))
* **consumer:** Optimize tree access and change handling ([#134](https://www.github.com/AccessKit/accesskit/issues/134)) ([765ab74](https://www.github.com/AccessKit/accesskit/commit/765ab74efcf10a3b3871dc901d28f3cf1ff6020c))
* Store node ID in `TreeUpdate`, not `accesskit::Node` ([#132](https://www.github.com/AccessKit/accesskit/issues/132)) ([0bb86dd](https://www.github.com/AccessKit/accesskit/commit/0bb86ddb298cb5a253a91f07be0bad8b84b2fda3))
* Wrap `TreeUpdate` nodes in `Arc` ([#135](https://www.github.com/AccessKit/accesskit/issues/135)) ([907bc18](https://www.github.com/AccessKit/accesskit/commit/907bc1820b80d95833b6c5c3acaa2a8a4e93a6c2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.5.1 to 0.6.0
    * accesskit_consumer bumped from 0.5.1 to 0.6.0

### [0.5.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.5.0...accesskit_windows-v0.5.1) (2022-10-03)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.5.0 to 0.5.1
    * accesskit_consumer bumped from 0.5.0 to 0.5.1

## [0.5.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.4.0...accesskit_windows-v0.5.0) (2022-09-23)


### ⚠ BREAKING CHANGES

* Basic live regions (#128)
* **platforms/windows:** Bump windows-rs dependency (#126)

### Features

* Basic live regions ([#128](https://www.github.com/AccessKit/accesskit/issues/128)) ([03d745b](https://www.github.com/AccessKit/accesskit/commit/03d745b891147175bde2693cc10b96a2f6e31f39))


### Miscellaneous Chores

* **platforms/windows:** Bump windows-rs dependency ([#126](https://www.github.com/AccessKit/accesskit/issues/126)) ([472a75e](https://www.github.com/AccessKit/accesskit/commit/472a75e4214b90396f3282f247df08100ed8362d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.4.0 to 0.5.0
    * accesskit_consumer bumped from 0.4.0 to 0.5.0

## [0.4.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.3.0...accesskit_windows-v0.4.0) (2022-07-22)


### ⚠ BREAKING CHANGES

* String indices are always in UTF-8 code units (#114)
* **platforms/windows:** Refactor window subclassing to avoid lifetime issue (#120)
* **platforms/windows:** Simplify the adapter API by always boxing the tree source (#119)
* Drop unused tree IDs (#113)
* **platforms/windows:** Migrate to windows-rs 0.37 (#109)
* Switch to NonZeroU128 for NodeIDs (#99)

### Features

* **platforms/windows:** Win32 subclassing support ([#118](https://www.github.com/AccessKit/accesskit/issues/118)) ([60c69b7](https://www.github.com/AccessKit/accesskit/commit/60c69b7b8a18ca8db62a84495b9e71a6e8140204))
* **platforms/winit:** New winit adapter ([#121](https://www.github.com/AccessKit/accesskit/issues/121)) ([fdc274e](https://www.github.com/AccessKit/accesskit/commit/fdc274e7d3a901873d2ad0c7a4824a19111787ef))


### Bug Fixes

* **consumer, platforms/windows:** Resolve new clippy warning ([#100](https://www.github.com/AccessKit/accesskit/issues/100)) ([e8cd95c](https://www.github.com/AccessKit/accesskit/commit/e8cd95c3741b39b77e4ddc8ce82efdc20f93f096))
* Migrate to 2021 edition ([#115](https://www.github.com/AccessKit/accesskit/issues/115)) ([f2333c8](https://www.github.com/AccessKit/accesskit/commit/f2333c8ce17d46aab6fc190338ab4cfcf8569f9e))
* **platforms/windows:** Print usage text to the terminal from the Windows example ([#103](https://www.github.com/AccessKit/accesskit/issues/103)) ([7fba3ce](https://www.github.com/AccessKit/accesskit/commit/7fba3ce55345d7787f08d2ae60d841dd13b27693))
* **platforms/windows:** Restore the optimization of the FragmentRoot method ([#116](https://www.github.com/AccessKit/accesskit/issues/116)) ([d48c31b](https://www.github.com/AccessKit/accesskit/commit/d48c31b41f35baebe59bb654b38dd48265062b14))
* Switch to NonZeroU128 for NodeIDs ([#99](https://www.github.com/AccessKit/accesskit/issues/99)) ([25a1a52](https://www.github.com/AccessKit/accesskit/commit/25a1a52c4562b163bfcc8c625a233c00a41aacf2))


### Miscellaneous Chores

* **platforms/windows:** Migrate to windows-rs 0.37 ([#109](https://www.github.com/AccessKit/accesskit/issues/109)) ([1065e11](https://www.github.com/AccessKit/accesskit/commit/1065e11421176a8abc37ef579cb47d973c968462))


### Code Refactoring

* Drop unused tree IDs ([#113](https://www.github.com/AccessKit/accesskit/issues/113)) ([ca60770](https://www.github.com/AccessKit/accesskit/commit/ca607702cee13c93fe538d2faec88e474261f7ab))
* **platforms/windows:** Refactor window subclassing to avoid lifetime issue ([#120](https://www.github.com/AccessKit/accesskit/issues/120)) ([37579aa](https://www.github.com/AccessKit/accesskit/commit/37579aa8dd0c019ffaf4eac1b0bf1f7a8c719323))
* **platforms/windows:** Simplify the adapter API by always boxing the tree source ([#119](https://www.github.com/AccessKit/accesskit/issues/119)) ([27d5c78](https://www.github.com/AccessKit/accesskit/commit/27d5c78afa0f8d1ae3b626265da8bccd3e5b09d1))
* String indices are always in UTF-8 code units ([#114](https://www.github.com/AccessKit/accesskit/issues/114)) ([386ca0a](https://www.github.com/AccessKit/accesskit/commit/386ca0a89c42fd201843f617b2fd6b6d1de77f59))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.3.0 to 0.4.0
    * accesskit_consumer bumped from 0.3.0 to 0.4.0

## [0.3.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_windows-v0.2.0...accesskit_windows-v0.3.0) (2021-12-29)


### ⚠ BREAKING CHANGES

* Drop `TreeUpdate::clear` (#96)

### Code Refactoring

* Drop `TreeUpdate::clear` ([#96](https://www.github.com/AccessKit/accesskit/issues/96)) ([38f520b](https://www.github.com/AccessKit/accesskit/commit/38f520b960c6db7b3927b369aee206ee6bc5e8aa))



### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.2.0 to 0.3.0
    * accesskit_consumer bumped from 0.2.0 to 0.3.0
