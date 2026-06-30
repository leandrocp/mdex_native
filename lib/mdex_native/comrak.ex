defmodule MDExNative.Comrak do
  @moduledoc ~S"""
  Markdown parsing and rendering powered by the Rust `comrak` crate.

  Elixir bindings for Comrak's parser and renderers.

  Options follow Rust [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html)
  and use keyword lists. MDExNative also accepts `:sanitize` and
  `:syntax_highlight`.

  ## Examples

      iex> MDExNative.Comrak.markdown_to_html("# Hello")
      "<h1>Hello</h1>\n"

      iex> MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
      "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> done</li>\n</ul>\n"

      iex> MDExNative.Comrak.anchorize("Hello World")
      "hello-world"
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

  @typedoc "Comrak [`Options`](https://docs.rs/comrak/latest/comrak/options/struct.Options.html), plus MDExNative rendering options."
  @type options :: keyword()

  @doc ~S"""
  Parses Markdown into a generic MDExNative.Comrak AST.

  ## Examples

      iex> MDExNative.Comrak.parse_document("# Hello")
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
      }
  """
  @spec parse_document(markdown(), options()) :: MDExNative.Comrak.Document.t()
  def parse_document(markdown, options \\ []) when is_binary(markdown) do
    markdown
    |> MDExNative.Native.parse_document(options!(options))
    |> check_native_output()
  end

  @doc ~S"""
  Converts Markdown to HTML.

  ## Options

  Pass Comrak options as keyword lists matching [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html)
  or the extra MDExNative top-level options:

  - `:extension` - mapper to Comrak's [`Extension` options](https://docs.rs/comrak/latest/comrak/options/struct.Extension.html).
  - `:parse` - mapper to Comrak's [`Parse` options](https://docs.rs/comrak/latest/comrak/options/struct.Parse.html).
  - `:render` - mapper to Comrak's [`Render` options](https://docs.rs/comrak/latest/comrak/options/struct.Render.html).
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

  - `:escape_curly_braces_in_code` - escapes `{` and `}` inside `<code>` tags after
    rendering. Defaults to `true`.

  ## Examples

      iex> MDExNative.Comrak.markdown_to_html("**bold**")
      "<p><strong>bold</strong></p>\n"

      iex> MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
      "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> done</li>\n</ul>\n"

      iex> MDExNative.Comrak.markdown_to_html("<h1>Title</h1><p>Content</p>", render: [unsafe: true], sanitize: [rm_tags: ["h1"]])
      "Title<p>Content</p>\n"

      # default disabled syntax highlighter
      iex> markdown = "```rust\nfn main() {}\n```"
      iex> MDExNative.Comrak.markdown_to_html(markdown)
      "<pre><code class=\"language-rust\">fn main() &lbrace;&rbrace;\n</code></pre>\n"

      iex> markdown = "```rust\nfn main() {}\n```"
      iex> MDExNative.Comrak.markdown_to_html(markdown, escape_curly_braces_in_code: false)
      "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>\n"

  """
  @spec markdown_to_html(markdown(), options()) :: html()
  def markdown_to_html(markdown, options \\ []) when is_binary(markdown) do
    markdown
    |> MDExNative.Native.markdown_to_html_with_options(options!(options))
    |> check_native_output()
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to HTML.
  """
  @spec document_to_html(MDExNative.Comrak.Document.t(), options()) :: html()
  def document_to_html(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document
    |> MDExNative.Native.document_to_html_with_options(options!(options))
    |> check_native_output()
  end

  @doc ~S"""
  Converts Markdown to XML.

  ## Examples

      iex> MDExNative.Comrak.markdown_to_xml("# Hello")
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n<document xmlns=\"http://commonmark.org/xml/1.0\">\n  <heading level=\"1\">\n    <text xml:space=\"preserve\">Hello</text>\n  </heading>\n</document>\n"

      iex> MDExNative.Comrak.markdown_to_xml("# Hello", render: [sourcepos: true])
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n<document sourcepos=\"1:1-1:7\" xmlns=\"http://commonmark.org/xml/1.0\">\n  <heading sourcepos=\"1:1-1:7\" level=\"1\">\n    <text sourcepos=\"1:3-1:7\" xml:space=\"preserve\">Hello</text>\n  </heading>\n</document>\n"
  """
  @spec markdown_to_xml(markdown(), options()) :: xml()
  def markdown_to_xml(markdown, options \\ []) when is_binary(markdown) do
    markdown
    |> MDExNative.Native.markdown_to_xml_with_options(options!(options))
    |> check_native_output()
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to XML.
  """
  @spec document_to_xml(MDExNative.Comrak.Document.t(), options()) :: xml()
  def document_to_xml(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document
    |> MDExNative.Native.document_to_xml_with_options(options!(options))
    |> check_native_output()
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to CommonMark.
  """
  @spec document_to_commonmark(MDExNative.Comrak.Document.t(), options()) :: markdown()
  def document_to_commonmark(%MDExNative.Comrak.Document{} = document, options \\ []) do
    document
    |> MDExNative.Native.document_to_commonmark_with_options(options!(options))
    |> check_native_output()
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
    Map.new(options, fn
      {key, value} when key in [:extension, :parse, :render] and is_list(value) ->
        {key, Map.new(value)}

      {:syntax_highlight, value} when is_list(value) ->
        {:syntax_highlight, syntax_highlight_options(value)}

      {:sanitize, value} ->
        {:sanitize, MDExNative.Sanitize.normalize(value)}

      {key, value} ->
        {key, value}
    end)
  end

  defp syntax_highlight_options(options) do
    options
    |> Map.new(fn
      {:opts, opts} when is_list(opts) -> {:opts, Map.new(opts, &syntax_highlight_option/1)}
      option -> syntax_highlight_option(option)
    end)
  end

  defp syntax_highlight_option({:formatter, {formatter, opts}}) when is_list(opts) do
    {:formatter, {formatter, Map.new(opts)}}
  end

  defp syntax_highlight_option(option), do: option

  defp check_native_output(:lumis_not_enabled) do
    raise """
    Lumis is not enabled.

    Comrak tried to syntax highlight a code block with Lumis, but this NIF was not compiled with Lumis support.

    Enable it in your config:

        config :mdex_native, syntax_highlighter: :lumis

    """
  end

  defp check_native_output(:syntect_not_enabled) do
    raise """
    Syntect is not enabled.

    Comrak tried to syntax highlight a code block with Syntect, but this NIF was not compiled with Syntect support.

    Enable it in your config:

        config :mdex_native, syntax_highlighter: :syntect

    """
  end

  defp check_native_output(value), do: value
end
