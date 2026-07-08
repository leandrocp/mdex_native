defmodule MDExNativeE2E.ConfigTest do
  use ExUnit.Case

  @rust "```rust\nfn main() {}\n```"
  test "compile-time syntax highlighter config is used" do
    case System.fetch_env!("MDEX_NATIVE_E2E_CASE") do
      e2e_case when e2e_case in ["default", "cloudflare"] ->
        assert MDExNative.Comrak.markdown_to_html(@rust, syntax_highlight: nil) ==
                 "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>\n"

        lumis_error =
          assert_raise RuntimeError, fn ->
            lumis_html(@rust)
          end

        assert lumis_error.message ==
                 "Lumis is not enabled.\n\nComrak tried to syntax highlight a code block with Lumis, but this NIF was not compiled with Lumis support.\n\nEnable it in your config:\n\n    config :mdex_native, syntax_highlighter: :lumis\n\n"

        syntect_error =
          assert_raise RuntimeError, fn ->
            syntect_html(@rust)
          end

        assert syntect_error.message ==
                 "Syntect is not enabled.\n\nComrak tried to syntax highlight a code block with Syntect, but this NIF was not compiled with Syntect support.\n\nEnable it in your config:\n\n    config :mdex_native, syntax_highlighter: :syntect\n\n"

      "lumis" ->
        html = lumis_html(@rust)

        assert html =~ "<pre class=\"lumis\" style=\"color: #"
        assert html =~ "<code class=\"language-rust\" translate=\"no\" tabindex=\"0\">"
        assert html =~ "<span style=\"color: #"

        error =
          assert_raise RuntimeError, fn ->
            MDExNative.Comrak.markdown_to_html(@rust, syntax_highlight: [engine: :syntect])
          end

        assert error.message ==
                 "Syntect is not enabled.\n\nComrak tried to syntax highlight a code block with Syntect, but this NIF was not compiled with Syntect support.\n\nEnable it in your config:\n\n    config :mdex_native, syntax_highlighter: :syntect\n\n"

      "syntect" ->
        assert syntect_html(@rust) ==
                 "<pre class=\"syntax-highlighting\"><code class=\"language-rust\"><span class=\"source rust\"><span class=\"meta function rust\"><span class=\"meta function rust\"><span class=\"storage type function rust\">fn</span> </span><span class=\"entity name function rust\">main</span></span><span class=\"meta function rust\"><span class=\"meta function parameters rust\"><span class=\"punctuation section parameters begin rust\">(</span></span><span class=\"meta function rust\"><span class=\"meta function parameters rust\"><span class=\"punctuation section parameters end rust\">)</span></span></span></span><span class=\"meta function rust\"> </span><span class=\"meta function rust\"><span class=\"meta block rust\"><span class=\"punctuation section block begin rust\">{</span></span><span class=\"meta block rust\"><span class=\"punctuation section block end rust\">}</span></span></span>\n</span></code></pre>\n"

        error =
          assert_raise RuntimeError, fn ->
            MDExNative.Comrak.markdown_to_html(@rust, syntax_highlight: [engine: :lumis])
          end

        assert error.message ==
                 "Lumis is not enabled.\n\nComrak tried to syntax highlight a code block with Lumis, but this NIF was not compiled with Lumis support.\n\nEnable it in your config:\n\n    config :mdex_native, syntax_highlighter: :lumis\n\n"
    end
  end

  defp lumis_html(markdown) do
    MDExNative.Comrak.markdown_to_html(markdown,
      syntax_highlight: [
        engine: :lumis,
        opts: [formatter: {:html_inline, theme: "catppuccin_macchiato"}]
      ]
    )
  end

  defp syntect_html(markdown) do
    MDExNative.Comrak.markdown_to_html(markdown, syntax_highlight: [engine: :syntect])
  end
end
