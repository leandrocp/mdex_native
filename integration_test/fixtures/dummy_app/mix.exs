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
      {:lumis, "~> 0.9"},
      # A parser is an ordinary dependency now: a language the project does not
      # depend on is not fetched, it renders plain. These are the ones the
      # fixture's fences name, plus the two html injects.
      {:lumis_wasm_rust, "~> 0.26"},
      {:lumis_wasm_elixir, "~> 0.26"},
      {:lumis_wasm_html, "~> 0.26"},
      {:lumis_wasm_json, "~> 0.26"},
      {:lumis_wasm_css, "~> 0.26"},
      {:lumis_wasm_javascript, "~> 0.26"}
    ]
  end
end
