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
       ref: "449f7520a0fb700db66512b82928888a12bba351",
       sparse: "packages/elixir/lumis"}
    ]
  end
end
