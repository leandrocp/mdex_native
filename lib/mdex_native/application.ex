defmodule MDExNative.Application do
  @moduledoc false

  # `:lumis` is not a dependency of this project — an application that
  # highlights brings it and its parsers itself. Naming the module here would
  # otherwise warn on every build that does not.
  @compile {:no_warn_undefined, Lumis.Application}

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
  # Skipped unless there is something to configure. Without Lumis nothing
  # highlights, and a NIF built without the Lumis feature does not export the
  # function at all.
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
    if @lumis? and lumis_loaded?() do
      MDExNative.Native.configure_lumis_store(data_dir(), parser_dirs())
      :persistent_term.put(@configured, true)
    end
  end

  # `function_exported?/3` answers for a *loaded* module, and at boot `Lumis`
  # has not been loaded yet. Skipping the `ensure_loaded` left a release
  # configuring no store at all, which rendered every fence plain.
  defp lumis_loaded? do
    Code.ensure_loaded?(Lumis.Application) and
      function_exported?(Lumis.Application, :data_dir, 0)
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

  # Lumis resolves this from `config :lumis, :data_dir`, then `LUMIS_DATA_DIR`,
  # then its own `priv`. Asking it rather than re-deriving it is deliberate:
  # the two have to name the same directory or the VM keeps two compile caches,
  # and the `priv` default is not visible from here.
  #
  # `nil` hands the decision to the NIF, which reads `LUMIS_DATA_DIR` and then
  # falls back to the user data directory — the same answer Lumis would give.
  defp data_dir, do: Lumis.Application.data_dir()
end
