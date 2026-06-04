defmodule MDExNative.MixProject do
  use Mix.Project

  @source_url "https://github.com/leandrocp/mdex_native"
  @version "0.1.0"
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
      description: "Native foundation for MDEx: Comrak, Ammonia, and Lumis"
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
        native/mdex_native_nif/src
        native/mdex_native_nif/.cargo
        native/mdex_native_nif/Cargo.*
        native/mdex_native_nif/Cross.toml
        mix.exs
        README.md
        LICENSE.md
        CHANGELOG.md
        checksum-Elixir.MDExNative.Native.exs
      ]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.32"},
      {:rustler_precompiled, "~> 0.7"}
    ]
  end

  defp aliases do
    [
      setup: ["deps.get", "compile"],
      "gen.checksum": "rustler_precompiled.download MDExNative.Native --all --print",
      "format.all": ["format", "rust.fmt"],
      "rust.lint": [
        "cmd cargo clippy --manifest-path=native/mdex_native_nif/Cargo.toml -- -Dwarnings"
      ],
      "rust.fmt": ["cmd cargo fmt --manifest-path=native/mdex_native_nif/Cargo.toml --all"]
    ]
  end
end
