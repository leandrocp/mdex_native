# Changelog

## [0.2.2](https://github.com/leandrocp/mdex_native/compare/v0.2.1...v0.2.2) (2026-06-19)


### Features

* add `MDExNative.Comrak.dangerous_url?` ([#24](https://github.com/leandrocp/mdex_native/issues/24)) ([d05d02b](https://github.com/leandrocp/mdex_native/commit/d05d02bc954d5f869c2619c442f46da0bca70757))


### Documentation

* expose changelog ([487cb1c](https://github.com/leandrocp/mdex_native/commit/487cb1c6a32ba51b1fc9ee8cabe666bb46c73cfb))

## [0.2.1](https://github.com/leandrocp/mdex_native/compare/v0.2.0...v0.2.1) (2026-06-15)


### Features

* expose sanitization options ([#19](https://github.com/leandrocp/mdex_native/issues/19)) ([45b7d0f](https://github.com/leandrocp/mdex_native/commit/45b7d0f37e95cf49caa99eca6c4630b2dcb68b6e))

## [0.2.0](https://github.com/leandrocp/mdex_native/compare/v0.1.5...v0.2.0) (2026-06-12)


### ⚠ BREAKING CHANGES

* adopt the `l-` prefix on token class https://github.com/leandrocp/lumis/pull/952

### Features

* update lumis ([#17](https://github.com/leandrocp/mdex_native/issues/17)) ([b285d88](https://github.com/leandrocp/mdex_native/commit/b285d88530d17ae3d969ae191b05f27d02826a8d))

## [0.1.5](https://github.com/leandrocp/mdex_native/compare/v0.1.4...v0.1.5) (2026-06-08)


### Features

* choose syntax highlighter at compile-time ([#16](https://github.com/leandrocp/mdex_native/issues/16)) ([c4aecc2](https://github.com/leandrocp/mdex_native/commit/c4aecc2fa1288613f728a7f041a32d9669f9a67b))


### Documentation

* project description ([8a240a2](https://github.com/leandrocp/mdex_native/commit/8a240a287d9c494c5b390dffe1e73621a326bd22))

## [0.1.4](https://github.com/leandrocp/mdex_native/compare/v0.1.3...v0.1.4) (2026-06-06)


### Features

* add syntect syntax highlighter ([#12](https://github.com/leandrocp/mdex_native/issues/12)) ([57b7619](https://github.com/leandrocp/mdex_native/commit/57b76192eee2f44642c7c968495a2a0be02cfa87))

## [0.1.3](https://github.com/leandrocp/mdex_native/compare/v0.1.2...v0.1.3) (2026-06-06)


### Features

* explicit :syntax_highlight with :engine ([#11](https://github.com/leandrocp/mdex_native/issues/11)) ([b334d35](https://github.com/leandrocp/mdex_native/commit/b334d3529f9c34ecf166b34c0086be089e93305b))


### Documentation

* main page ([9ca4ae9](https://github.com/leandrocp/mdex_native/commit/9ca4ae9aa1f63c9e9ed1bdc1f2afc8e849cc9b2d))
* Update documentation for Comrak options mapping ([#10](https://github.com/leandrocp/mdex_native/issues/10)) ([0811ce8](https://github.com/leandrocp/mdex_native/commit/0811ce8c08c0ac04b5e43ceede76c27c11262af3)) by @josevalim

## [0.1.2](https://github.com/leandrocp/mdex_native/compare/v0.1.1...v0.1.2) (2026-06-05)


### Bug Fixes

* use From trait for Sourcepos ([#7](https://github.com/leandrocp/mdex_native/issues/7)) ([43a7424](https://github.com/leandrocp/mdex_native/commit/43a74243d7c1aa314f650d28d8adb2258d12b78e))

## [0.1.1](https://github.com/leandrocp/mdex_native/compare/v0.1.0...v0.1.1) (2026-06-05)


### Bug Fixes

* **deps:** optional rustler ([ca2b873](https://github.com/leandrocp/mdex_native/commit/ca2b873052bd8b66cd8c54bdb46f4fcf15460c80))

## 0.1.0 (2026-06-04)

### Features

- Initial release of `MDExNative`.
- Add Comrak Markdown parsing and rendering through `MDExNative.Comrak`.
- Add Ammonia HTML sanitization through `MDExNative.Ammonia`.
- Add Lumis syntax highlighting support through the native `LumisAdapter`.
