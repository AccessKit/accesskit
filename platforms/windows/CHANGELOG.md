# Changelog

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