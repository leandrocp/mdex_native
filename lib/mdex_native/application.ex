defmodule MDExNative.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    configure_lumis_store()

    Supervisor.start_link([], strategy: :one_for_one, name: MDExNative.Supervisor)
  end

  # Lumis resolves its store directory in Elixir, from `config :lumis, :data_dir`,
  # then `LUMIS_DATA_DIR`, then its own `priv`. Nothing in the environment says
  # which it picked, so re-deriving it here would miss the `priv` default and
  # give the VM two stores, downloading and compiling every parser twice.
  if Code.ensure_loaded?(Lumis) and function_exported?(Lumis, :data_dir, 0) do
    defp configure_lumis_store do
      MDExNative.Native.configure_lumis_store(Lumis.data_dir())
    rescue
      # A NIF built without the Lumis feature does not export this one.
      UndefinedFunctionError -> :ok
      ErlangError -> :ok
    end
  else
    defp configure_lumis_store, do: :ok
  end
end
