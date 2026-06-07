import Config

case System.fetch_env!("MDEX_NATIVE_E2E_CASE") do
  "default" ->
    :ok

  "syntect" ->
    config :mdex_native, syntax_highlighter: :syntect

  "lumis_web" ->
    config :mdex_native,
      syntax_highlighter: :lumis,
      bundles: [:web]
end
