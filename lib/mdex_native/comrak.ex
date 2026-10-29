defmodule MDExNative.Comrak do
  # `:lumis` is brought by the application that highlights, not by this
  # project, so the module is legitimately absent on most builds.
  @compile {:no_warn_undefined, Lumis}

  @moduledoc ~S"""
  Markdown parsing and rendering powered by the Rust `comrak` crate.

  Elixir bindings for Comrak's parser and renderers.

  Options follow Rust [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html)
  and use keyword lists. MDExNative also accepts `:sanitize`,
  `:syntax_highlight`, and `:budget`.

  ## Return values

  `parse_document/2`, `markdown_to_html/2`, `markdown_to_xml/2`, and the
  `document_to_*` functions return `{:ok, result}` or `{:error, exception}`.
  A failed or unavailable syntax highlighter is a `MDExNative.SyntaxHighlightError`.
  Each has a bang variant, such as `markdown_to_html!/2`, that returns the result
  or raises the exception. Invalid options raise `ArgumentError` from both.

  The `:tagfilter` extension is deprecated but retains its existing behavior
  for compatibility. Use `:sanitize` to clean rendered HTML instead.

  ## Obsidian Flavored Markdown

  These extensions are disabled by default:

  - `extension: [ofm_comments: true]` parses `%%inline comments%%` and multiline
    comments starting with `%%` at the beginning of a line. Comments remain in
    the AST, CommonMark, and XML, but are omitted from HTML.
  - `extension: [ofm_callouts: true]` parses callouts such as
    `> [!warning|metadata]- **Title**`. Callouts support arbitrary types,
    metadata, Markdown titles, and fold markers (`+` expanded, `-` collapsed).
    They take precedence over `:alerts` when both are enabled; syntax rejected
    by callouts can still be parsed as an alert. HTML exposes fold state as
    data attributes, not interactive folding behavior.
  - Either `:wikilinks_title_after_pipe` or `:wikilinks_title_before_pipe`
    enables wiki links and `![[url]]` embeds. Embeds set `WikiLink.embed` to
    `true`, preserve `!` in CommonMark, and render as ordinary links in HTML.
    Set `extension: [wikilinks_title_after_pipe: true, wikilinks_ignore_embeds: true]`
    to leave `![[url]]` as literal text while still parsing `[[url]]`.

  ## Examples

      iex> MDExNative.Comrak.markdown_to_html("# Hello")
      {:ok, "<h1>Hello</h1>\n"}

      iex> MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
      {:ok, "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> done</li>\n</ul>\n"}

      iex> MDExNative.Comrak.anchorize("Hello World")
      "hello-world"

      iex> MDExNative.Comrak.markdown_to_html("Visible %%hidden%% text", extension: [ofm_comments: true])
      {:ok, "<p>Visible  text</p>\n"}
  """

  @typedoc "Markdown source text."
  @type markdown :: String.t()

  @typedoc "Rendered HTML."
  @type html :: String.t()

  @typedoc "Rendered CommonMark XML."
  @type xml :: String.t()

  @typedoc "Parsed fenced code block info string."
  @type code_fence_info :: %{
          language: String.t(),
          metadata: String.t(),
          attributes: %{String.t() => String.t() | true}
        }

  @typedoc "Parsed MDExNative.Comrak AST node."
  @type ast_node :: struct()

  @typedoc "Comrak [`Extension`](https://docs.rs/comrak/latest/comrak/options/struct.Extension.html) options."
  @type extension_options :: keyword()

  @typedoc "Comrak [`Parse`](https://docs.rs/comrak/latest/comrak/options/struct.Parse.html) options."
  @type parse_options :: keyword()

  @typedoc "Comrak [`Render`](https://docs.rs/comrak/latest/comrak/options/struct.Render.html) options."
  @type render_options :: keyword()

  @typedoc "Comrak [`Options`](https://docs.rs/comrak/latest/comrak/options/struct.Options.html), plus `:syntax_highlight`, `:sanitize`, and `:budget`."
  @type options :: keyword()

  @doc ~S"""
  Parses Markdown into a generic MDExNative.Comrak AST.

  ## Examples

      iex> MDExNative.Comrak.parse_document("# Hello")
      {:ok,
       %MDExNative.Comrak.Document{
         nodes: [
           %MDExNative.Comrak.Heading{
             nodes: [
               %MDExNative.Comrak.Text{literal: "Hello", sourcepos: %MDExNative.Comrak.Sourcepos{start: {1, 3}, end: {1, 7}}}
             ],
             level: 1,
             setext: false,
             sourcepos: %MDExNative.Comrak.Sourcepos{start: {1, 1}, end: {1, 7}}
           }
         ],
         sourcepos: %MDExNative.Comrak.Sourcepos{start: {1, 1}, end: {1, 7}}
       }}
  """
  @spec parse_document(markdown(), options()) ::
          {:ok, MDExNative.Comrak.Document.t()} | {:error, Exception.t()}
  def parse_document(markdown, options \\ []) when is_binary(markdown) do
    markdown
    |> MDExNative.Native.parse_document(options!(options))
    |> native_result()
  end

  @doc """
  Same as `parse_document/2` but raises the error instead of returning it.
  """
  @spec parse_document!(markdown(), options()) :: MDExNative.Comrak.Document.t()
  def parse_document!(markdown, options \\ []) when is_binary(markdown) do
    markdown |> parse_document(options) |> unwrap!()
  end

  @doc ~S"""
  Converts Markdown to HTML.

  ## Options

  Pass Comrak options as keyword lists matching [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html)
  or the extra `:syntax_highlight` and `:sanitize` options:
  MDExNative adds two top-level options:

  - `:extension` - maps to Comrak's [`Extension` options](https://docs.rs/comrak/latest/comrak/options/struct.Extension.html).
  - `:parse` - maps to Comrak's [`Parse` options](https://docs.rs/comrak/latest/comrak/options/struct.Parse.html).
  - `:render` - maps to Comrak's [`Render` options](https://docs.rs/comrak/latest/comrak/options/struct.Render.html),
  - `:syntax_highlight` - highlights fenced code blocks. Disabled by default.

    Defaults to `syntax_highlight: nil`.

    To highlight code, compile MDExNative with a highlighter and choose the engine:

      **Lumis**

      ```
      config :mdex_native, syntax_highlighter: :lumis

      [engine: :lumis, opts: [formatter: {:html_inline, theme: "catppuccin_macchiato"}]]
      ```

      **Syntect**

      ```
      config :mdex_native, syntax_highlighter: :syntect

      [engine: :syntect, opts: [theme: "Catppuccin Macchiato"]]
      ```

    See the [Syntax highlighting](syntax_highlighting.md) guide for complete examples.

  - `:sanitize` - cleans rendered HTML. Defaults to `nil`.

    See the [Sanitization](sanitization.md) guide for more info.

  - `:budget` - bounds parsing, as `[parse_time: milliseconds]`. Omitted, or
    `parse_time` set to `nil` or `0`, means no limit, which is the default.

    Applies to the Comrak parse in `parse_document/2`, `markdown_to_html/2`, and
    `markdown_to_xml/2`; rendering, syntax highlighting, and sanitization are
    not counted, and the `document_to_*` functions do not parse. Comrak checks
    the limit cooperatively, so a parse can run somewhat past it. Running out
    returns `{:error, %MDExNative.ParseTimeExceededError{}}`, never partial output;
    the bang variants raise it.
    This is separate from the per-block `:budget` in Lumis's `:syntax_highlight`
    options; Syntect has none.

        {:ok, html} = MDExNative.Comrak.markdown_to_html(markdown, budget: [parse_time: 1_000])

  ## Examples

      iex> MDExNative.Comrak.markdown_to_html("**bold**")
      {:ok, "<p><strong>bold</strong></p>\n"}

      iex> MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
      {:ok, "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> done</li>\n</ul>\n"}

      iex> MDExNative.Comrak.markdown_to_html("<h1>Title</h1><p>Content</p>", render: [unsafe: true], sanitize: [rm_tags: ["h1"]])
      {:ok, "Title<p>Content</p>\n"}

      # default disabled syntax highlighter
      iex> markdown = "```rust\nfn main() {}\n```"
      iex> MDExNative.Comrak.markdown_to_html(markdown)
      {:ok, "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>\n"}

  """
  @spec markdown_to_html(markdown(), options()) :: {:ok, html()} | {:error, Exception.t()}
  def markdown_to_html(markdown, options \\ []) when is_binary(markdown) do
    markdown
    |> MDExNative.Native.markdown_to_html_with_options(options!(options))
    |> native_result()
  end

  @doc """
  Same as `markdown_to_html/2` but raises the error instead of returning it.
  """
  @spec markdown_to_html!(markdown(), options()) :: html()
  def markdown_to_html!(markdown, options \\ []) when is_binary(markdown) do
    markdown |> markdown_to_html(options) |> unwrap!()
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to HTML.
  """
  @spec document_to_html(MDExNative.Comrak.Document.t(), options()) ::
          {:ok, html()} | {:error, Exception.t()}
  def document_to_html(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document
    |> MDExNative.Native.document_to_html_with_options(options!(options))
    |> native_result()
  end

  @doc """
  Same as `document_to_html/2` but raises the error instead of returning it.
  """
  @spec document_to_html!(MDExNative.Comrak.Document.t(), options()) :: html()
  def document_to_html!(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document |> document_to_html(options) |> unwrap!()
  end

  @doc ~S"""
  Converts Markdown to XML.

  ## Examples

      iex> MDExNative.Comrak.markdown_to_xml("# Hello")
      {:ok, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n<document xmlns=\"http://commonmark.org/xml/1.0\">\n  <heading level=\"1\">\n    <text xml:space=\"preserve\">Hello</text>\n  </heading>\n</document>\n"}

      iex> MDExNative.Comrak.markdown_to_xml("# Hello", render: [sourcepos: true])
      {:ok, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n<document sourcepos=\"1:1-1:7\" xmlns=\"http://commonmark.org/xml/1.0\">\n  <heading sourcepos=\"1:1-1:7\" level=\"1\">\n    <text sourcepos=\"1:3-1:7\" xml:space=\"preserve\">Hello</text>\n  </heading>\n</document>\n"}
  """
  @spec markdown_to_xml(markdown(), options()) :: {:ok, xml()} | {:error, Exception.t()}
  def markdown_to_xml(markdown, options \\ []) when is_binary(markdown) do
    markdown
    |> MDExNative.Native.markdown_to_xml_with_options(options!(options))
    |> native_result()
  end

  @doc """
  Same as `markdown_to_xml/2` but raises the error instead of returning it.
  """
  @spec markdown_to_xml!(markdown(), options()) :: xml()
  def markdown_to_xml!(markdown, options \\ []) when is_binary(markdown) do
    markdown |> markdown_to_xml(options) |> unwrap!()
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to XML.
  """
  @spec document_to_xml(MDExNative.Comrak.Document.t(), options()) ::
          {:ok, xml()} | {:error, Exception.t()}
  def document_to_xml(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document
    |> MDExNative.Native.document_to_xml_with_options(options!(options))
    |> native_result()
  end

  @doc """
  Same as `document_to_xml/2` but raises the error instead of returning it.
  """
  @spec document_to_xml!(MDExNative.Comrak.Document.t(), options()) :: xml()
  def document_to_xml!(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document |> document_to_xml(options) |> unwrap!()
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to CommonMark.
  """
  @spec document_to_commonmark(MDExNative.Comrak.Document.t(), options()) ::
          {:ok, markdown()} | {:error, Exception.t()}
  def document_to_commonmark(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document
    |> MDExNative.Native.document_to_commonmark_with_options(options!(options))
    |> native_result()
  end

  @doc """
  Same as `document_to_commonmark/2` but raises the error instead of returning it.
  """
  @spec document_to_commonmark!(MDExNative.Comrak.Document.t(), options()) :: markdown()
  def document_to_commonmark!(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document |> document_to_commonmark(options) |> unwrap!()
  end

  @doc ~S"""
  Converts text to a heading anchor.

  ## Examples

      iex> MDExNative.Comrak.anchorize("Hello World")
      "hello-world"
  """
  @spec anchorize(String.t()) :: String.t()
  def anchorize(text) when is_binary(text) do
    MDExNative.Native.text_to_anchor(text)
  end

  @doc ~S"""
  Returns whether a URL is considered dangerous or not.

  Calls [`comrak::html::dangerous_url/1`](https://docs.rs/comrak/latest/comrak/html/fn.dangerous_url.html).

  ## Examples

      iex> MDExNative.Comrak.dangerous_url?("javascript:alert(1)")
      true

      iex> MDExNative.Comrak.dangerous_url?("https://elixir-lang.org")
      false

      iex> MDExNative.Comrak.dangerous_url?("data:image/png;base64,AAAA")
      false

  """
  @spec dangerous_url?(String.t()) :: boolean()
  def dangerous_url?(url) when is_binary(url) do
    MDExNative.Native.dangerous_url(url)
  end

  @doc ~S"""
  Parses a fenced code block info string into generic parts.

  The first word is returned as the language, the remaining text is preserved as
  metadata, and shell-like tokens in the metadata are exposed as attributes.

  ## Examples

      iex> MDExNative.Comrak.parse_code_fence_info(~s(elixir pre_class="demo" highlight_lines=2 include_highlights))
      %{
        language: "elixir",
        metadata: ~s(pre_class="demo" highlight_lines=2 include_highlights),
        attributes: %{
          "pre_class" => "demo",
          "highlight_lines" => "2",
          "include_highlights" => true
        }
      }

      iex> MDExNative.Comrak.parse_code_fence_info("")
      %{language: "", metadata: "", attributes: %{}}

  """
  @spec parse_code_fence_info(String.t() | nil) :: code_fence_info()
  def parse_code_fence_info(info) when is_binary(info) or is_nil(info) do
    {language, metadata} = split_code_fence_info(info || "")

    %{
      language: language,
      metadata: metadata,
      attributes: code_fence_attributes(metadata)
    }
  end

  defp split_code_fence_info(info) do
    case String.split(info, ~r/\s+/, parts: 2, trim: true) do
      [language, metadata] -> {language, metadata}
      [language] -> {language, ""}
      [] -> {"", ""}
    end
  end

  defp code_fence_attributes(metadata) do
    metadata
    |> OptionParser.split()
    |> Map.new(fn token ->
      case String.split(token, "=", parts: 2) do
        [key, value] -> {key, value}
        [key] -> {key, true}
      end
    end)
  end

  defp options!(options) do
    MDExNative.Application.configure_lumis_store()

    Map.new(options, fn
      {key, value} when key in [:extension, :parse, :render, :budget] and is_list(value) ->
        {key, Map.new(value)}

      {:syntax_highlight, value} when is_list(value) ->
        {:syntax_highlight, syntax_highlight_options(value)}

      {:syntax_highlight, value} when is_map(value) ->
        {:syntax_highlight, legacy_syntax_highlight(value)}

      {:sanitize, value} ->
        {:sanitize, MDExNative.Sanitize.normalize(value)}

      {key, value} ->
        {key, value}
    end)
  end

  defp syntax_highlight_options(options) do
    engine = Keyword.get(options, :engine, :lumis)

    cond do
      Keyword.has_key?(options, :opts) ->
        # An engine defaulted here has to be written down: without the key the
        # NIF reads the legacy shape instead and ignores `:opts` entirely.
        options
        |> Map.new(fn
          {:opts, opts} when is_list(opts) -> {:opts, normalize_opts(engine, opts)}
          {:opts, %{} = opts} -> {:opts, legacy_opts(opts)}
          option -> syntax_highlight_option(option)
        end)
        |> Map.put(:engine, engine)

      # Legacy `syntax_highlight: [formatter: ...]`, which the NIF decodes
      # without an engine key, as the Lumis options themselves.
      Keyword.has_key?(options, :formatter) ->
        normalize_opts(engine, options)

      true ->
        Map.new(options, &syntax_highlight_option/1)
    end
  end

  # Lumis options cross as written, `[formatter: {:html_inline, theme: "onedark"}]`.
  # The NIF decodes them with the decoder the `:lumis` NIF uses, so it accepts
  # the same options, fills in the same defaults and raises the same
  # `ArgumentError` for a bad one.
  defp normalize_opts(:lumis, opts) do
    if Keyword.keyword?(opts),
      do: opts,
      else: Map.new(opts, &syntax_highlight_option/1)
  end

  defp normalize_opts(_engine, opts), do: Map.new(opts, &syntax_highlight_option/1)

  # MDEx 0.14.1 and earlier convert Lumis options with `Lumis.rust_options!/1`
  # before calling here and send the map it returns, with the formatter in the
  # wire format `lumis_nif` decoded through Lumis 0.10. So do projects that
  # copied that, as `syntax_highlight: [engine: :lumis, opts: rust_options]`.
  # Those versions and projects accept this release of mdex_native, so the
  # formatter is read back into the shape a caller writes, which is the only
  # one the NIF decodes now.
  @doc false
  def legacy_syntax_highlight(%{opts: %{} = opts} = options),
    do: %{options | opts: legacy_opts(opts)}

  def legacy_syntax_highlight(%{formatter: _} = options), do: legacy_opts(options)
  def legacy_syntax_highlight(options), do: options

  defp legacy_opts(%{formatter: formatter} = opts),
    do: %{opts | formatter: legacy_formatter(formatter)}

  defp legacy_opts(opts), do: opts

  defp legacy_formatter({name, %{} = opts}) when is_atom(name),
    do: {name, Enum.map(opts, &legacy_formatter_option/1)}

  defp legacy_formatter(formatter), do: formatter

  defp legacy_formatter_option({key, {:string, value}}) when key in [:theme, :background],
    do: {key, value}

  defp legacy_formatter_option({:theme, {:theme, theme}}), do: {:theme, theme}

  defp legacy_formatter_option({:themes, %{} = themes}),
    do: {:themes, Enum.map(themes, fn {id, theme} -> {legacy_key(id), theme} end)}

  defp legacy_formatter_option({:header, %{open_tag: open_tag, close_tag: close_tag}}),
    do: {:header, %{open_tag: open_tag, close_tag: close_tag}}

  defp legacy_formatter_option({:highlight_lines, %{lines: lines} = highlight_lines}) do
    highlight_lines =
      highlight_lines
      |> Map.delete(:__struct__)
      |> Map.put(:lines, Enum.map(lines, &legacy_line/1))
      |> Map.replace_lazy(:style, &legacy_line_style/1)

    {:highlight_lines, highlight_lines}
  end

  defp legacy_formatter_option(option), do: option

  # Atoms in the caller's options before Lumis turned them into strings.
  defp legacy_key(key) when is_binary(key), do: String.to_existing_atom(key)
  defp legacy_key(key), do: key

  defp legacy_line({:single, line}), do: line

  defp legacy_line({:range, %{start: first, end: last, step: step}}),
    do: Range.new(first, last, step)

  defp legacy_line(line), do: line

  defp legacy_line_style({:style, %{style: style}}), do: style
  defp legacy_line_style(style), do: style

  defp syntax_highlight_option({:formatter, {formatter, opts}}) when is_list(opts) do
    {:formatter, {formatter, Map.new(opts)}}
  end

  defp syntax_highlight_option(option), do: option

  # The NIF returns its value directly and reports failures as `{:error, term}`
  # or a bare atom; this turns them into the public `{:ok, _} | {:error, _}`.
  defp native_result({:error, {:lumis_error, reason}}) do
    highlight_error("""
    Lumis failed to highlight a code block.

    #{reason}

    """)
  end

  defp native_result({:error, {:parse_time_exceeded, parse_time}}),
    do: {:error, %MDExNative.ParseTimeExceededError{parse_time: parse_time}}

  defp native_result(:lumis_not_enabled), do: highlight_error(lumis_not_enabled_message())

  defp native_result(:syntect_not_enabled) do
    highlight_error("""
    Syntect is not enabled.

    Comrak tried to syntax highlight a code block with Syntect, but this NIF was not compiled with Syntect support.

    Enable it in your config:

        config :mdex_native, syntax_highlighter: :syntect

    """)
  end

  defp native_result(value), do: {:ok, value}

  defp highlight_error(message), do: {:error, %MDExNative.SyntaxHighlightError{message: message}}

  defp unwrap!({:ok, value}), do: value
  defp unwrap!({:error, error}), do: raise(error)

  defp lumis_not_enabled_message do
    """
    Lumis is not enabled.

    Comrak tried to syntax highlight a code block with Lumis, but this NIF was not compiled with Lumis support.

    Enable it in your config:

        config :mdex_native, syntax_highlighter: :lumis

    And add a parser package to your deps for every language you highlight:

        {:lumis_wasm_elixir, "~> 0.26"}

    """
  end
end
