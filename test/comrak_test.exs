defmodule MDExNative.ComrakTest do
  use ExUnit.Case

  @code_block_markdown """
  ```elixir
  IO.puts("Hello")
  ```
  """

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

  test "raises when lumis is requested but no syntax highlighter is compiled" do
    error =
      assert_raise RuntimeError, fn ->
        MDExNative.Comrak.markdown_to_html(@code_block_markdown,
          syntax_highlight: [
            engine: :lumis,
            opts: [
              formatter:
                {:html_inline, theme: "catppuccin_macchiato", pre_class: "code-block-example"}
            ]
          ]
        )
      end

    assert error.message =~ "Lumis is not enabled."
    assert error.message =~ "config :mdex_native, syntax_highlighter: :lumis"
  end

  test "raises when syntect is requested but no syntax highlighter is compiled" do
    error =
      assert_raise RuntimeError, fn ->
        MDExNative.Comrak.markdown_to_html(@code_block_markdown,
          syntax_highlight: [engine: :syntect]
        )
      end

    assert error.message =~ "Syntect is not enabled."
    assert error.message =~ "config :mdex_native, syntax_highlighter: :syntect"
  end

  test "does not syntax highlight when syntax_highlight is absent" do
    assert MDExNative.Comrak.markdown_to_html(@code_block_markdown) ==
             "<pre><code class=\"language-elixir\">IO.puts(&quot;Hello&quot;)\n</code></pre>\n"
  end

  test "does not syntax highlight when syntax_highlight is nil" do
    assert MDExNative.Comrak.markdown_to_html(@code_block_markdown, syntax_highlight: nil) ==
             "<pre><code class=\"language-elixir\">IO.puts(&quot;Hello&quot;)\n</code></pre>\n"
  end

  test "does not syntax highlight when syntax_highlight is false" do
    assert MDExNative.Comrak.markdown_to_html(@code_block_markdown, syntax_highlight: false) ==
             "<pre><code class=\"language-elixir\">IO.puts(&quot;Hello&quot;)\n</code></pre>\n"
  end
end
