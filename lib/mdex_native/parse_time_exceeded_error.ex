defmodule MDExNative.ParseTimeExceededError do
  @moduledoc """
  Markdown parsing ran past the `limits: [parse_time: milliseconds]` option.

  `:parse_time` is the limit that was exceeded, in milliseconds.
  """

  defexception [:parse_time]

  @type t() :: %__MODULE__{parse_time: pos_integer()}

  @impl true
  def message(%__MODULE__{parse_time: parse_time}) do
    "Markdown parsing exceeded its #{parse_time} ms time limit"
  end
end
