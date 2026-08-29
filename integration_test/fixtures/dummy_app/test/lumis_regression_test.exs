defmodule MDExNativeE2E.LumisRegressionTest do
  use ExUnit.Case

  if Application.compile_env(:mdex_native, :syntax_highlighter) == :lumis do
    test "native markdown_to_html_with_options includes Lumis multi-theme pre attributes (issue #32)" do
      markdown = "```elixir\nIO.puts(:hello)\n```"

      formatter =
        {:html_multi_themes,
         themes: [light: "catppuccin_latte", dark: "catppuccin_mocha"],
         default_theme: "light-dark()"}

      lumis_opts =
        [formatter: formatter]
        |> Lumis.validate_options!()
        |> Lumis.rust_options!()
        |> Map.put(:mdex_bridge, Lumis.__mdex_bridge__())

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

  test "a decorated fence keeps its decorators to itself" do
    markdown = """
    ```elixir highlight_lines=1 pre_class=first
    IO.puts(:hello)
    ```

    ```elixir
    IO.puts(:bye)
    ```

    ```
    #!/usr/bin/env python
    ```
    """

    html =
      MDExNative.Comrak.markdown_to_html(markdown,
        render: [unsafe: true, full_info_string: true],
        syntax_highlight: [engine: :lumis, opts: [formatter: {:html_inline, theme: "onedark"}]]
      )

    [first, second, third] = tl(String.split(html, "<pre "))

    assert first =~ ~s(class="lumis first")
    assert first =~ "background-color: #3b4252;"

    refute second =~ "first"
    refute second =~ "background-color: #3b4252;"
    assert second =~ "language-elixir"

    assert third =~ "language-plaintext"
    refute third =~ "language-python"
  end

  test "a Lumis failure reports what Lumis said" do
    error =
      assert_raise RuntimeError, fn ->
        MDExNative.Comrak.markdown_to_html("```elixir\nIO.puts(:x)\n```",
          syntax_highlight: [
            engine: :lumis,
            opts: [
              formatter:
                {:html_multi_themes,
                 themes: [light: "catppuccin_latte"], default_theme: "light-dark()"}
            ]
          ]
        )
      end

    assert error.message =~ "Lumis failed to highlight a code block"
    assert error.message =~ "LightDark mode requires themes named 'light' and 'dark'"
  end

  defp pre_classes(html) do
    [_before, pre] = String.split(html, "<pre class=\"", parts: 2)
    [classes | _after] = String.split(pre, "\"", parts: 2)

    String.split(classes, " ")
  end
end
