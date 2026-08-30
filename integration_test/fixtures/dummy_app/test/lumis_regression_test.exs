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

    @parity_samples [
      {"elixir",
       "defmodule A do\n  @moduledoc \"\"\"\n  Doc\n  \"\"\"\n  def f(%{a: b}), do: {:ok, b}\nend"},
      {"python", "def f(a, *, b=1):\n    return f\"{a}{b}\"\n"},
      {"heex", "<div id={@id}><%= @name %></div>"},
      {"html", "<div class=\"a\"><script>let x = 1;</script></div>"},
      {"rust", "fn main() {\n    let v: Vec<String> = vec![];\n}"},
      {"json", "{\"a\": [1, 2, {\"b\": null}]}"}
    ]

    # Comrak and Lumis reach the same formatter through different callers, and
    # the fence is the only place their output could drift apart.
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

    test "warming a parser reports it ready and shares the lumis store" do
      assert is_binary(Lumis.data_dir())
      assert MDExNative.load_language("elixir")
    end
  end

  defp pre_classes(html) do
    [_before, pre] = String.split(html, "<pre class=\"", parts: 2)
    [classes | _after] = String.split(pre, "\"", parts: 2)

    String.split(classes, " ")
  end
end
