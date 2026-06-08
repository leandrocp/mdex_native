import Config

case System.fetch_env!("MDEX_NATIVE_E2E_CASE") do
  "default" ->
    :ok

  "lumis" ->
    config :mdex_native, syntax_highlighter: :lumis

  "syntect" ->
    config :mdex_native, syntax_highlighter: :syntect
end
