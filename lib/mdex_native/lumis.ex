defmodule MDExNative.Lumis do
  @moduledoc """
  Syntax highlighting option helpers powered by the Rust `lumis` crate.
  """

  @doc """
  Builds the default syntax highlighting options accepted by `MDExNative.Comrak`.
  """
  @spec default_options() :: map()
  def default_options do
    %{
      language: nil,
      formatter:
        {:html_inline,
         %{
           theme: {:string, "onedark"},
           pre_class: nil,
           italic: false,
           include_highlights: false,
           highlight_lines: nil,
           header: nil
         }}
    }
  end
end
