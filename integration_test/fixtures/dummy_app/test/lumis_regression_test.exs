if System.fetch_env!("MDEX_NATIVE_E2E_CASE") == "lumis" do
  defmodule MDExNativeE2E.LumisRegressionTest do
    use ExUnit.Case

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
  end

  defp pre_classes(html) do
    [_before, pre] = String.split(html, "<pre class=\"", parts: 2)
    [classes | _after] = String.split(pre, "\"", parts: 2)

    String.split(classes, " ")
  end
end
