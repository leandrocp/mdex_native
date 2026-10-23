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

Each selected revision runs with its original test expectations. Changes in upstream
output or behavior fail the compatibility check; the harness does not patch released tests.

To verify that the latest Hex release loads its precompiled artifact through the Cloudflare mirror, run after the package has been released and uploaded to R2:

```sh
MDEX_NATIVE_E2E_CLOUDFLARE=1 mix test --only cloudflare
```
