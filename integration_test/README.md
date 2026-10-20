# MDExNative Integration Tests

This Mix project runs integration tests that compile `mdex_native` under different compile-time native feature configurations.

Run from this directory:

```sh
mix test
```

The tests create isolated temporary projects under `tmp/` so feature variants like default, Lumis, and Syntect do not share Mix build artifacts.

The downstream job runs the full MDEx suites selected by `MDEX_NATIVE_E2E_MDEX_REFS`.
While the standalone Lumis stack is unmerged, CI selects `lp-drop-lumis-dependency`
and the latest release. Switch the development ref back to `main` after MDEx #449 lands.

The v0.14.2 suite at commit `7aba3ae57b235a986d419de3918cd5c1fd5b4edc` receives
[`fixtures/mdex-v0.14.2-expectations.patch`](fixtures/mdex-v0.14.2-expectations.patch)
before running. It changes only two expectations for intentional upstream fixes:

- [Lumis #1748](https://github.com/leandrocp/lumis/pull/1748) uses actual theme names,
  not the `light`/`dark` aliases, as multi-theme HTML classes. The aliases still name CSS variables.
- [Comrak's table-count fix](https://github.com/kivikakk/comrak/commit/b20f2768986b055555a256ac0f194a88a47700ca)
  excludes generated padding cells. MDEx main already adopted the corrected expectation in
  [#454](https://github.com/leandrocp/mdex/pull/454).

The exact release commit and patch applicability are checked before execution.
All tests still run; production code and test inputs remain unchanged. Other refs,
including future releases selected by `latest`, run without this patch. This verifies
released MDEx compatibility with those corrections, not byte-identical historical output.

To verify that the latest Hex release loads its precompiled artifact through the Cloudflare mirror, run after the package has been released and uploaded to R2:

```sh
MDEX_NATIVE_E2E_CLOUDFLARE=1 mix test --only cloudflare
```
