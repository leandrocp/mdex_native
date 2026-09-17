defmodule MDExNativeE2E.MixProject do
  use Mix.Project

  def project do
    [
      app: :mdex_native_e2e,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp deps do
    [
      {:mdex_native, path: "../../.."},
      {:lumis,
       github: "leandrocp/lumis",
       ref: "1a6a20e582ecf35a9127890620a6c2d055467463",
       sparse: "packages/elixir/lumis"}
    ]
  end
end
