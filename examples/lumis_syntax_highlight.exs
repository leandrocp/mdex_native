Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

markdown = """
```elixir
IO.puts("Hello from Lumis")
```
"""

options = [syntax_highlight: MDExNative.Lumis.default_options()]

markdown
|> MDExNative.Comrak.markdown_to_html(options)
|> IO.puts()
