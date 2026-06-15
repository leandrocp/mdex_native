# MDExNative

Markdown Elixir Native.

Used by:
- [MDEx](https://github.com/leandrocp/mdex)
- [NimblePublisher](https://github.com/dashbitco/nimble_publisher)

It uses the following Rust crates:

- [`comrak`](https://github.com/kivikakk/comrak) for Markdown parsing and rendering
- [`ammonia`](https://github.com/rust-ammonia/ammonia) for HTML sanitization
- [`lumis`](https://lumis.sh) or [`syntect`](https://crates.io/crates/syntect)/[`two-face`](https://crates.io/crates/two-face) for syntax highlighting

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

Guides:

- [Sanitization](guides/sanitization.md)
- [Syntax highlighting](guides/syntax_highlighting.md)

## Development

```sh
export MDEX_NATIVE_BUILD=1
mix setup
mix test
```

## Packages

### [MDExNative.Comrak](https://mdex-native.hexdocs.pm/MDExNative.Comrak.html)

Markdown parsing and rendering.

```elixir
html = MDExNative.Comrak.markdown_to_html("# Hello")
```

Comrak options are accepted as keyword lists. See [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html). MDExNative also accepts `:sanitize` and `:syntax_highlight`.

```elixir
html = MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
```

It also exposes XML, CommonMark, AST parsing, and heading anchor helpers. See the moduledoc.

### [MDExNative.Ammonia](https://mdex-native.hexdocs.pm/MDExNative.Ammonia.html)

HTML sanitization.

```elixir
html = ~s|<script>alert("xss")</script><p>Hello <strong>MDEx</strong></p>|

MDExNative.Ammonia.safe_html(html)
#=> "<p>Hello <strong>MDEx</strong></p>"
```
