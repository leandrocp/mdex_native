Mix.install(
  [
    {:mdex_native, path: Path.expand("..", __DIR__)},
    {:lumis, "~> 0.9"},
    {:lumis_wasm_markdown, "~> 0.26"},
    {:lumis_wasm_elixir, "~> 0.26"}
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

output = MDExNative.Comrak.markdown_to_html(markdown, options)

theme_css =
  "https://cdn.jsdelivr.net/gh/leandrocp/lumis@hex-lumis/v0.9.0/packages/elixir/lumis/priv/static/css/catppuccin_macchiato.css"

html = """
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Lumis html_linked</title>
    <link rel="stylesheet" href="#{theme_css}" />
    <style>
      .line-highlight {
        background-color: #303347;
      }
    </style>
  </head>
  <body>
#{output}
  </body>
</html>
"""

IO.puts(html)
