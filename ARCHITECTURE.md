# Architecture

mdex_native is a single [Rustler] NIF, [`native/mdex_native_nif`][nif], loaded by [`MDExNative.Native`][native.ex]. It parses and renders Markdown with [comrak], can highlight code fences with [Lumis] or [syntect], and can sanitize the HTML with [ammonia].

```text
Markdown, or an MDEx document term
  -> comrak parses it into an AST (attributes, shortcodes, phoenix_heex)
  -> MdexFormatter writes comrak's HTML, plus node attributes on links and images
       code fences -> the build's highlighter adapter, if it has one
                        lumis:   lumis_adapter -> lumis_runtime -> lumis_render
                        syntect: comrak's SyntectAdapter with two-face syntaxes and themes
  -> ammonia, when :sanitize is set
  -> lol_html escapes curly braces inside code, when phoenix_heex is on
  -> HTML
```

`MdexFormatter` is in [`html_formatter.rs`][html_formatter], the render entry points and the ammonia and [lol_html] steps in [`lib.rs`][lib.rs]. See the [sanitization] and [syntax highlighting] guides for the options.

## Build variants

`config :mdex_native, syntax_highlighter: :lumis | :syntect | nil` is read at compile time in [`native.ex`][native.ex]. It turns on the matching Cargo feature in [`Cargo.toml`][cargo] and picks the [rustler_precompiled] artifact: `--lumis`, `--syntect` or plain, each with a `legacy_cpu` build. A NIF has at most one highlighter. Asking for the other one at render time returns `:lumis_not_enabled` or `:syntect_not_enabled`.

`MDEX_NATIVE_BUILD=1` compiles the NIF from source instead of downloading it.

## Lumis

The `lumis` feature compiles [`lumis-core`][lumis-core] and [`lumis-wasm-runtime`][lumis-wasm-runtime] into the NIF. Parsers are not compiled in. They are WASM modules shipped in `lumis_wasm_*` dependencies.

[`MDExNative.Application`][application.ex] asks the `:lumis` application for its data directory ([`Lumis.Application.data_dir/0`][lumis-data-dir]), collects `priv/parsers` from every `lumis_wasm_*` application on the code path, and hands both to the NIF. It does this at boot and again before the first render that highlights, because templates can be rendered during `mix compile`, when no application is running.

A fence goes through [`lumis_adapter.rs`][lumis_adapter], which implements comrak's [`SyntaxHighlighterAdapter`][adapter-trait], then [`lumis_runtime.rs`][lumis_runtime], which turns the source into highlight events, then [`lumis_render.rs`][lumis_render], which applies MDEx's info-string decorators and writes the HTML with a lumis-core formatter. Highlighting runs on the runtime's own threads with 8 MiB stacks, since nested injections recurse once per layer and overflow a dirty scheduler's stack.

### Two runtimes when `:lumis` is installed

An application that also highlights with `:lumis` runs two Lumis runtimes, one in `lumis_nif` and one here.

They share files: the parser packages and the compiled-module cache under the Lumis data directory. A parser compiled by one is not compiled again by the other.

They don't share memory. Each NIF has its own [wasmtime] engine and Tree-sitter [`WasmStore`][wasmstore], so a language used by both is loaded twice, and each store has its own `store_full` limit. With lumis 0.10.0 and mdex_native 0.2.10 on macOS arm64, the first language costs about 15 MB and 120 ms per NIF, and each additional language about 2.7 MB and 13 ms.

This is on purpose. comrak calls the adapter synchronously inside the NIF call, and a NIF can't call back into Elixir halfway through, so the highlighter has to live in this library. Sharing one runtime would need either a native contract between the two NIFs ([`enif_dynamic_resource_call`][dyncall] or a C API) or fence highlighting moved out of the render and into Elixir. At these numbers, neither pays for itself.

## syntect

The `syntect` feature uses comrak's [`SyntectAdapter`][syntect-adapter] with the syntax definitions and themes from [two-face], all compiled into the NIF. There is no runtime setup, no parser package and no data directory. The set of supported languages is fixed when the NIF is built.

[Rustler]: https://github.com/rusterlium/rustler
[rustler_precompiled]: https://github.com/philss/rustler_precompiled
[comrak]: https://github.com/kivikakk/comrak
[adapter-trait]: https://docs.rs/comrak/latest/comrak/adapters/trait.SyntaxHighlighterAdapter.html
[syntect-adapter]: https://docs.rs/comrak/latest/comrak/plugins/syntect/struct.SyntectAdapter.html
[ammonia]: https://github.com/rust-ammonia/ammonia
[lol_html]: https://github.com/cloudflare/lol-html
[Lumis]: https://lumis.sh
[lumis-core]: https://docs.rs/lumis-core
[lumis-wasm-runtime]: https://docs.rs/lumis-wasm-runtime
[lumis-data-dir]: https://github.com/leandrocp/lumis/blob/hex-lumis/v0.10.0/packages/elixir/lumis/lib/lumis/application.ex
[wasmtime]: https://wasmtime.dev
[wasmstore]: https://docs.rs/tree-sitter/latest/tree_sitter/struct.WasmStore.html
[dyncall]: https://www.erlang.org/doc/apps/erts/erl_nif.html#enif_dynamic_resource_call
[syntect]: https://github.com/trishume/syntect
[two-face]: https://github.com/CosmicHorrorDev/two-face
[nif]: native/mdex_native_nif
[cargo]: native/mdex_native_nif/Cargo.toml
[lib.rs]: native/mdex_native_nif/src/lib.rs
[html_formatter]: native/mdex_native_nif/src/html_formatter.rs
[lumis_adapter]: native/mdex_native_nif/src/lumis_adapter.rs
[lumis_runtime]: native/mdex_native_nif/src/lumis_runtime.rs
[lumis_render]: native/mdex_native_nif/src/lumis_render.rs
[native.ex]: lib/mdex_native/native.ex
[application.ex]: lib/mdex_native/application.ex
[sanitization]: guides/sanitization.md
[syntax highlighting]: guides/syntax_highlighting.md
