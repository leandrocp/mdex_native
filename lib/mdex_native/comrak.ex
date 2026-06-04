defmodule MDExNative.Comrak do
  @moduledoc ~S"""
  Markdown parsing and rendering powered by the Rust `comrak` crate.

  This package provides Elixir bindings for MDExNative.Comrak. It returns rendered HTML/XML
  directly and keeps the native NIF behind the public API.

  Options mirror Rust [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html)
  and can be passed as keyword lists. Nested `:extension`, `:parse`, and
  `:render` options are validated in Elixir before crossing the NIF boundary.

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

  @typedoc "Heading anchor generated from text."
  @type anchor :: String.t()

  @typedoc "Parsed fenced code block info string."
  @type code_fence_info :: %{
          language: String.t(),
          metadata: String.t(),
          attributes: %{String.t() => String.t() | true}
        }

  @typedoc "Parsed MDExNative.Comrak AST node."
  @type ast_node :: struct()

  @typedoc "MDExNative.Comrak extension options."
  @type extension_options :: keyword()

  @typedoc "MDExNative.Comrak parse options."
  @type parse_options :: keyword()

  @typedoc "MDExNative.Comrak list marker style used by CommonMark rendering."
  @type list_style :: :dash | :plus | :star

  @typedoc "MDExNative.Comrak render options."
  @type render_options :: keyword()

  @typedoc "Comrak options accepted by `markdown_to_html/2` and `markdown_to_xml/2`."
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
    MDExNative.Native.parse_document(markdown, options!(options))
  end

  @doc ~S"""
  Converts Markdown to HTML.

  ## Options

  Pass Comrak options as a keyword list matching Rust
  [`comrak::Options`](https://docs.rs/comrak/latest/comrak/struct.Options.html),
  with any of these top-level keys:

  - `:extension` - MDExNative.Comrak
    [`ExtensionOptions`](https://docs.rs/comrak/latest/comrak/struct.ExtensionOptions.html)
    options, for example `tasklist: true`, `table: true`, `autolink: true`, or
    `header_id_prefix: "prefix-"`.
  - `:parse` - MDExNative.Comrak
    [`ParseOptions`](https://docs.rs/comrak/latest/comrak/struct.ParseOptions.html)
    options, for example `smart: true`.
  - `:render` - MDExNative.Comrak
    [`RenderOptions`](https://docs.rs/comrak/latest/comrak/struct.RenderOptions.html)
    options, for example `unsafe: true`, `hardbreaks: true`, or `sourcepos: true`.
  - `:syntax_highlight` - syntax highlighting options produced by `MDExNative.Lumis`.
    When present, the native Rust `LumisAdapter` is installed as Comrak's
    `SyntaxHighlighterAdapter` for fenced code blocks.

  The accepted option keys are defined by `t:options/0`, `t:extension_options/0`,
  `t:parse_options/0`, and `t:render_options/0`.

  ## Examples

      iex> MDExNative.Comrak.markdown_to_html("**bold**")
      "<p><strong>bold</strong></p>\n"

      iex> MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])
      "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> done</li>\n</ul>\n"
  """
  @spec markdown_to_html(markdown(), options()) :: html()
  def markdown_to_html(markdown, options \\ []) when is_binary(markdown) do
    MDExNative.Native.markdown_to_html_with_options(markdown, options!(options))
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to HTML.
  """
  @spec document_to_html(MDExNative.Comrak.Document.t(), options()) :: html()
  def document_to_html(%MDExNative.Comrak.Document{} = document, options \\ []) do
    MDExNative.Native.document_to_html_with_options(document, options!(options))
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
    MDExNative.Native.markdown_to_xml_with_options(markdown, options!(options))
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to XML.
  """
  @spec document_to_xml(MDExNative.Comrak.Document.t(), options()) :: xml()
  def document_to_xml(%MDExNative.Comrak.Document{} = document, options \\ []) do
    MDExNative.Native.document_to_xml_with_options(document, options!(options))
  end

  @doc ~S"""
  Converts a generic MDExNative.Comrak document to CommonMark.
  """
  @spec document_to_commonmark(MDExNative.Comrak.Document.t(), options()) :: markdown()
  def document_to_commonmark(%MDExNative.Comrak.Document{} = document, options \\ []) do
    MDExNative.Native.document_to_commonmark_with_options(document, options!(options))
  end

  @doc ~S"""
  Converts text to a heading anchor.

  ## Examples

      iex> MDExNative.Comrak.anchorize("Hello World")
      "hello-world"
  """
  @spec anchorize(String.t()) :: anchor()
  def anchorize(text) when is_binary(text) do
    MDExNative.Native.text_to_anchor(text)
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

      {key, value} ->
        {key, value}
    end)
  end
end
