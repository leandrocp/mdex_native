# MDExNative Integration Tests

This Mix project runs integration tests that compile `mdex_native` under different compile-time native feature configurations.

Run from this directory:

```sh
mix test
```

The tests create isolated temporary projects under `tmp/` so feature variants like default, Lumis, and Syntect do not share Mix build artifacts.

To verify that the latest Hex release loads its precompiled artifact through the Cloudflare mirror, run after the package has been released and uploaded to R2:

```sh
MDEX_NATIVE_E2E_CLOUDFLARE=1 mix test --only cloudflare
```
