# Changelog

## [0.4.0](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.3.0...accesskit-v0.4.0) (2022-07-22)


### ⚠ BREAKING CHANGES

* String indices are always in UTF-8 code units (#114)
* Drop unused tree IDs (#113)
* Switch to NonZeroU128 for NodeIDs (#99)

### Features

* **common:** Conversion from `NonZeroU64` to `NodeId` ([#112](https://www.github.com/AccessKit/accesskit/issues/112)) ([b7adfb9](https://www.github.com/AccessKit/accesskit/commit/b7adfb906cb09107be71a148b5199ba87df2a6b3))


### Bug Fixes

* **common:** Various documentation fixes and improvements ([#111](https://www.github.com/AccessKit/accesskit/issues/111)) ([4d27234](https://www.github.com/AccessKit/accesskit/commit/4d27234195e96de65bf55869877405cb5e45f6fc))
* Migrate to 2021 edition ([#115](https://www.github.com/AccessKit/accesskit/issues/115)) ([f2333c8](https://www.github.com/AccessKit/accesskit/commit/f2333c8ce17d46aab6fc190338ab4cfcf8569f9e))
* Switch to NonZeroU128 for NodeIDs ([#99](https://www.github.com/AccessKit/accesskit/issues/99)) ([25a1a52](https://www.github.com/AccessKit/accesskit/commit/25a1a52c4562b163bfcc8c625a233c00a41aacf2))


### Code Refactoring

* Drop unused tree IDs ([#113](https://www.github.com/AccessKit/accesskit/issues/113)) ([ca60770](https://www.github.com/AccessKit/accesskit/commit/ca607702cee13c93fe538d2faec88e474261f7ab))
* String indices are always in UTF-8 code units ([#114](https://www.github.com/AccessKit/accesskit/issues/114)) ([386ca0a](https://www.github.com/AccessKit/accesskit/commit/386ca0a89c42fd201843f617b2fd6b6d1de77f59))

## [0.3.0](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.2.0...accesskit-v0.3.0) (2021-12-29)


### ⚠ BREAKING CHANGES

* Drop `TreeUpdate::clear` (#96)

### Code Refactoring

* Drop `TreeUpdate::clear` ([#96](https://www.github.com/AccessKit/accesskit/issues/96)) ([38f520b](https://www.github.com/AccessKit/accesskit/commit/38f520b960c6db7b3927b369aee206ee6bc5e8aa))
