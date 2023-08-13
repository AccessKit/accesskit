# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit_macos bumped from 0.1.4 to 0.1.5

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.0 to 0.8.1
    * accesskit_windows bumped from 0.10.0 to 0.10.1
    * accesskit_macos bumped from 0.2.0 to 0.2.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.10.1 to 0.10.2
    * accesskit_macos bumped from 0.2.1 to 0.3.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_macos bumped from 0.3.0 to 0.4.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.10.3 to 0.10.4
    * accesskit_macos bumped from 0.4.1 to 0.4.2
    * accesskit_unix bumped from 0.1.0 to 0.1.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.3.0 to 0.3.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.0 to 0.10.1
    * accesskit_windows bumped from 0.13.0 to 0.13.1
    * accesskit_macos bumped from 0.6.0 to 0.6.1
    * accesskit_unix bumped from 0.3.1 to 0.3.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.13.1 to 0.13.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit_macos bumped from 0.6.1 to 0.6.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.10.1 to 0.11.0
    * accesskit_windows bumped from 0.13.2 to 0.13.3
    * accesskit_macos bumped from 0.6.2 to 0.6.3
    * accesskit_unix bumped from 0.3.2 to 0.3.3

* The following workspace dependencies were updated
  * dependencies
    * accesskit_macos bumped from 0.7.0 to 0.7.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.0 to 0.11.1
    * accesskit_windows bumped from 0.14.0 to 0.14.1
    * accesskit_macos bumped from 0.7.1 to 0.8.0
    * accesskit_unix bumped from 0.5.0 to 0.5.1

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.11.1 to 0.11.2
    * accesskit_windows bumped from 0.14.1 to 0.14.2
    * accesskit_macos bumped from 0.8.0 to 0.9.0
    * accesskit_unix bumped from 0.5.1 to 0.5.2

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.14.2 to 0.14.3

## [0.14.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.13.0...accesskit_winit-v0.14.0) (2023-05-21)


### Features

* Add features for async runtimes on Unix ([#248](https://github.com/AccessKit/accesskit/issues/248)) ([b56b4ea](https://github.com/AccessKit/accesskit/commit/b56b4ea7c967ee5a1dae21a2fa0dcd385346031e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_unix bumped from 0.4.0 to 0.5.0

## [0.13.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.12.5...accesskit_winit-v0.13.0) (2023-03-30)


### ⚠ BREAKING CHANGES

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234))

### Bug Fixes

* Force a semver-breaking version bump in downstream crates ([#234](https://github.com/AccessKit/accesskit/issues/234)) ([773389b](https://github.com/AccessKit/accesskit/commit/773389bff857fa18edf15de426e029251fc34591))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.13.3 to 0.14.0
    * accesskit_macos bumped from 0.6.3 to 0.7.0
    * accesskit_unix bumped from 0.3.3 to 0.4.0

## [0.12.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.11.0...accesskit_winit-v0.12.0) (2023-02-18)


### Features

* Feature-gate the Unix adapter in accesskit_winit ([#214](https://github.com/AccessKit/accesskit/issues/214)) ([be95807](https://github.com/AccessKit/accesskit/commit/be95807dda64f2a49b4d20cc9084b14a7aa2844e))

## [0.11.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.10.0...accesskit_winit-v0.11.0) (2023-02-12)


### ⚠ BREAKING CHANGES

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212))

### Code Refactoring

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212)) ([5df52e5](https://github.com/AccessKit/accesskit/commit/5df52e5545faddf6a51905409013c2f5be23981e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.9.0 to 0.10.0
    * accesskit_windows bumped from 0.12.0 to 0.13.0
    * accesskit_macos bumped from 0.5.0 to 0.6.0
    * accesskit_unix bumped from 0.2.0 to 0.3.0

## [0.10.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.9.1...accesskit_winit-v0.10.0) (2023-02-05)


### ⚠ BREAKING CHANGES

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205))

### Code Refactoring

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205)) ([4811152](https://github.com/AccessKit/accesskit/commit/48111521439b76c1a8687418a4b20f9b705eac6d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.1 to 0.9.0
    * accesskit_windows bumped from 0.11.0 to 0.12.0
    * accesskit_macos bumped from 0.4.2 to 0.5.0
    * accesskit_unix bumped from 0.1.1 to 0.2.0

## [0.9.1](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.9.0...accesskit_winit-v0.9.1) (2023-02-05)


### Bug Fixes

* Don't force winit's X11 and Wayland features to be enabled ([#209](https://github.com/AccessKit/accesskit/issues/209)) ([a3ed357](https://github.com/AccessKit/accesskit/commit/a3ed35754ad8f69a8ed54adacc30b6d57c19329a))

## [0.9.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.8.1...accesskit_winit-v0.9.0) (2023-02-02)


### ⚠ BREAKING CHANGES

* Update winit to 0.28 ([#207](https://github.com/AccessKit/accesskit/issues/207))

### Miscellaneous Chores

* Update winit to 0.28 ([#207](https://github.com/AccessKit/accesskit/issues/207)) ([3ff0cf5](https://github.com/AccessKit/accesskit/commit/3ff0cf59f982af504499142a3804f7aeeb4defe0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.10.4 to 0.11.0

## [0.8.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.7.3...accesskit_winit-v0.8.0) (2023-01-05)


### Features

* Basic Unix platform adapter ([#198](https://github.com/AccessKit/accesskit/issues/198)) ([1cea32e](https://github.com/AccessKit/accesskit/commit/1cea32e44ee743b778ac941ceff9087ae745cb37))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.10.2 to 0.10.3
    * accesskit_macos bumped from 0.4.0 to 0.4.1

## [0.7.0](https://github.com/AccessKit/accesskit/compare/accesskit_winit-v0.6.6...accesskit_winit-v0.7.0) (2022-11-29)


### ⚠ BREAKING CHANGES

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179))

### Code Refactoring

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179)) ([f35c941](https://github.com/AccessKit/accesskit/commit/f35c941f395f3162db376a69cfaaaf770d376267))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_windows bumped from 0.9.3 to 0.10.0
    * accesskit_macos bumped from 0.1.5 to 0.2.0

### [0.6.4](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.6.3...accesskit_winit-v0.6.4) (2022-11-25)


### Bug Fixes

* Reduce the winit version requirement to match egui ([#170](https://www.github.com/AccessKit/accesskit/issues/170)) ([1d27482](https://www.github.com/AccessKit/accesskit/commit/1d27482221140c1f3b3e3eaf93e7feaf8105611d))

## [0.6.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.5.1...accesskit_winit-v0.6.0) (2022-11-23)


### Features

* **platforms/macos:** Basic macOS platform adapter ([#158](https://www.github.com/AccessKit/accesskit/issues/158)) ([a06725e](https://www.github.com/AccessKit/accesskit/commit/a06725e952e6041dbd366944fa793b746c9f195e))


### Bug Fixes

* **platforms/macos:** Fix macOS crate version number ([#161](https://www.github.com/AccessKit/accesskit/issues/161)) ([e0a6a40](https://www.github.com/AccessKit/accesskit/commit/e0a6a401050cdcaea4efa870ed77ae94388f1ce0))
* **platforms/windows:** Re-export the windows-rs HWND type ([#159](https://www.github.com/AccessKit/accesskit/issues/159)) ([389187a](https://www.github.com/AccessKit/accesskit/commit/389187ac5e96895ed1763d14d315d2f8f4256460))

### [0.5.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.5.0...accesskit_winit-v0.5.1) (2022-11-17)


### Bug Fixes

* **platforms/winit:** Eliminate some problematic indirect dependencies ([#154](https://www.github.com/AccessKit/accesskit/issues/154)) ([58048ae](https://www.github.com/AccessKit/accesskit/commit/58048aebedc293eda5c5819ea66db9b40b8926b0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.7.0 to 0.8.0

## [0.5.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.4.0...accesskit_winit-v0.5.0) (2022-11-14)


### Features

* **platforms/winit:** Allow a custom action handler ([#149](https://www.github.com/AccessKit/accesskit/issues/149)) ([cdb1a16](https://www.github.com/AccessKit/accesskit/commit/cdb1a164de06f18cad497409a514f270a8336b4c))

## [0.4.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.3.3...accesskit_winit-v0.4.0) (2022-11-12)


### ⚠ BREAKING CHANGES

* **platforms/windows:** Update to windows-rs 0.42.0 (#148)

### Bug Fixes

* **consumer, platforms/windows, platforms/winit:** Update to parking_lot 0.12.1 ([#146](https://www.github.com/AccessKit/accesskit/issues/146)) ([6772855](https://www.github.com/AccessKit/accesskit/commit/6772855a7b540fd728faad15d8d208b05c1bbd8a))
* **platforms/windows:** Update to windows-rs 0.42.0 ([#148](https://www.github.com/AccessKit/accesskit/issues/148)) ([70d1a89](https://www.github.com/AccessKit/accesskit/commit/70d1a89f51fd6c3a32b7192d9d7f3937db09d196))

### [0.3.3](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.3.2...accesskit_winit-v0.3.3) (2022-11-11)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.6.1 to 0.7.0

### [0.3.2](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.3.1...accesskit_winit-v0.3.2) (2022-10-11)


### Bug Fixes

* **platforms/winit:** Derive `Debug` on `ActionRequestEvent` ([#141](https://www.github.com/AccessKit/accesskit/issues/141)) ([8b84c75](https://www.github.com/AccessKit/accesskit/commit/8b84c7547c6fdb52cd6d5c6d79f812dc614f08dd))

### [0.3.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.3.0...accesskit_winit-v0.3.1) (2022-10-10)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.6.0 to 0.6.1

## [0.3.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.2.1...accesskit_winit-v0.3.0) (2022-10-09)


### ⚠ BREAKING CHANGES

* Wrap `TreeUpdate` nodes in `Arc` (#135)
* Store node ID in `TreeUpdate`, not `accesskit::Node` (#132)

### Code Refactoring

* Store node ID in `TreeUpdate`, not `accesskit::Node` ([#132](https://www.github.com/AccessKit/accesskit/issues/132)) ([0bb86dd](https://www.github.com/AccessKit/accesskit/commit/0bb86ddb298cb5a253a91f07be0bad8b84b2fda3))
* Wrap `TreeUpdate` nodes in `Arc` ([#135](https://www.github.com/AccessKit/accesskit/issues/135)) ([907bc18](https://www.github.com/AccessKit/accesskit/commit/907bc1820b80d95833b6c5c3acaa2a8a4e93a6c2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.5.1 to 0.6.0

### [0.2.1](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.2.0...accesskit_winit-v0.2.1) (2022-10-03)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.5.0 to 0.5.1

## [0.2.0](https://www.github.com/AccessKit/accesskit/compare/accesskit_winit-v0.1.0...accesskit_winit-v0.2.0) (2022-09-23)


### ⚠ BREAKING CHANGES

* Basic live regions (#128)
* **platforms/windows:** Bump windows-rs dependency (#126)
* **platforms/winit:** Bump winit dependency (#125)

### Features

* Basic live regions ([#128](https://www.github.com/AccessKit/accesskit/issues/128)) ([03d745b](https://www.github.com/AccessKit/accesskit/commit/03d745b891147175bde2693cc10b96a2f6e31f39))


### Miscellaneous Chores

* **platforms/windows:** Bump windows-rs dependency ([#126](https://www.github.com/AccessKit/accesskit/issues/126)) ([472a75e](https://www.github.com/AccessKit/accesskit/commit/472a75e4214b90396f3282f247df08100ed8362d))
* **platforms/winit:** Bump winit dependency ([#125](https://www.github.com/AccessKit/accesskit/issues/125)) ([6026c1b](https://www.github.com/AccessKit/accesskit/commit/6026c1b2ecede3ca2f2076075ed158000154b34e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.4.0 to 0.5.0

## 0.1.0 (2022-07-22)


### Features

* **platforms/winit:** New winit adapter ([#121](https://www.github.com/AccessKit/accesskit/issues/121)) ([fdc274e](https://www.github.com/AccessKit/accesskit/commit/fdc274e7d3a901873d2ad0c7a4824a19111787ef))



### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.3.0 to 0.4.0
