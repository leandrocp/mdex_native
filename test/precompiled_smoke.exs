# Installs this package's precompiled NIF next to lumis, as an MDEx app does
# (leandrocp/lumis#2114), and checks both highlight. Release CI runs it with
# RUSTLER_PRECOMPILED_GLOBAL_CACHE_PATH pointing at the NIFs it just built.
Mix.install(
  [
    {:mdex_native, path: Path.expand("..", __DIR__)},
    {:lumis, "~> 0.10"},
    {:lumis_wasm_elixir, "~> 0.26.0"}
  ],
  config: [mdex_native: [syntax_highlighter: :lumis]]
)

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
