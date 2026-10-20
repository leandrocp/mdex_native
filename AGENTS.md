# AGENTS.md

## Keep the Wasmtime compile cache aligned with Lumis

`leandrocp/lumis` and `leandrocp/mdex_native` build separate NIFs that share a
disk compilation cache, not an in-memory runtime. Wasmtime's default cache
namespace and keys include the full version; matching only the major version
does not preserve cache reuse.

- `lumis-wasm-runtime` owns the Wasmtime version. Pin it with an exact Cargo
  requirement (`=X.Y.Z`) in Lumis's `crates/lumis-wasm-runtime/Cargo.toml`.
- Pin the same exact `lumis-wasm-runtime` release in both NIF manifests:
  Lumis's `packages/elixir/lumis/native/lumis_nif/Cargo.toml` and mdex_native's
  `native/mdex_native_nif/Cargo.toml`. Do not upgrade either side independently.
- When changing either dependency, coordinate the change across both repositories
  and check the resolved Wasmtime version in all three lockfiles: Lumis's root
  `Cargo.lock`, its standalone NIF lockfile, and mdex_native's NIF lockfile.
  Commit the lockfiles and use `--locked` for release builds.
- Require CI to check those resolutions against the intended release pair, not
  just the two moving `main` branches. Check packaged Hex source builds as well
  as workspace builds; workspace dependency overrides do not follow a crate
  onto crates.io.
- Verify cache reuse with release builds, the same parser bytes and data directory,
  and compatible engine settings on the same target. Version equality alone is
  not proof of a cache hit or of sharing in-memory warm-up.
- Coordinate MDEx's supported Lumis/mdex_native package constraints when Wasmtime
  changes. Cargo cannot align independently precompiled NIFs after installation,
  and new pins do not change already-published artifacts.

Keep this policy in both repositories' `AGENTS.md`. Guidance is not enforcement;
report missing pins or CI checks rather than assuming this file guarantees them.
