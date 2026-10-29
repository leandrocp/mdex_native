defmodule MDExNative.BudgetExhaustedError do
  @moduledoc """
  Markdown parsing ran past the `limits: [parse_time: milliseconds]` option.

  `:time_limit` is the limit that was exceeded, in milliseconds.
  """

  defexception [:time_limit]

  @type t() :: %__MODULE__{time_limit: pos_integer()}

  @impl true
  def message(%__MODULE__{time_limit: time_limit}) do
    "Markdown parsing exceeded its #{time_limit} ms time limit"
  end
end
