defmodule MDExNative.Ammonia do
  @moduledoc """
  HTML sanitization powered by the Rust `ammonia` crate.
  """

  @type sanitize_option :: nil | :default | :clean | {:custom, map()}
  @type options :: [
          sanitize: sanitize_option(),
          escape_content: boolean(),
          escape_curly_braces_in_code: boolean()
        ]

  @doc """
  Sanitizes an HTML string with native Ammonia options.

  ## Options

  - `:sanitize` - `t:sanitize_option/0`. Defaults to `:default`, which uses
    Ammonia's default cleaner. Pass `nil` to skip sanitization.
  - `:escape_content` - `boolean/0`. Defaults to `false`. Escapes the full
    sanitized HTML string when enabled.
  - `:escape_curly_braces_in_code` - `boolean/0`. Defaults to `true`. Escapes
    `{` and `}` inside `<code>` tags.
  """
  @spec safe_html(String.t(), options()) :: String.t()
  def safe_html(html, opts \\ []) when is_binary(html) and is_list(opts) do
    opts =
      Keyword.validate!(opts,
        sanitize: :default,
        escape_content: false,
        escape_curly_braces_in_code: true
      )

    MDExNative.Native.safe_html(
      html,
      normalize_sanitize(opts[:sanitize]),
      opts[:escape_content],
      opts[:escape_curly_braces_in_code]
    )
  end

  defp normalize_sanitize(:default), do: :clean
  defp normalize_sanitize(sanitize), do: sanitize
end
