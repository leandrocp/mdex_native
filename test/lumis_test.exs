defmodule MDExNative.LumisTest do
  use ExUnit.Case

  doctest MDExNative.Lumis

  test "builds default syntax highlight options for Comrak" do
    markdown = """
    ```elixir
    IO.puts("Hello")
    ```
    """

    html =
      MDExNative.Comrak.markdown_to_html(markdown,
        syntax_highlight: MDExNative.Lumis.default_options()
      )

    assert html =~ ~s(class="language-elixir")
    assert html =~ "Hello"
  end
end
