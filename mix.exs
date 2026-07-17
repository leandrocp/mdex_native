defmodule MDExNative.MixProject do
  use Mix.Project

  @source_url "https://github.com/leandrocp/mdex_native"
  @version "0.2.6"
  @force_build? System.get_env("MDEX_NATIVE_BUILD") in ["1", "true"]

  def project do
    [
      app: :mdex_native,
      version: @version,
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      package: package(),
      deps: deps(),
      aliases: aliases(),
      name: "MDExNative",
      homepage_url: @source_url,
      description: "Markdown Elixir Native: Comrak, Ammonia, Lumis, and Syntect",
      test_coverage: [
        summary: [threshold: 80],
        ignore_modules: [MDExNative.ComptimeUtils, MDExNative.Native]
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp package do
    [
      maintainers: ["Leandro Pereira"],
      licenses: ["MIT"],
      links: %{
        GitHub: @source_url
      },
      files: ~w[
        lib
        examples
        guides
        native/mdex_native_nif/src
        native/mdex_native_nif/.cargo
        native/mdex_native_nif/Cargo.*
        native/mdex_native_nif/Cross.toml
        mix.exs
        docs.exs
        README.md
        LICENSE.md
        CHANGELOG.md
        checksum-Elixir.MDExNative.Native.exs
      ]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.32", optional: not @force_build?},
      {:rustler_precompiled, "~> 0.8"},
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false}
    ]
  end

  defp aliases do
    [
      setup: ["deps.get", "compile"],
      docs: &build_docs/1,
      "gen.checksum": "rustler_precompiled.download MDExNative.Native --all --print",
      format: ["cmd cargo fmt --manifest-path=native/mdex_native_nif/Cargo.toml --all", "format"],
      lint: [
        "credo",
        "cmd cargo clippy --manifest-path=native/mdex_native_nif/Cargo.toml -- -Dwarnings"
      ],
      test: ["cmd cargo test --manifest-path=native/mdex_native_nif/Cargo.toml", "test"]
    ]
  end

  defp build_docs(_) do
    Mix.Task.run("compile")
    ex_doc = Path.join(Mix.path_for(:escripts), "ex_doc")

    if not File.exists?(ex_doc) do
      raise "cannot build docs because the ex_doc escript is not installed, " <>
              "make sure to run `mix escript.install hex ex_doc` before"
    end

    args = ["MDExNative", @version, Mix.Project.compile_path()]

    opts =
      ~w[--main MDExNative --source-ref v#{@version} --source-url #{@source_url} --config docs.exs]

    System.cmd(ex_doc, args ++ opts)
  end
end
