defmodule MDExNative.AmmoniaTest do
  use ExUnit.Case

  doctest MDExNative.Ammonia

  test "sanitizes html with default options" do
    html = ~s|<script>alert("xss")</script><p>Hello <strong>MDEx</strong></p>|

    assert MDExNative.Ammonia.safe_html(html) == "<p>Hello <strong>MDEx</strong></p>"
  end

  test "escapes curly braces in code by default" do
    html = ~s|<p><code>{:ok, :mdex}</code></p>|

    assert MDExNative.Ammonia.safe_html(html) == "<p><code>&lbrace;:ok, :mdex&rbrace;</code></p>"

    assert MDExNative.Ammonia.safe_html(html, escape_curly_braces_in_code: false) ==
             "<p><code>{:ok, :mdex}</code></p>"
  end

  test "validates keyword options" do
    assert_raise ArgumentError, ~r/unknown keys \[:unknown\]/, fn ->
      MDExNative.Ammonia.safe_html("<p>ok</p>", unknown: true)
    end
  end
end
