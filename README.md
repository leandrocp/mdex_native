# MDExNative

Markdown Elixir Native.

Used by:
- [MDEx](https://github.com/leandrocp/mdex)
- [NimblePublisher](https://github.com/dashbitco/nimble_publisher)

It uses the following Rust crates:

- [`comrak`](https://github.com/kivikakk/comrak) for Markdown parsing and rendering
- [`ammonia`](https://github.com/rust-ammonia/ammonia) for HTML sanitization
- [`lumis`](https://github.com/leandrocp/lumis) and [`syntect`](https://github.com/trishume/syntect) for syntax highlighting
- [`two-face`](https://crates.io/crates/two-face) for extra Syntect syntax and theme definitions

Most applications should use `MDEx` directly to benefit from plugins, Document AST, Phoenix LiveView integration, streaming, additional syntax highlighting features, extra formats, MD sigil, and more.

But this project offers direct access to underlying Rust crates when you don't need all those features, or need a bit more performance, or less dependencies.

## Installation

Add `:mdex_native` to your dependencies:

```elixir
def deps do
  [
    {:mdex_native, "~> 0.1"}
  ]
end
```

## Quickstart

See all [examples](https://github.com/leandrocp/mdex_native/tree/main/examples).

## Development

```sh
export MDEX_NATIVE_BUILD=1
mix setup
mix test
```

## Packages

### MDExNative.Comrak

Markdown parsing and rendering.

```elixir
html = MDExNative.Comrak.markdown_to_html("# Hello")
```

Comrak options are accepted as keyword lists. See [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html).

```elixir
html = MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
```

It also exposes XML, CommonMark, AST parsing, and heading anchor helpers.

```elixir
xml = MDExNative.Comrak.markdown_to_xml("# Hello", render: [sourcepos: true])
anchor = MDExNative.Comrak.anchorize("Hello World")
```

#### Syntax Highlighting

Syntax highlighting of code blocks is enabled with the `:syntax_highlight` option. MDExNative supports two engines:

- `:lumis` - uses [`lumis`](https://lumis.sh)
- `:syntect` - uses Comrak's Syntect adapter with [`two-face`](https://crates.io/crates/two-face) syntax and theme definitions

**Lumis** example:

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

All Lumis formatters and options can be found on [Lumis formatter docs](https://lumis.hexdocs.pm/Lumis.html#t:formatter/0).

Precompiled NIFs do not include a syntax highlighter unless you opt in. This is the default:

```elixir
config :mdex_native, syntax_highlighter: nil
```

With syntax highlighting disabled, fenced code blocks still render as code blocks. The code is left unchanged, and Comrak keeps the language class from the fence.

To use Lumis:

```elixir
config :mdex_native, syntax_highlighter: :lumis
```

To use Syntect instead:

```elixir
config :mdex_native, syntax_highlighter: :syntect
```

Accepted values are `:lumis`, `:syntect`, and `nil`.

Bundle size might affect which syntax highlighter to use. Currently you can expect:

| Config | Compressed artifact size |
| --- | ---: |
| `syntax_highlighter: :lumis` | 15 MB |
| `syntax_highlighter: :syntect` | 3 MB |
| `syntax_highlighter: nil` | - |

### Legacy CPUs

Modern CPU features are enabled by default in `:mdex_native`. If your environment has an older CPU, use legacy artifacts:

```elixir
config :mdex_native, use_legacy_artifacts: true
```

**Syntect** example:

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

Note that `:syntax_highlight` is not a built-in Comrak option but it was added in MDExNative for convenience.

Omit or pass `syntax_highlight: nil` to disable syntax highlighting.


### MDExNative.Ammonia

HTML sanitization.

```elixir
html = ~s|<script>alert("xss")</script><p>Hello <strong>MDEx</strong></p>|

MDExNative.Ammonia.safe_html(html)
#=> "<p>Hello <strong>MDEx</strong></p>"
```
