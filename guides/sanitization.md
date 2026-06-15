# Sanitization

MDExNative has four relevant HTML modes:

- omit raw HTML from Markdown, the default Comrak behavior
- escape raw HTML with `render: [escape: true]`
- render raw HTML and sanitize it with Ammonia
- render raw HTML without sanitization with `render: [unsafe: true]`

Two libraries are involved:

- Comrak decides whether raw HTML from Markdown is rendered, escaped, or omitted.
- Ammonia sanitizes HTML after rendering.

That order matters. If Comrak omits raw HTML, Ammonia never sees it.

## Raw HTML is omitted by default

```elixir
md = ~S"""
# Release notes

<script>trackPageView()</script>

## Changes
"""

MDExNative.Comrak.markdown_to_html(md)
#=> "<h1>Release notes</h1>\n<!-- raw HTML omitted -->\n<h2>Changes</h2>\n"
```

## Escape raw HTML

`render: [escape: true]` emits the raw HTML as escaped text:

```elixir
MDExNative.Comrak.markdown_to_html("<h1>Hello</h1>", render: [escape: true])
#=> "&lt;h1&gt;Hello&lt;/h1&gt;\n"
```

## Sanitize raw HTML

To sanitize raw HTML in Markdown, render it first with `render: [unsafe: true]`:

```elixir
MDExNative.Comrak.markdown_to_html(
  ~s|<p>Hello</p><script>trackPageView()</script>|,
  render: [unsafe: true],
  sanitize: :clean
)

#=> "<p>Hello</p>\n"
```

`:clean` calls [`ammonia::clean`](https://docs.rs/ammonia/latest/ammonia/fn.clean.html).

This also works with custom sanitizer options:

```elixir
MDExNative.Comrak.markdown_to_html(
  "<h1>Title</h1><p>Content</p>",
  render: [unsafe: true],
  sanitize: [rm_tags: ["h1"]]
)

#=> "Title<p>Content</p>\n"
```

## Custom sanitizer options

Sanitizer options map to Ammonia builder operations. The base key replaces a set,
`add_*` appends to it, and `rm_*` removes from it.

Set allowed tags:

```elixir
MDExNative.Ammonia.safe_html(
  "<h1>Title</h1><p>Content</p>",
  sanitize: [tags: ["p"]]
)

#=> "Title<p>Content</p>"
```

Add a tag:

```elixir
MDExNative.Ammonia.safe_html(
  "<custom>Intro</custom>",
  sanitize: [add_tags: ["custom"]]
)

#=> "<custom>Intro</custom>"
```

Remove a tag:

```elixir
MDExNative.Ammonia.safe_html(
  "<h1>Title</h1><p>Content</p>",
  sanitize: [rm_tags: ["h1"]]
)

#=> "Title<p>Content</p>"
```

Combine operations:

```elixir
MDExNative.Ammonia.safe_html(
  ~s|<h1>Title</h1><section data-kind="note" onclick="x">Content</section>|,
  sanitize: [
    add_tags: ["section"],
    add_tag_attributes: %{"section" => ["data-kind"]},
    rm_tags: ["h1"]
  ]
)

#=> ~s|Title<section data-kind="note">Content</section>|
```

## Render raw HTML without sanitization

If the input is trusted, render raw HTML without sanitizing:

```elixir
MDExNative.Comrak.markdown_to_html("<script>hello</script>", render: [unsafe: true])
#=> "<script>hello</script>\n"
```
