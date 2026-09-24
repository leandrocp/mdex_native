defmodule MDExNativeE2E.LumisRegressionTest do
  use ExUnit.Case

  if Application.compile_env(:mdex_native, :syntax_highlighter) == :lumis do
    test "native markdown_to_html_with_options includes Lumis multi-theme pre attributes (issue #32)" do
      markdown = "```elixir\nIO.puts(:hello)\n```"

      formatter =
        {:html_multi_themes,
         themes: [light: "catppuccin_latte", dark: "catppuccin_mocha"],
         default_theme: "light-dark()"}

      lumis_opts = [formatter: formatter] |> Lumis.validate_options!() |> Lumis.rust_options!()

      html =
        MDExNative.Native.markdown_to_html_with_options(markdown, %{
          render: %{unsafe: true},
          syntax_highlight: %{
            engine: :lumis,
            opts: lumis_opts
          }
        })

      assert html =~
               "style=\"color: light-dark(#4c4f69, #cdd6f4); background-color: light-dark(#eff1f5, #1e1e2e);\""

      pre_classes = pre_classes(html)

      assert Enum.sort(pre_classes) == Enum.sort(["lumis", "lumis-themes", "light", "dark"])
    end

    # MDEx publishes flat spans where Lumis nests them, so a fence and
    # Lumis.highlight/2 agree on colour but not on element structure. These are
    # the languages whose scopes never nest, where the two must match exactly.
    @parity_samples [
      {"html", "<div class=\"a\"><script>let x = 1;</script></div>"},
      {"rust", "fn main() {\n    let v: Vec<String> = vec![];\n}"},
      {"json", "{\"a\": [1, 2, {\"b\": null}]}"}
    ]

    test "a fence renders exactly what Lumis.highlight would" do
      formatter = {:html_inline, theme: "onedark"}
      opts = [formatter: formatter] |> Lumis.validate_options!() |> Lumis.rust_options!()

      for {language, source} <- @parity_samples do
        {name, formatter_opts} = formatter

        direct =
          Lumis.highlight!(source, formatter: {name, [language: language] ++ formatter_opts})

        rendered =
          "```#{language}\n#{source}\n```"
          |> MDExNative.Comrak.markdown_to_html(syntax_highlight: [engine: :lumis, opts: opts])
          |> String.trim_trailing("\n")

        assert direct == rendered, "#{language} diverged from Lumis.highlight/2"
      end
    end

    test "warming a parser reports it ready and reads the lumis dependencies" do
      # The same `lumis_wasm_*` dependencies Lumis itself resolves, so a parser
      # is installed once for the VM rather than once per NIF.
      assert Lumis.Packages.installed_dirs() != []
      assert MDExNative.load_language("elixir") == true
    end

    test "a language the project does not depend on renders plain" do
      # Not fetched, and not an error either: one undeclared fence costs itself,
      # not the document.
      html = MDExNative.Comrak.markdown_to_html("```zig\nconst x = 1;\n```")

      assert html =~ "const x = 1;"
      refute html =~ "<span style=\"color: #"
    end

    test "an invalid Lumis option reports what Lumis said" do
      error =
        assert_raise NimbleOptions.ValidationError, fn ->
          MDExNative.Comrak.markdown_to_html("```elixir\n:ok\n```",
            syntax_highlight: [engine: :lumis, opts: [formatter: {:html_inline, nope: true}]]
          )
        end

      assert Exception.message(error) =~ "nope"
    end
  end

  defp pre_classes(html) do
    [_before, pre] = String.split(html, "<pre class=\"", parts: 2)
    [classes | _after] = String.split(pre, "\"", parts: 2)

    String.split(classes, " ")
  end
end
