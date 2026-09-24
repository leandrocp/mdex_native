Mix.install(
  [
    {:mdex_native, path: Path.expand("..", __DIR__)},
    {:lumis, "~> 0.9"},
    {:lumis_wasm_markdown, "~> 0.26"},
    {:lumis_wasm_rust, "~> 0.26"}
  ],
  config: [mdex_native: [syntax_highlighter: :lumis]]
)

markdown = """
# Lumis

```rust
impl<T> Option<T> {
    #[must_use]
    #[inline]
    pub const fn is_some_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            None => false,
            Some(x) => f(x),
        }
    }
}
```
"""

options = [
  syntax_highlight: [
    engine: :lumis,
    opts: [
      formatter: {:html_inline, theme: "catppuccin_macchiato", pre_class: "code-block-example"}
    ]
  ]
]

output = MDExNative.Comrak.markdown_to_html(markdown, options)
IO.puts(output)
