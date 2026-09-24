# Syntax highlighting

Syntax highlighting is off by default and turned on with the
`:syntax_highlight` option. Two engines are available:

- `:lumis` - uses [`Lumis`](https://lumis.sh)
- `:syntect` - uses [Syntect](https://crates.io/crates/syntect) with [`two-face`](https://crates.io/crates/two-face)

With it off, fenced code blocks still render as code blocks with their content
unchanged, and the language name goes on the `<pre>` class.

## Lumis

Add Lumis to your deps, along with a parser for every language you highlight:

```elixir
{:lumis, "~> 0.9"},
{:lumis_wasm_rust, "~> 0.26"},
{:lumis_wasm_elixir, "~> 0.26"}
```

The full list is at [docs.lumis.sh/reference/languages](https://docs.lumis.sh/reference/languages).

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

A parser is a WebAssembly module published as a
[`lumis_wasm_*`](https://hex.pm/packages?search=lumis_wasm_&sort=recent_downloads)
package. None are compiled into this library and none are fetched at runtime.
Want to highlight Elixir? Depend on `lumis_wasm_elixir`.

MDExNative reads those packages from the same place the `:lumis` application
does, and shares its cache of compiled parsers, so the VM pays for each one
once instead of once per NIF.

Name a language you haven't installed and that fence comes out as plain text.
The rest of the document is unaffected.

Loading happens on first use. That first fence pays for a Wasmtime compile,
usually a few hundred milliseconds; everything after it is fast. Calling
`Lumis.Languages.load/1` at startup wins about half of that back, since the
compiled module lands in the shared cache, though MDExNative still has to
register the parser with its own runtime.

Why lazy? Parsers are not free to keep in memory, and they vary a lot in size.
Loading everything you have installed, on the chance a document mentions it, is
usually the wrong trade. Per-language sizes are listed in
[docs.lumis.sh/reference/languages](https://docs.lumis.sh/reference/languages).

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
| `syntax_highlighter: nil` | 1 MB |

Parsers are not in those numbers. They arrive as their own Hex packages, so you
only download the languages you asked for.

## Legacy CPUs

Modern CPU features are enabled by default. If your environment has an older
CPU, use legacy artifacts:

```elixir
config :mdex_native, use_legacy_artifacts: true
```
