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

  test "renders fenced code with syntax highlighting options" do
    html =
      MDExNative.Comrak.markdown_to_html(@code_block_markdown,
        syntax_highlight: [
          engine: :lumis,
          opts: [
            formatter:
              {:html_inline, theme: "catppuccin_macchiato", pre_class: "code-block-example"}
          ]
        ]
      )

    assert html =~ ~s(<pre class="lumis code-block-example")
    assert html =~ "IO"
  end

  test "supports legacy syntax highlighting options" do
    html =
      MDExNative.Comrak.markdown_to_html(@code_block_markdown,
        syntax_highlight: [
          formatter:
            {:html_inline, theme: "catppuccin_macchiato", pre_class: "code-block-example"}
        ]
      )

    assert html =~ ~s(<pre class="lumis code-block-example")
  end

  test "renders fenced code with syntect and no default theme" do
    html =
      MDExNative.Comrak.markdown_to_html("```rust\nfn main() {}\n```",
        syntax_highlight: [engine: :syntect]
      )

    assert html =~ ~s(<pre class="syntax-highlighting"><code class="language-rust">)
    assert html =~ ~s(<span class="source rust">)
  end

  test "renders fenced code with syntect theme" do
    html =
      MDExNative.Comrak.markdown_to_html("```rust\nfn main() {}\n```",
        syntax_highlight: [engine: :syntect, opts: [theme: "Catppuccin Macchiato"]]
      )

    assert html =~ ~s(<pre style="background-color:)
    assert html =~ ~s(<span style=)
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
