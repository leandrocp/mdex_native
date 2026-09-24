defmodule MDExNative.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    configure_lumis_store()

    Supervisor.start_link([], strategy: :one_for_one, name: MDExNative.Supervisor)
  end

  # Parsers arrive as `lumis_wasm_*` dependencies, and only the BEAM knows which
  # of them a release loaded, so Lumis reads that off the code path rather than
  # from a directory. The store directory is Lumis's too, resolved in Elixir
  # from `config :lumis, :data_dir`, then `LUMIS_DATA_DIR`, then its own `priv`;
  # nothing in the environment says which it picked, so re-deriving it here
  # would miss the `priv` default and give the VM two compile caches.
  if Code.ensure_loaded?(Lumis.Application) and
       function_exported?(Lumis.Application, :data_dir, 0) and
       Code.ensure_loaded?(Lumis.Packages) and
       function_exported?(Lumis.Packages, :installed_dirs, 0) do
    defp configure_lumis_store do
      MDExNative.Native.configure_lumis_store(
        Lumis.Application.data_dir(),
        Lumis.Packages.installed_dirs()
      )
    rescue
      # A NIF built without the Lumis feature does not export this one.
      UndefinedFunctionError -> :ok
      ErlangError -> :ok
    end
  else
    defp configure_lumis_store, do: :ok
  end
end
