defmodule MDExNativeE2E.LumisRegressionTest do
  use ExUnit.Case

  if Application.compile_env(:mdex_native, :syntax_highlighter) == :lumis do
    test "a multi-theme fence carries the Lumis pre attributes (issue #32)" do
      markdown = "```elixir\nIO.puts(:hello)\n```"

      formatter =
        {:html_multi_themes,
         themes: [light: "catppuccin_latte", dark: "catppuccin_mocha"],
         default_theme: "light-dark()"}

      html =
        MDExNative.Comrak.markdown_to_html(markdown,
          render: [unsafe: true],
          syntax_highlight: [engine: :lumis, opts: [formatter: formatter]]
        )

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
        direct = Lumis.highlight!(source, language: language, formatter: formatter)

        rendered =
          "```#{language}\n#{source}\n```"
          |> MDExNative.Comrak.markdown_to_html(syntax_highlight: [engine: :lumis, opts: opts])
          |> String.trim_trailing("\n")

        assert direct == rendered, "#{language} diverged from Lumis.highlight/2"
      end
    end

    test "a parser warmed through Lumis is the one MDExNative renders with" do
      assert is_binary(Lumis.data_dir())
      assert :ok = Lumis.Languages.load("elixir")
      assert "elixir" in Lumis.loaded_languages()

      # Nothing here loads anything: the render finds the parser Lumis already
      # holds, or it does not highlight at all.
      assert MDExNative.Comrak.markdown_to_html("```elixir\n:ok\n```",
               syntax_highlight: [
                 engine: :lumis,
                 opts: [formatter: {:html_inline, theme: "onedark"}]
               ]
             ) =~ "<span"
    end

    test "a render handed no bridge reports it rather than rendering" do
      options = %{syntax_highlight: %{engine: :lumis, opts: %{}}}

      assert MDExNative.Native.markdown_to_html_with_options("```elixir\n:ok\n```", options, nil) ==
               :lumis_bridge_missing
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
