Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

markdown = """
# My Code

```elixir
IO.puts("Hello from Lumis")
```
"""

options = [
  syntax_highlight: [
    engine: :lumis,
    opts: [formatter: {:html_inline, theme: "github_light", pre_class: "code-block-example"}]
  ]
]

markdown
|> MDExNative.Comrak.markdown_to_html(options)
|> IO.puts()
