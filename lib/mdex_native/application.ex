defmodule MDExNative.Application do
  @moduledoc false

  use Application

  # A parser application ships `priv/parsers` and is named for the language it
  # carries. Both halves are Lumis's published layout, which is what makes
  # reading them here safe.
  @parser_prefix "lumis_wasm_"

  @impl true
  def start(_type, _args) do
    # Past the guard in `configure_lumis_store/0`: a render before this may have
    # configured the store without building it, and `config/runtime.exs` has
    # run since. The NIF refuses once the store is built.
    put_lumis_store()

    Supervisor.start_link([], strategy: :one_for_one, name: MDExNative.Supervisor)
  end

  # Where this NIF looks for parsers and keeps compiled modules. Both are
  # answers only the BEAM has: a release resolves its dependencies at boot and
  # has no project directory to infer them from.
  #
  # Whether the NIF has a Lumis store to configure at all is fixed when it is
  # built, and `MDExNative.Native` exports the stub either way — only calling
  # it tells the two apart, and it answers with a `:nif_not_loaded` error. The
  # config that selected the artifact is the honest question to ask.
  @lumis? Application.compile_env(:mdex_native, :syntax_highlighter) == :lumis

  @configured {__MODULE__, :lumis_store_configured}

  @doc false
  # The NIF reads these once, when its store is built, and refuses to change them
  # after. A render can build it before this application starts — `mix compile`
  # rendering a template runs no application at all — so `MDExNative.Comrak`
  # calls this before every render that highlights, not only `start/2`.
  def configure_lumis_store do
    if @lumis? and not :persistent_term.get(@configured, false), do: put_lumis_store()

    :ok
  end

  defp put_lumis_store do
    if @lumis? do
      MDExNative.Native.configure_lumis_store(data_dir(), parser_dirs())
      :persistent_term.put(@configured, true)
    end
  end

  @doc false
  # Where compiled parsers are cached: the directory the `:lumis` application
  # would pick, so with both installed each parser is compiled once between the
  # two NIFs. That is `config :lumis, :data_dir`, then `LUMIS_DATA_DIR`, then
  # `:lumis`'s own `priv/lumis`. OTP answers all three without loading a Lumis
  # module, so `:lumis` does not have to be installed; without it, the cache
  # lives in this application's `priv/lumis` instead.
  #
  # `nil` defers to the NIF, which reads `LUMIS_DATA_DIR` itself. An empty
  # value names no directory, as the NIF also treats it. A `priv` directory
  # rather than a user data directory, because a release ships it next to the
  # code and its user may have no home directory.
  def data_dir do
    cond do
      path = Application.get_env(:lumis, :data_dir) -> Path.expand(path)
      System.get_env("LUMIS_DATA_DIR") not in [nil, ""] -> nil
      true -> priv_dir(:lumis) || priv_dir(:mdex_native)
    end
  end

  defp priv_dir(app) do
    case :code.priv_dir(app) do
      {:error, _} -> nil
      priv -> Path.join(List.to_string(priv), "lumis")
    end
  end

  # `priv/parsers` of every installed `lumis_wasm_*` application.
  #
  # Read off the code path rather than `Application.loaded_applications/0`. A
  # parser application has no supervision tree and nothing depends on it at the
  # OTP level, so a release never loads it — the bytes are there in `lib/` and
  # the application is invisible. The code path lists it either way.
  #
  # An empty list is a real answer, not a missing one: the project depends on
  # no parsers, so it may render none. Nothing is fetched to make up the
  # difference.
  defp parser_dirs do
    for path <- :code.get_path(),
        dir = to_string(path),
        Path.basename(dir) == "ebin",
        root = Path.dirname(dir),
        String.starts_with?(Path.basename(root), @parser_prefix),
        parsers = Path.join([root, "priv", "parsers"]),
        File.dir?(parsers),
        uniq: true,
        do: parsers
  end
end
