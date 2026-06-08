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
      # TODO: change to `~> 0.1` after publish
      {:mdex_native, path: "../.."}
    ]
  end
end
