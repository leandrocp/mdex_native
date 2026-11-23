defmodule MDExNativeSmoke.MixProject do
  use Mix.Project

  def project do
    [app: :mdex_native_smoke, version: "0.1.0", elixir: "~> 1.15", deps: deps()]
  end

  def application, do: [extra_applications: [:logger]]

  defp deps do
    [
      {:mdex_native, path: ".."},
      # Another NIF in the same VM, as in an MDEx app: glibc shares one static
      # TLS reserve between them (leandrocp/lumis#2114).
      {:lumis, "~> 0.10"},
      {:lumis_wasm_elixir, "~> 0.26.0"}
    ]
  end
end
