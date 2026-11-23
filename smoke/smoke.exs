markdown =
  MDExNative.Comrak.markdown_to_html!("# Smoke\n\n```elixir\n:ok\n```\n",
    syntax_highlight: [engine: :lumis, opts: [formatter: {:html_inline, theme: "github_light"}]]
  )

if not String.contains?(markdown, "<h1>Smoke</h1>") or not String.contains?(markdown, "<span") do
  raise "unexpected MDExNative output: #{markdown}"
end

html = Lumis.highlight!(":ok", formatter: {:html_inline, language: "elixir"})
if not String.contains?(html, "<span"), do: raise("unexpected Lumis output: #{html}")

IO.puts("smoke ok")
