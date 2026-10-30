defmodule MDExNative.ParseMemoryExceededError do
  @moduledoc """
  Markdown parsing held more memory than the `budget: [parse_memory: bytes]` option allows.

  `:parse_memory` is the limit that was exceeded, in bytes.
  """

  defexception [:parse_memory]

  @type t() :: %__MODULE__{parse_memory: pos_integer()}

  @impl true
  def message(%__MODULE__{parse_memory: parse_memory}) do
    "Markdown parsing exceeded its #{parse_memory} byte memory limit"
  end
end
