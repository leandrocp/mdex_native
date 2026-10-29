defmodule MDExNative.SyntaxHighlightError do
  @moduledoc """
  Syntax highlighting failed, or the requested highlighter is not compiled in.
  """

  defexception [:message]

  @type t() :: %__MODULE__{message: String.t()}
end
