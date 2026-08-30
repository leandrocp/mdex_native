defmodule MDExNative do
  @external_resource "README.md"
  @moduledoc File.read!("README.md")

  @doc """
  Downloads and compiles a Lumis parser before it is first needed.

  Parsers are WebAssembly modules fetched on demand, and a cold one costs a
  download and a Wasmtime compile. Warming the languages a deployment renders
  moves both off the first request. Returns whether the parser is ready.

  Shares its store with the `:lumis` application, so a parser either side warms
  serves both. Answers `false` when this NIF was built without Lumis.

  ## Examples

      MDExNative.load_language("elixir")
      #=> true

  """
  @spec load_language(String.t()) :: boolean()
  def load_language(name) when is_binary(name) do
    MDExNative.Native.load_lumis_language(name)
  rescue
    # The NIF exports this only when built with the Lumis feature.
    UndefinedFunctionError -> false
    ErlangError -> false
  end
end
