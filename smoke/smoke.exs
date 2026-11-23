markdown =
  MDExNative.Comrak.markdown_to_html!("# Smoke\n\n```elixir\ndefmodule Smoke do\nend\n```\n",
    syntax_highlight: [
      engine: :lumis,
      opts: [formatter: {:html_inline, include_highlights: true}]
    ]
  )

if not String.contains?(markdown, "<h1>Smoke</h1>") or
     not String.contains?(markdown, ~s(data-highlight="keyword)) do
  raise "MDExNative did not render highlighted Markdown: #{markdown}"
end

html =
  Lumis.highlight!("defmodule Smoke do\nend\n",
    formatter: {:html_inline, language: "elixir", include_highlights: true}
  )

if not String.contains?(html, ~s(data-highlight="keyword)) do
  raise "Lumis did not highlight Elixir: #{html}"
end

IO.puts("smoke ok")
