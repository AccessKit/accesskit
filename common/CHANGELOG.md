# Changelog

## [0.11.2](https://github.com/AccessKit/accesskit/compare/accesskit-v0.11.1...accesskit-v0.11.2) (2023-08-08)


### Bug Fixes

* Support the enumn crate in all public enums ([#264](https://github.com/AccessKit/accesskit/issues/264)) ([b9b3cd1](https://github.com/AccessKit/accesskit/commit/b9b3cd18fccdd6526fb4f58c13eb91599452a3d6))

## [0.11.1](https://github.com/AccessKit/accesskit/compare/accesskit-v0.11.0...accesskit-v0.11.1) (2023-07-30)


### Bug Fixes

* Fix broken intra-doc-link. ([#262](https://github.com/AccessKit/accesskit/issues/262)) ([63c1715](https://github.com/AccessKit/accesskit/commit/63c17152d1eb8ae6ff19c2bc4a6756372bc490c2))

## [0.11.0](https://github.com/AccessKit/accesskit/compare/accesskit-v0.10.1...accesskit-v0.11.0) (2023-03-29)


### Features

* Add C bindings ([#230](https://github.com/AccessKit/accesskit/issues/230)) ([7f7f4c7](https://github.com/AccessKit/accesskit/commit/7f7f4c755890ab8210a5a8bf8e237ba6a51dd205))

## [0.10.1](https://github.com/AccessKit/accesskit/compare/accesskit-v0.10.0...accesskit-v0.10.1) (2023-02-20)


### Bug Fixes

* Set appropriate representations on all public types that will be exposed via FFI ([54e82f6](https://github.com/AccessKit/accesskit/commit/54e82f673f5c7b46d9077fe5f946305800862bf0))

## [0.10.0](https://github.com/AccessKit/accesskit/compare/accesskit-v0.9.0...accesskit-v0.10.0) (2023-02-12)


### ⚠ BREAKING CHANGES

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212))

### Code Refactoring

* Move thread synchronization into platform adapters; drop parking_lot ([#212](https://github.com/AccessKit/accesskit/issues/212)) ([5df52e5](https://github.com/AccessKit/accesskit/commit/5df52e5545faddf6a51905409013c2f5be23981e))

## [0.9.0](https://github.com/AccessKit/accesskit/compare/accesskit-v0.8.1...accesskit-v0.9.0) (2023-02-05)


### ⚠ BREAKING CHANGES

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205))

### Code Refactoring

* Make `Node` opaque and optimize it for size ([#205](https://github.com/AccessKit/accesskit/issues/205)) ([4811152](https://github.com/AccessKit/accesskit/commit/48111521439b76c1a8687418a4b20f9b705eac6d))

## [0.8.1](https://github.com/AccessKit/accesskit/compare/accesskit-v0.8.0...accesskit-v0.8.1) (2022-12-04)


### Documentation

* Fix outdated documentation for `TreeUpdate` ([#182](https://github.com/AccessKit/accesskit/issues/182)) ([dd658c7](https://github.com/AccessKit/accesskit/commit/dd658c70df55b2234a0346220362b0b9a40bb41d))

## [0.8.0](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.7.0...accesskit-v0.8.0) (2022-11-17)


### ⚠ BREAKING CHANGES

* Be opinionated about coordinates after all (#151)

### Code Refactoring

* Be opinionated about coordinates after all ([#151](https://www.github.com/AccessKit/accesskit/issues/151)) ([91a29a1](https://www.github.com/AccessKit/accesskit/commit/91a29a1bf99bca39e9a00a744025533924e45190))

## [0.7.0](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.6.1...accesskit-v0.7.0) (2022-11-11)


### ⚠ BREAKING CHANGES

* Text range support (#145)
* Drop the `ignored` field and implement generic filtered tree traversal (#143)

### Features

* Text range support ([#145](https://www.github.com/AccessKit/accesskit/issues/145)) ([455e6f7](https://www.github.com/AccessKit/accesskit/commit/455e6f73bc058644d299c06eeeda9cc4cbe8844f))


### Code Refactoring

* Drop the `ignored` field and implement generic filtered tree traversal ([#143](https://www.github.com/AccessKit/accesskit/issues/143)) ([a4befe6](https://www.github.com/AccessKit/accesskit/commit/a4befe6e8a5afbe4a52dfd09eb87fdf2078d6c1d))

### [0.6.1](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.6.0...accesskit-v0.6.1) (2022-10-10)


### Bug Fixes

* **common:** Restore compatibility with Rust 1.61 ([#139](https://www.github.com/AccessKit/accesskit/issues/139)) ([d8c6b16](https://www.github.com/AccessKit/accesskit/commit/d8c6b166c83796bfd6d748df60136029a9ec81d2))

## [0.6.0](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.5.1...accesskit-v0.6.0) (2022-10-09)


### ⚠ BREAKING CHANGES

* Wrap `TreeUpdate` nodes in `Arc` (#135)
* Store node ID in `TreeUpdate`, not `accesskit::Node` (#132)

### Bug Fixes

* Don't try to optimize tree updates with unchanged nodes ([#138](https://www.github.com/AccessKit/accesskit/issues/138)) ([7721719](https://www.github.com/AccessKit/accesskit/commit/7721719fb0ab90bf41cc30dd0469c7de90228fe9))


### Code Refactoring

* Store node ID in `TreeUpdate`, not `accesskit::Node` ([#132](https://www.github.com/AccessKit/accesskit/issues/132)) ([0bb86dd](https://www.github.com/AccessKit/accesskit/commit/0bb86ddb298cb5a253a91f07be0bad8b84b2fda3))
* Wrap `TreeUpdate` nodes in `Arc` ([#135](https://www.github.com/AccessKit/accesskit/issues/135)) ([907bc18](https://www.github.com/AccessKit/accesskit/commit/907bc1820b80d95833b6c5c3acaa2a8a4e93a6c2))

### [0.5.1](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.5.0...accesskit-v0.5.1) (2022-10-03)


### Bug Fixes

* **common:** Write a README specifically for the accesskit crate ([#130](https://www.github.com/AccessKit/accesskit/issues/130)) ([0c2f5cf](https://www.github.com/AccessKit/accesskit/commit/0c2f5cf71bdacf3142bff77defea36eeb2b4e1e9)), closes [#129](https://www.github.com/AccessKit/accesskit/issues/129)

## [0.5.0](https://www.github.com/AccessKit/accesskit/compare/accesskit-v0.4.0...accesskit-v0.5.0) (2022-09-23)


### ⚠ BREAKING CHANGES

* Basic live regions (#128)

### Features

* Basic live regions ([#128](https://www.github.com/AccessKit/accesskit/issues/128)) ([03d745b](https://www.github.com/AccessKit/accesskit/commit/03d745b891147175bde2693cc10b96a2f6e31f39))


### Bug Fixes

* **common:** Enable the serde feature when the schemars feature is turned on ([#122](https://www.github.com/AccessKit/accesskit/issues/122)) ([126b6e1](https://www.github.com/AccessKit/accesskit/commit/126b6e13294bee2b4c905a78147b49d763a61d05))
* **common:** Skip `ActionRequest::data` if it is `None` during serialization ([#123](https://www.github.com/AccessKit/accesskit/issues/123)) ([2d88ea8](https://www.github.com/AccessKit/accesskit/commit/2d88ea8518c99692beacfb955ef0bd4f388a4908))

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
