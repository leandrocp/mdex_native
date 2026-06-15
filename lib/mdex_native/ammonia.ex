defmodule MDExNative.Ammonia do
  @moduledoc ~S"""
  HTML sanitization powered by the Rust `ammonia` crate.

  See the [Sanitization](sanitization.html) guide for Markdown rendering and
  raw HTML behavior.
  """

  @typedoc """
  HTML sanitizer mode.

  - `nil` - disables sanitization.
  - `:clean` - uses [`ammonia::clean/1`](https://docs.rs/ammonia/latest/ammonia/fn.clean.html).
  - `t:sanitize_options/0` - keyword options for Ammonia's builder,
    for example `[rm_tags: ["h1"]]`.
  """
  @type sanitize_option :: nil | :clean | sanitize_options()

  @typedoc """
  Keyword options for Ammonia's builder.

  Keys map to Ammonia builder operations. For keys with `set`, `add`, and `rm`
  variants, the base key replaces the default set, `add_*` appends to it, and
  `rm_*` removes from it.
  """
  @type sanitize_options :: [
          tags: [String.t()],
          add_tags: [String.t()],
          rm_tags: [String.t()],
          clean_content_tags: [String.t()],
          add_clean_content_tags: [String.t()],
          rm_clean_content_tags: [String.t()],
          tag_attributes: %{String.t() => [String.t()]},
          add_tag_attributes: %{String.t() => [String.t()]},
          rm_tag_attributes: %{String.t() => [String.t()]},
          tag_attribute_values: %{String.t() => %{String.t() => [String.t()]}},
          add_tag_attribute_values: %{String.t() => %{String.t() => [String.t()]}},
          rm_tag_attribute_values: %{String.t() => %{String.t() => [String.t()]}},
          set_tag_attribute_values: %{String.t() => %{String.t() => String.t()}},
          set_tag_attribute_value: %{String.t() => %{String.t() => String.t()}},
          rm_set_tag_attribute_value: %{String.t() => String.t()},
          generic_attribute_prefixes: [String.t()],
          add_generic_attribute_prefixes: [String.t()],
          rm_generic_attribute_prefixes: [String.t()],
          generic_attributes: [String.t()],
          add_generic_attributes: [String.t()],
          rm_generic_attributes: [String.t()],
          url_schemes: [String.t()],
          add_url_schemes: [String.t()],
          rm_url_schemes: [String.t()],
          url_relative:
            :deny
            | :passthrough
            | {:rewrite_with_base, String.t()}
            | {:rewrite_with_root, {String.t(), String.t()}},
          link_rel: String.t() | nil,
          allowed_classes: %{String.t() => [String.t()]},
          add_allowed_classes: %{String.t() => [String.t()]},
          rm_allowed_classes: %{String.t() => [String.t()]},
          strip_comments: boolean(),
          id_prefix: String.t() | nil,
          filter_style_properties: [String.t()]
        ]

  @type options :: [
          sanitize: sanitize_option(),
          escape_content: boolean(),
          escape_curly_braces_in_code: boolean()
        ]

  @doc """
  Sanitizes an HTML string.

  ## Options

  - `:sanitize` - `t:sanitize_option/0`. Defaults to `:clean`.
  - `:escape_content` - `boolean/0`. Defaults to `false`.
    Escapes the full sanitized HTML string when enabled.
  - `:escape_curly_braces_in_code` - `boolean/0`. Defaults to `true`.
    Escapes `{` and `}` inside `<code>` tags.

  ## Examples

  Set allowed tags, replacing the default tag list:

      iex> MDExNative.Ammonia.safe_html("<h1>Title</h1><p>Content</p>", sanitize: [tags: ["p"]])
      "Title<p>Content</p>"

  Add an allowed tag without replacing the defaults:

      iex> MDExNative.Ammonia.safe_html("<custom>Intro</custom>", sanitize: [add_tags: ["custom"]])
      "<custom>Intro</custom>"

  Remove an allowed tag from the defaults:

      iex> MDExNative.Ammonia.safe_html("<h1>Title</h1><p>Content</p>", sanitize: [rm_tags: ["h1"]])
      "Title<p>Content</p>"

  Combine operations:

      iex> MDExNative.Ammonia.safe_html(~s|<h1>Title</h1><section data-kind="note" onclick="x">Content</section>|, sanitize: [add_tags: ["section"], add_tag_attributes: %{"section" => ["data-kind"]}, rm_tags: ["h1"]])
      ~s|Title<section data-kind="note">Content</section>|

  """
  @spec safe_html(String.t(), options()) :: String.t()
  def safe_html(html, opts \\ []) when is_binary(html) and is_list(opts) do
    opts =
      Keyword.validate!(opts,
        sanitize: :clean,
        escape_content: false,
        escape_curly_braces_in_code: true
      )

    MDExNative.Native.safe_html(
      html,
      MDExNative.Sanitize.normalize(opts[:sanitize]),
      opts[:escape_content],
      opts[:escape_curly_braces_in_code]
    )
  end
end
