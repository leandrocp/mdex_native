defmodule MDExNativeE2E.ConfigTest do
  use ExUnit.Case

  @rust "```rust\nfn main() {}\n```"
  @typescript "```typescript\nconst answer: number = 42\n```"

  test "compile-time syntax highlighter config is used" do
    case System.fetch_env!("MDEX_NATIVE_E2E_CASE") do
      "default" ->
        assert lumis_html(@rust) ==
                 "<pre class=\"lumis\" style=\"color: #cad3f5; background-color: #24273a;\"><code class=\"language-rust\" translate=\"no\" tabindex=\"0\"><div class=\"line\" data-line=\"1\"><span style=\"color: #c6a0f6;\">fn</span> <span style=\"color: #8aadf4;\">main</span><span style=\"color: #939ab7;\">(</span><span style=\"color: #939ab7;\">)</span> <span style=\"color: #939ab7;\">&lbrace;</span><span style=\"color: #939ab7;\">&rbrace;</span>\n</div></code></pre>\n"

        error =
          assert_raise RuntimeError, fn ->
            MDExNative.Comrak.markdown_to_html(@rust, syntax_highlight: [engine: :syntect])
          end

        assert error.message ==
                 "Syntect is not enabled.\n\nComrak tried to syntax highlight a code block with Syntect, but this NIF was not compiled with Syntect support.\n\nEnable it in your config:\n\n    config :mdex_native, syntax_highlighter: :syntect\n\n"

      "syntect" ->
        assert syntect_html(@rust) ==
                 "<pre class=\"syntax-highlighting\"><code class=\"language-rust\"><span class=\"source rust\"><span class=\"meta function rust\"><span class=\"meta function rust\"><span class=\"storage type function rust\">fn</span> </span><span class=\"entity name function rust\">main</span></span><span class=\"meta function rust\"><span class=\"meta function parameters rust\"><span class=\"punctuation section parameters begin rust\">(</span></span><span class=\"meta function rust\"><span class=\"meta function parameters rust\"><span class=\"punctuation section parameters end rust\">)</span></span></span></span><span class=\"meta function rust\"> </span><span class=\"meta function rust\"><span class=\"meta block rust\"><span class=\"punctuation section block begin rust\">&lbrace;</span></span><span class=\"meta block rust\"><span class=\"punctuation section block end rust\">&rbrace;</span></span></span>\n</span></code></pre>\n"

        error =
          assert_raise RuntimeError, fn ->
            MDExNative.Comrak.markdown_to_html(@rust, syntax_highlight: [engine: :lumis])
          end

        assert error.message ==
                 "Lumis is not enabled.\n\nComrak tried to syntax highlight a code block with Lumis, but this NIF was not compiled with Lumis support.\n\nEnable it in your config:\n\n    config :mdex_native, syntax_highlighter: :lumis, bundles: [:all]\n\n"

      "lumis_web" ->
        assert lumis_html(@typescript) ==
                 "<pre class=\"lumis\" style=\"color: #cad3f5; background-color: #24273a;\"><code class=\"language-typescript\" translate=\"no\" tabindex=\"0\"><div class=\"line\" data-line=\"1\"><span style=\"color: #c6a0f6;\">const</span> <span style=\"color: #cad3f5;\">answer</span><span style=\"color: #939ab7;\">:</span> <span style=\"color: #c6a0f6;\">number</span> <span style=\"color: #91d7e3;\">=</span> <span style=\"color: #f5a97f;\">42</span>\n</div></code></pre>\n"

        assert lumis_html(@rust) ==
                 "<pre class=\"lumis\" style=\"color: #cad3f5; background-color: #24273a;\"><code class=\"language-plaintext\" translate=\"no\" tabindex=\"0\"><div class=\"line\" data-line=\"1\">fn main() &lbrace;&rbrace;\n</div></code></pre>\n"
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
