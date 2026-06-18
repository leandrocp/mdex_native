# MDExNative Integration Tests

This Mix project runs integration tests that compile `mdex_native` under different compile-time native feature configurations.

Run from this directory:

```sh
mix test
```

The tests create isolated temporary projects under `tmp/` so feature variants like default, Lumis, and Syntect do not share Mix build artifacts.
