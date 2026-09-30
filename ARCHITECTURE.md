# Architecture

mdex_native is a single Rustler NIF, `native/mdex_native_nif`, loaded by `MDExNative.Native`. It parses and renders Markdown with comrak, can highlight code fences with Lumis or syntect, and can sanitize the HTML with ammonia.

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

## Build variants

`config :mdex_native, syntax_highlighter: :lumis | :syntect | nil` is read at compile time. It turns on the matching Cargo feature and picks the precompiled artifact: `--lumis`, `--syntect` or plain, each with a `legacy_cpu` build. A NIF has at most one highlighter. Asking for the other one at render time returns `:lumis_not_enabled` or `:syntect_not_enabled`.

`MDEX_NATIVE_BUILD=1` compiles the NIF from source instead of downloading it.

## Lumis

The `lumis` feature compiles `lumis-core` and `lumis-wasm-runtime` into the NIF. Parsers are not compiled in. They are WASM modules shipped in `lumis_wasm_*` dependencies.

`MDExNative.Application` asks the `:lumis` application for its data directory (`Lumis.Application.data_dir/0`), collects `priv/parsers` from every `lumis_wasm_*` application on the code path, and hands both to the NIF. It does this at boot and again before the first render that highlights, because templates can be rendered during `mix compile`, when no application is running.

A fence goes through `lumis_adapter.rs`, which implements comrak's `SyntaxHighlighterAdapter`, then `lumis_runtime.rs`, which turns the source into highlight events, then `lumis_render.rs`, which applies MDEx's info-string decorators and writes the HTML with a lumis-core formatter. Highlighting runs on the runtime's own threads with 8 MiB stacks, since nested injections recurse once per layer and overflow a dirty scheduler's stack.

### Two runtimes when `:lumis` is installed

An application that also highlights with `:lumis` runs two Lumis runtimes, one in `lumis_nif` and one here.

They share files: the parser packages and the compiled-module cache under the Lumis data directory. A parser compiled by one is not compiled again by the other.

They don't share memory. Each NIF has its own wasmtime engine and Tree-sitter `WasmStore`, so a language used by both is loaded twice, and each store has its own `store_full` limit. With lumis 0.10.0 and mdex_native 0.2.10 on macOS arm64, the first language costs about 15 MB and 120 ms per NIF, and each additional language about 2.7 MB and 13 ms.

This is on purpose. comrak calls the adapter synchronously inside the NIF call, and a NIF can't call back into Elixir halfway through, so the highlighter has to live in this library. Sharing one runtime would need either a native contract between the two NIFs (`enif_dynamic_resource_call` or a C API) or fence highlighting moved out of the render and into Elixir. At these numbers, neither pays for itself.

## syntect

The `syntect` feature uses comrak's `SyntectAdapter` with the syntax definitions and themes from `two-face`, all compiled into the NIF. There is no runtime setup, no parser package and no data directory. The set of supported languages is fixed when the NIF is built.
