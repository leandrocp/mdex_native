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

  test "accepts sanitize keyword options" do
    assert MDExNative.Ammonia.safe_html("<h1>Title</h1><p>Content</p>",
             sanitize: [rm_tags: ["h1"]]
           ) == "Title<p>Content</p>"
  end

  test "accepts sanitize atoms" do
    custom = {:custom, %{tags: %{set: ["p"]}}}

    assert MDExNative.Sanitize.normalize(:default) == :clean
    assert MDExNative.Sanitize.normalize(custom) == custom

    assert MDExNative.Ammonia.safe_html("<script>bad()</script>", sanitize: nil) ==
             "<script>bad()</script>"

    assert MDExNative.Ammonia.safe_html("<script>bad()</script>", sanitize: :clean) == ""
  end

  test "filters style properties" do
    assert MDExNative.Ammonia.safe_html(
             ~s|<p style="font-weight: heavy; color: red">Content</p>|,
             sanitize: [add_generic_attributes: ["style"], filter_style_properties: ["color"]]
           ) == ~s|<p style="color:red">Content</p>|
  end

  test "validates keyword options" do
    assert_raise ArgumentError, ~r/unknown keys \[:unknown\]/, fn ->
      MDExNative.Ammonia.safe_html("<p>ok</p>", unknown: true)
    end
  end
end
