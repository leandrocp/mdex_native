exclude = if System.get_env("MDEX_NATIVE_E2E_CLOUDFLARE") == "1", do: [], else: [cloudflare: true]

ExUnit.configure(timeout: :infinity, exclude: exclude)
ExUnit.start()
