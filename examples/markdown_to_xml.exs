Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

markdown = """
## Notes

Markdown can render to XML too.

> Source positions are useful for editor integrations.
"""

options = [render: [sourcepos: true]]

markdown
|> MDExNative.Comrak.markdown_to_xml(options)
|> IO.puts()
