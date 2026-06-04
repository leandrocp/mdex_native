Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

anchor = MDExNative.Comrak.anchorize("MDExNative.Comrak Examples")

IO.puts("Anchor: #{anchor}")
