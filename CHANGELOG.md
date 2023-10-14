# Changelog

## [0.2.0](https://github.com/alexpovel/srgn/compare/v0.1.0...v0.2.0) (2023-10-14)


### Features

* `fail-any` and `fail-none` feature flags ([1dd3dcb](https://github.com/alexpovel/srgn/commit/1dd3dcb3b233d787abc91622ddc7ce019c764878))


### Bug Fixes

* **ci:** Linking/cc fails for tarpaulin; try w/o cache ([ab40957](https://github.com/alexpovel/srgn/commit/ab409571417881a599c3fd32645dd26a5c9d8349))
* Do not `pub use` `Action` ([1e2c663](https://github.com/alexpovel/srgn/commit/1e2c663473f5c6140f065301f815f3cd3726837a))

## 0.1.0 (2023-10-05)


### Features

* `german_prefer_original` option ([a25479a](https://github.com/alexpovel/srgn/commit/a25479ac3b4e1d650311f24b0a624bd8e62386e1)), closes [#25](https://github.com/alexpovel/srgn/issues/25)
* `is_compound_word` -&gt; `decompose_compound_word` ([8cf6175](https://github.com/alexpovel/srgn/commit/8cf6175a6d7e4470482f413c125cfc94c5c36335))
* `squeeze` stage, `Scoped` concept ([4c18820](https://github.com/alexpovel/srgn/commit/4c18820c9b0e7e85a475af792821debb85c0c12e))
* `symbols` stage ([daae90c](https://github.com/alexpovel/srgn/commit/daae90cb1b5e5b67e17ad19b6b8bee0e56111eba))
* Add memoization ([58b5fdf](https://github.com/alexpovel/srgn/commit/58b5fdf2ab6537a3cb4bff33acbd3e96b27cc5f7)), closes [#1](https://github.com/alexpovel/srgn/issues/1)
* Child options and flags imply their parent ([20518c2](https://github.com/alexpovel/srgn/commit/20518c2ea8ce62bbef3c97663d93446cdaaf8d56))
* CSharp support (comments) and `UserService.cs` sample ([1cef201](https://github.com/alexpovel/srgn/commit/1cef20113cad889058c00c024fca59ba2a8b5507))
* Deletion stage ([4fd7e76](https://github.com/alexpovel/srgn/commit/4fd7e7646d908e9a68dc6a8254258934de74fd3f))
* Introduce FSTs ([f3434d6](https://github.com/alexpovel/srgn/commit/f3434d6bcac9c44763d27929d6dde2f58a3f68d3))
* Introduce proper error enum for word casing ([e3a8c5a](https://github.com/alexpovel/srgn/commit/e3a8c5a9da069f2dc97d4417fb2d9c47be301cec))
* Inversion ([b0c3b6b](https://github.com/alexpovel/srgn/commit/b0c3b6b00f9393fb5f63d2e3d531005fecde2d35))
* Lowercasing stage ([e0b097a](https://github.com/alexpovel/srgn/commit/e0b097a692735257c9dce063682b662b46ebb0ed))
* naive mode for German ([cb3357c](https://github.com/alexpovel/srgn/commit/cb3357c21aae80735c1986e89564f924f27e0e83))
* replacement stage ([8886880](https://github.com/alexpovel/srgn/commit/88868805b8cafd7770f1252a9ce10986fa82cec5))
* Support upper/mixed case special characters ([90111da](https://github.com/alexpovel/srgn/commit/90111da3fa69cb7fd105856608acf1afd9a05a49)), closes [#5](https://github.com/alexpovel/srgn/issues/5)
* TypeScript, with TODO app example ([3d3ed21](https://github.com/alexpovel/srgn/commit/3d3ed21582cc91d76e62bc8729dbeb38f70ebfb9))
* uppercase stage ([c92ad21](https://github.com/alexpovel/srgn/commit/c92ad21584c875aa196ac18dea8f845bb610b4be))
* Use `decompound` ([c640363](https://github.com/alexpovel/srgn/commit/c64036351fc7d0ea32b89f4744de97c04d8c39fe))
* Use `once_cell` to build `fst::Set` only once ([3eae2b6](https://github.com/alexpovel/srgn/commit/3eae2b6408feea8c824e13a3a718011a3f9326a1))
* Verbosity switch ([8c05d69](https://github.com/alexpovel/srgn/commit/8c05d69451be0fd072ee4e62b818d0f8206e5d41))
* Word list performance increased (now a single &str) ([32716fa](https://github.com/alexpovel/srgn/commit/32716fae902bb8c744b3898b73437e375539d469))
* Working tree-sitter for language scoping ([bf17589](https://github.com/alexpovel/srgn/commit/bf17589f782aeab41e61f51044999116065b3a74))


### Bug Fixes

* Add `aufwändig` to German word list ([69a138b](https://github.com/alexpovel/srgn/commit/69a138b050eea0a1e128e9d47543db68da13601d))
* **ci:** cargo-tarpaulin v0.27 broke 'Xml' ([df585a2](https://github.com/alexpovel/srgn/commit/df585a2e839605d4f39e32e5c6e51ce5c473146e))
* **ci:** Checkout and cache code coverage run ([938b7a5](https://github.com/alexpovel/srgn/commit/938b7a57b48b20a1b7797d4a02933f60fc017871))
* **ci:** Run coverage test in parallel ([f4167d5](https://github.com/alexpovel/srgn/commit/f4167d594ba701b6ff54bc150f2e7c9eec017134))
* **ci:** Run release chores in parallel ([afca5cf](https://github.com/alexpovel/srgn/commit/afca5cf8d94d0dfc856b71f2fb9325a5bf5c7032))
* **ci:** Trigger release-please ([505110a](https://github.com/alexpovel/srgn/commit/505110a6781bafdd4bb50210159f3d1f0cd90ab0))
* **ci:** Update all dependencies ([928a6d5](https://github.com/alexpovel/srgn/commit/928a6d5d219b7cee6c9698d9942fca7fb653550a))
* Drop custom `Span`, use `std::ops::Range` ([fad059e](https://github.com/alexpovel/srgn/commit/fad059e2f07b80581dee3c71fc9ac48fa4398fd4))
* Fix tests after project rename ([f5d4f17](https://github.com/alexpovel/srgn/commit/f5d4f1787148ddec5786f5a8c14d572624ac2873))
* Make initial/missing `Transition` an unrepresentable state ([ba2948d](https://github.com/alexpovel/srgn/commit/ba2948de94205b6729a02b57f611d3a287138387))
* Squeezing now has `tr`-like `squeeze-repeats` alias ([4d67a45](https://github.com/alexpovel/srgn/commit/4d67a458171a237b3b782753e140a564ed7f84d2))
* **test:** Symbols stage is *not* fully idempotent ([5ff9277](https://github.com/alexpovel/srgn/commit/5ff92773a01435db60b78c8c3e819533cacbfcdb))


### Performance Improvements

* Add profiling tooling (as a Justfile, for automation) ([9503761](https://github.com/alexpovel/srgn/commit/9503761dba51fd36d5dd1fd77937c4cc133f624c)), closes [#14](https://github.com/alexpovel/srgn/issues/14)

## 0.1.0 (2023-10-03)


### Features

* `german_prefer_original` option ([a25479a](https://github.com/alexpovel/betterletters/commit/a25479ac3b4e1d650311f24b0a624bd8e62386e1)), closes [#25](https://github.com/alexpovel/betterletters/issues/25)
* `is_compound_word` -&gt; `decompose_compound_word` ([8cf6175](https://github.com/alexpovel/betterletters/commit/8cf6175a6d7e4470482f413c125cfc94c5c36335))
* `squeeze` stage, `Scoped` concept ([4c18820](https://github.com/alexpovel/betterletters/commit/4c18820c9b0e7e85a475af792821debb85c0c12e))
* `symbols` stage ([daae90c](https://github.com/alexpovel/betterletters/commit/daae90cb1b5e5b67e17ad19b6b8bee0e56111eba))
* Add memoization ([58b5fdf](https://github.com/alexpovel/betterletters/commit/58b5fdf2ab6537a3cb4bff33acbd3e96b27cc5f7)), closes [#1](https://github.com/alexpovel/betterletters/issues/1)
* Child options and flags imply their parent ([20518c2](https://github.com/alexpovel/betterletters/commit/20518c2ea8ce62bbef3c97663d93446cdaaf8d56))
* CSharp support (comments) and `UserService.cs` sample ([1cef201](https://github.com/alexpovel/betterletters/commit/1cef20113cad889058c00c024fca59ba2a8b5507))
* Deletion stage ([4fd7e76](https://github.com/alexpovel/betterletters/commit/4fd7e7646d908e9a68dc6a8254258934de74fd3f))
* Introduce FSTs ([f3434d6](https://github.com/alexpovel/betterletters/commit/f3434d6bcac9c44763d27929d6dde2f58a3f68d3))
* Introduce proper error enum for word casing ([e3a8c5a](https://github.com/alexpovel/betterletters/commit/e3a8c5a9da069f2dc97d4417fb2d9c47be301cec))
* Inversion ([b0c3b6b](https://github.com/alexpovel/betterletters/commit/b0c3b6b00f9393fb5f63d2e3d531005fecde2d35))
* Lowercasing stage ([e0b097a](https://github.com/alexpovel/betterletters/commit/e0b097a692735257c9dce063682b662b46ebb0ed))
* naive mode for German ([cb3357c](https://github.com/alexpovel/betterletters/commit/cb3357c21aae80735c1986e89564f924f27e0e83))
* replacement stage ([8886880](https://github.com/alexpovel/betterletters/commit/88868805b8cafd7770f1252a9ce10986fa82cec5))
* Support upper/mixed case special characters ([90111da](https://github.com/alexpovel/betterletters/commit/90111da3fa69cb7fd105856608acf1afd9a05a49)), closes [#5](https://github.com/alexpovel/betterletters/issues/5)
* TypeScript, with TODO app example ([3d3ed21](https://github.com/alexpovel/betterletters/commit/3d3ed21582cc91d76e62bc8729dbeb38f70ebfb9))
* uppercase stage ([c92ad21](https://github.com/alexpovel/betterletters/commit/c92ad21584c875aa196ac18dea8f845bb610b4be))
* Use `decompound` ([c640363](https://github.com/alexpovel/betterletters/commit/c64036351fc7d0ea32b89f4744de97c04d8c39fe))
* Use `once_cell` to build `fst::Set` only once ([3eae2b6](https://github.com/alexpovel/betterletters/commit/3eae2b6408feea8c824e13a3a718011a3f9326a1))
* Verbosity switch ([8c05d69](https://github.com/alexpovel/betterletters/commit/8c05d69451be0fd072ee4e62b818d0f8206e5d41))
* Word list performance increased (now a single &str) ([32716fa](https://github.com/alexpovel/betterletters/commit/32716fae902bb8c744b3898b73437e375539d469))
* Working tree-sitter for language scoping ([bf17589](https://github.com/alexpovel/betterletters/commit/bf17589f782aeab41e61f51044999116065b3a74))


### Bug Fixes

* Add `aufwändig` to German word list ([69a138b](https://github.com/alexpovel/betterletters/commit/69a138b050eea0a1e128e9d47543db68da13601d))
* **ci:** cargo-tarpaulin v0.27 broke 'Xml' ([df585a2](https://github.com/alexpovel/betterletters/commit/df585a2e839605d4f39e32e5c6e51ce5c473146e))
* **ci:** Update all dependencies ([928a6d5](https://github.com/alexpovel/betterletters/commit/928a6d5d219b7cee6c9698d9942fca7fb653550a))
* Drop custom `Span`, use `std::ops::Range` ([fad059e](https://github.com/alexpovel/betterletters/commit/fad059e2f07b80581dee3c71fc9ac48fa4398fd4))
* Fix tests after project rename ([f5d4f17](https://github.com/alexpovel/betterletters/commit/f5d4f1787148ddec5786f5a8c14d572624ac2873))
* Make initial/missing `Transition` an unrepresentable state ([ba2948d](https://github.com/alexpovel/betterletters/commit/ba2948de94205b6729a02b57f611d3a287138387))
* Squeezing now has `tr`-like `squeeze-repeats` alias ([4d67a45](https://github.com/alexpovel/betterletters/commit/4d67a458171a237b3b782753e140a564ed7f84d2))
* **test:** Symbols stage is *not* fully idempotent ([5ff9277](https://github.com/alexpovel/betterletters/commit/5ff92773a01435db60b78c8c3e819533cacbfcdb))


### Performance Improvements

* Add profiling tooling (as a Justfile, for automation) ([9503761](https://github.com/alexpovel/betterletters/commit/9503761dba51fd36d5dd1fd77937c4cc133f624c)), closes [#14](https://github.com/alexpovel/betterletters/issues/14)
