defmodule MDExNative do
  @external_resource "README.md"
  @moduledoc File.read!("README.md")

  @doc """
  Compiles a Lumis parser before it is first needed.

  Parsers are WebAssembly modules that arrive as `lumis_wasm_*` dependencies,
  and a cold one costs a Wasmtime compile. Warming the languages a deployment
  renders moves that off the first request. Returns whether the parser is ready
  — `false` for a language the project does not depend on, and for a NIF built
  without Lumis.

  Shares its compiled-module cache with the `:lumis` application, so a language
  `Lumis.Languages.load/1` warmed compiles faster here too. The two keep
  separate runtimes, so each still registers the parser once.

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
