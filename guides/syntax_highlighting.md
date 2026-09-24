# Syntax highlighting

Syntax highlighting can be enabled with the `:syntax_highlight` option,
and it's disabled by default.

MDExNative supports two engines:

- `:lumis` - uses [`Lumis`](https://lumis.sh)
- `:syntect` - uses [Syntect](https://crates.io/crates/syntect) with [`two-face`](https://crates.io/crates/two-face)

It's disable by default, fenced code blocks still render as code
blocks keeping the code content unchanged, and the language name
is added on the `<pre>` class.

## Lumis

Add Lumis to your deps, along with a parser for every language you highlight:

```elixir
{:lumis, "~> 0.9"},
{:lumis_wasm_rust, "~> 0.26"},
{:lumis_wasm_elixir, "~> 0.26"}
```

The full list is at [docs.lumis.sh/languages](https://docs.lumis.sh/languages).

Configure MDExNative before compiling dependencies:

```elixir
config :mdex_native, syntax_highlighter: :lumis
```

Then pass `syntax_highlight` when rendering:

````elixir
markdown = """
```rust
fn main() {
    println!("Hello from Lumis");
}
```
"""

html = MDExNative.Comrak.markdown_to_html(markdown,
  syntax_highlight: [
    engine: :lumis,
    opts: [
      formatter: {:html_inline, theme: "catppuccin_macchiato"}
    ]
  ]
)
````

Lumis formatters and options are documented in [`Lumis`](https://lumis.hexdocs.pm/Lumis.html#t:formatter/0).

### Parsers are dependencies

A parser is a WebAssembly module shipped as a `lumis_wasm_*` package, not a
grammar compiled into this library. Depending on one is how a project declares
it may render that language; nothing is downloaded at runtime.

MDExNative reads the same installed parsers and the same compiled-module cache
as the `:lumis` application, so a parser is installed and compiled once for the
whole VM rather than once per NIF.

A fence naming a language the project does not depend on renders as plain text.
It costs that one fence, not the document.

The first render of a language pays a Wasmtime compile. Warm the ones a
deployment renders:

```elixir
MDExNative.load_language("elixir")
```

`Lumis.Languages.load/1` warms Lumis's own runtime. The two keep separate
runtimes and share the on-disk compile cache, so warming both is worthwhile and
the second call is the cheaper one.

## Syntect

Configure MDExNative before compiling dependencies:

```elixir
config :mdex_native, syntax_highlighter: :syntect
```

Then pass a Syntect theme:

````elixir
markdown = """
```rust
fn main() {
    println!("Hello from Syntect");
}
```
"""

html = MDExNative.Comrak.markdown_to_html(markdown,
  syntax_highlight: [
    engine: :syntect,
    opts: [theme: "Catppuccin Macchiato"]
  ]
)
````

Syntect theme names come from [`two-face`](https://crates.io/crates/two-face).

## Artifact size

Bundle size depends on the selected highlighter:

| Config | Compressed artifact size |
| --- | ---: |
| `syntax_highlighter: :lumis` | 5 MB |
| `syntax_highlighter: :syntect` | 3 MB |
| `syntax_highlighter: nil` | - |

## Legacy CPUs

Modern CPU features are enabled by default. If your environment has an older
CPU, use legacy artifacts:

```elixir
config :mdex_native, use_legacy_artifacts: true
```
