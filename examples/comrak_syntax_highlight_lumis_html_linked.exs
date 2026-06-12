Mix.install(
  [
    {:mdex_native, path: Path.expand("..", __DIR__)}
  ],
  config: [mdex_native: [syntax_highlighter: :lumis]]
)

markdown = ~S"""
# Lumis html_linked

```elixir highlight_lines="2" highlight_lines_class="line-highlight"
defmodule Example do
  def hello(name) do
    "Hello, #{name}!"
  end
end
```
"""

options = [
  render: [github_pre_lang: true, full_info_string: true],
  syntax_highlight: [
    engine: :lumis,
    opts: [
      formatter: {
        :html_linked,
        pre_class: "code-block-linked"
      }
    ]
  ]
]

markdown
|> MDExNative.Comrak.markdown_to_html(options)
|> IO.puts()
