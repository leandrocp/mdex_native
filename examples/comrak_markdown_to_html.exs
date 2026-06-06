Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

markdown = """
# Project Tasks

- [x] Extract parser
- [ ] Publish package

Visit https://github.com/leandrocp/mdex_native
"""

options = [extension: [tasklist: true, autolink: true, header_id_prefix: "example-"]]

markdown
|> MDExNative.Comrak.markdown_to_html(options)
|> IO.puts()
