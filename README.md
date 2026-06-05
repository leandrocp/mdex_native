# MDExNative

Native foundation for [MDEx](https://github.com/leandrocp/mdex)

It wraps the following Rust crates:

- [`comrak`](https://github.com/kivikakk/comrak) for Markdown parsing and rendering
- [`ammonia`](https://github.com/rust-ammonia/ammonia) for HTML sanitization
- [`lumis`](https://lumis.sh) for syntax highlighting

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

Precompiled NIFs are used by default. To build the NIF locally:

```sh
MDEX_NATIVE_BUILD=1 mix compile
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

### MDExNative.Ammonia

HTML sanitization.

```elixir
html = ~s|<script>alert("xss")</script><p>Hello <strong>MDEx</strong></p>|

MDExNative.Ammonia.safe_html(html)
#=> "<p>Hello <strong>MDEx</strong></p>"
```

### MDExNative.Lumis

Syntax highlighting options for Comrak rendering.
The native Rust `LumisAdapter` implements Comrak's `SyntaxHighlighterAdapter`
and is installed when `:syntax_highlight` options are passed to `MDExNative.Comrak`.

````elixir
markdown = """
```elixir
IO.puts("Hello from Lumis")
```
"""

options = [syntax_highlight: MDExNative.Lumis.default_options()]

html = MDExNative.Comrak.markdown_to_html(markdown, options)
````

## Examples

Run the examples from the project root:

```sh
elixir examples/markdown_to_html.exs
elixir examples/ammonia_safe_html.exs
# others...
```
