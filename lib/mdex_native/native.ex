# https://github.com/elixir-explorer/explorer/blob/d11216282bbdb0dcaef2519c2bfefda46c2981e0/lib/explorer/polars_backend/native.ex

defmodule MDExNative.Native do
  @moduledoc false

  mix_config = Mix.Project.config()
  version = mix_config[:version]
  github_url = mix_config[:package][:links][:GitHub]
  mode = if Mix.env() in [:dev, :test], do: :debug, else: :release

  syntax_highlighter = Application.compile_env(:mdex_native, :syntax_highlighter, nil)

  unless syntax_highlighter in [:lumis, :syntect, nil] do
    raise ArgumentError, "invalid mdex_native syntax highlighter: #{inspect(syntax_highlighter)}"
  end

  force_build = System.get_env("MDEX_NATIVE_BUILD") in ["1", "true"]

  syntax_highlighter_features =
    case syntax_highlighter do
      :lumis -> ["lumis"]
      :syntect -> ["syntect"]
      nil -> []
    end

  cargo_features = ["nif_version_2_15" | syntax_highlighter_features]

  variants = [
    :legacy_cpu,
    :lumis,
    :legacy_cpu_lumis,
    :syntect,
    :legacy_cpu_syntect
  ]

  feature_variant =
    case syntax_highlighter do
      :lumis -> :lumis
      :syntect -> :syntect
      nil -> nil
    end

  legacy_env = System.get_env("MDEX_NATIVE_USE_LEGACY_ARTIFACTS")

  use_legacy =
    Application.compile_env(
      :mdex_native,
      :use_legacy_artifacts,
      legacy_env in ["true", "1"]
    )

  use_legacy_for_linux = fn ->
    # These are the same from the release workflow.
    # See the meaning in: https://unix.stackexchange.com/a/43540
    needed_caps = ~w[fxsr sse sse2 ssse3 sse4_1 sse4_2 popcnt avx fma]

    use_legacy or
      (is_nil(use_legacy) and
         not MDExNative.ComptimeUtils.cpu_with_all_caps?(needed_caps))
  end

  use_legacy_for_other = fn -> use_legacy == true end

  legacy_variant = if feature_variant, do: :"legacy_cpu_#{feature_variant}", else: :legacy_cpu

  variants_for = fn use_legacy? ->
    Enum.map(variants, fn variant ->
      {variant,
       fn -> if(use_legacy?.(), do: legacy_variant, else: feature_variant) == variant end}
    end)
  end

  targets = ~w(
    aarch64-apple-darwin
    aarch64-unknown-linux-gnu
    aarch64-unknown-linux-musl
    arm-unknown-linux-gnueabihf
    riscv64gc-unknown-linux-gnu
    x86_64-apple-darwin
    x86_64-pc-windows-gnu
    x86_64-pc-windows-msvc
    x86_64-unknown-freebsd
    x86_64-unknown-linux-gnu
    x86_64-unknown-linux-musl
  )

  variants =
    Map.new(targets, fn target ->
      variants =
        case target do
          "x86_64-unknown-linux-gnu" -> variants_for.(use_legacy_for_linux)
          "x86_64-pc-windows-msvc" -> variants_for.(use_legacy_for_other)
          "x86_64-pc-windows-gnu" -> variants_for.(use_legacy_for_other)
          "x86_64-unknown-freebsd" -> variants_for.(use_legacy_for_other)
          _ -> variants_for.(fn -> false end)
        end

      {target, variants}
    end)

  use RustlerPrecompiled,
    otp_app: :mdex_native,
    crate: "mdex_native_nif",
    version: version,
    base_url: "#{github_url}/releases/download/v#{version}",
    targets: targets,
    variants: variants,
    nif_versions: ["2.15"],
    mode: mode,
    default_features: false,
    features: cargo_features,
    force_build: force_build

  def parse_document(_md, _opts), do: :erlang.nif_error(:nif_not_loaded)
  def markdown_to_html_with_options(_md, _opts), do: :erlang.nif_error(:nif_not_loaded)
  def markdown_to_xml_with_options(_md, _opts), do: :erlang.nif_error(:nif_not_loaded)

  def document_to_commonmark(_doc), do: :erlang.nif_error(:nif_not_loaded)
  def document_to_commonmark_with_options(_doc, _opts), do: :erlang.nif_error(:nif_not_loaded)
  def document_to_html(_doc), do: :erlang.nif_error(:nif_not_loaded)
  def document_to_html_with_options(_doc, _opts), do: :erlang.nif_error(:nif_not_loaded)
  def document_to_xml(_doc), do: :erlang.nif_error(:nif_not_loaded)
  def document_to_xml_with_options(_doc, _opts), do: :erlang.nif_error(:nif_not_loaded)

  def safe_html(_unsafe_html, _sanitize, _escape_content, _escape_curly_braces_in_code),
    do: :erlang.nif_error(:nif_not_loaded)

  def text_to_anchor(_text), do: :erlang.nif_error(:nif_not_loaded)
end
