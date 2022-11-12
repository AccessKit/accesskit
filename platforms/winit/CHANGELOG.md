# Changelog

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
