defmodule MDExNativeE2E.CompileTimeRender do
  @moduledoc false

  if Application.compile_env(:mdex_native, :syntax_highlighter) == :lumis do
    @markdown "```elixir\nIO.puts(:hello)\n```"
    @formatter [formatter: {:html_inline, theme: "onedark"}]

    # Rendered by `mix compile`, which starts no application. The first render
    # in a VM builds the store, so it goes first.
    @from_keyword MDExNative.Comrak.markdown_to_html(@markdown,
                    syntax_highlight: [engine: :lumis, opts: @formatter]
                  )

    def from_keyword, do: @from_keyword

    # The map MDEx 0.14.1 sends, with options in Lumis 0.10's wire format.
    if Code.ensure_loaded?(Lumis) do
      @from_map MDExNative.Comrak.markdown_to_html(@markdown,
                  syntax_highlight: %{
                    engine: :lumis,
                    opts: @formatter |> Lumis.validate_options!() |> Lumis.rust_options!()
                  }
                )

      def from_map, do: @from_map
    end
  end
end
