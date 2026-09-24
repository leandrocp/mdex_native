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

Add Lumis to your deps, which supplies the parsers:

```elixir
{:lumis, "~> 0.9"}
```

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

### Parsers load on demand

A parser is a WebAssembly module fetched and compiled the first time a language
is rendered, not a grammar compiled into this library. MDExNative and the
`:lumis` application share one store, so whichever one fetches a parser first,
both use it.

That first render pays a download and a Wasmtime compile. Warm the languages a
deployment renders:

```elixir
Lumis.Languages.load("elixir")
```

MDExNative calls the same Lumis runtime, so the warmed parser serves both.

Rendering raises when a parser cannot be obtained, and the whole document fails
with the reason Lumis gave. An unknown language name is not that case: it
renders as plain text, as it always has.

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

Bundle size depends on the selected highlighter. `:lumis` builds a small NIF
because the highlighter itself lives in the `:lumis` package's NIF, which that
configuration also downloads:

| Config | This NIF | `:lumis` NIF | Delivered |
| --- | ---: | ---: | ---: |
| `syntax_highlighter: :lumis` | 2 MB | 4.7 MB | 6.7 MB |
| `syntax_highlighter: :syntect` | 3 MB | - | 3 MB |
| `syntax_highlighter: nil` | 1.2 MB | - | 1.2 MB |

Compressed, as downloaded. Lumis parsers are fetched on demand on top of that,
one per language rendered.

## Legacy CPUs

Modern CPU features are enabled by default. If your environment has an older
CPU, use legacy artifacts:

```elixir
config :mdex_native, use_legacy_artifacts: true
```
