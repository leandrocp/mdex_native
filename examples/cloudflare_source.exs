cache_path = Path.join(__DIR__, "tmp/cloudflare_source")

File.rm_rf!(cache_path)
System.delete_env("MDEX_NATIVE_BUILD")

System.put_env("MIX_INSTALL_DIR", Path.join(cache_path, "mix_install"))

System.put_env(
  "RUSTLER_PRECOMPILED_GLOBAL_CACHE_PATH",
  Path.join(cache_path, "rustler_precompiled")
)

Mix.install(
  [{:mdex_native, ">= 0.2.3"}],
  config: [mdex_native: [artifact_source: :cloudflare]]
)

markdown = """
# Cloudflare artifact source

This example downloads the precompiled NIF from Cloudflare.
"""

html = MDExNative.Comrak.markdown_to_html(markdown)

IO.puts(html)
