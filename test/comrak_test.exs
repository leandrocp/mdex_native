defmodule MDExNative.ComrakTest do
  use ExUnit.Case

  doctest MDExNative.Comrak

  test "anchorizes text" do
    assert MDExNative.Comrak.anchorize("Hello World") == "hello-world"
  end

  test "renders markdown with Comrak options" do
    html = MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])

    assert html =~ ~s(<input type="checkbox" checked="" disabled="" /> done)
  end

  test "render functions return rendered strings" do
    assert MDExNative.Comrak.markdown_to_html("**bold**") == "<p><strong>bold</strong></p>\n"
    assert MDExNative.Comrak.markdown_to_xml("# Hello") =~ ~s(<heading level="1">)
  end
end
