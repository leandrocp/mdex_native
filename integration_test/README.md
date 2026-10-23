# MDExNative Integration Tests

This Mix project runs integration tests that compile `mdex_native` under different compile-time native feature configurations.

Run from this directory:

```sh
mix test
```

The tests create isolated temporary projects under `tmp/` so feature variants like default, Lumis, and Syntect do not share Mix build artifacts.

The downstream job runs the full MDEx suite against this native checkout.
`MDEX_NATIVE_E2E_MDEX_REF` selects a branch or tag; without it, the repository's
default branch is used. CI currently selects the coordinated MDEx #449 branch,
`lp-drop-lumis-dependency`. Remove that override after #449 lands.

The selected revision's tests run unchanged. This checks the pair being released,
not compatibility between a new native implementation and every older MDEx release.

To verify that the latest Hex release loads its precompiled artifact through the Cloudflare mirror, run after the package has been released and uploaded to R2:

```sh
MDEX_NATIVE_E2E_CLOUDFLARE=1 mix test --only cloudflare
```
