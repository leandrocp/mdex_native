Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

html = ~s|<script>alert("xss")</script><p>Hello <strong>MDEx</strong></p>|

html
|> MDExNative.Ammonia.safe_html()
|> IO.puts()
