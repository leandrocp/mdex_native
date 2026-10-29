defmodule MDExNative.SyntaxHighlightError do
  @moduledoc """
  Syntax highlighting failed, or the requested highlighter is not compiled in.

  Returned as `{:error, error}` by the `MDExNative.Comrak` rendering functions
  and raised by their bang variants.
  """

  defexception [:message]

  @type t() :: %__MODULE__{message: String.t()}
end
