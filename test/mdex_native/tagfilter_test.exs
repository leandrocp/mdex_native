defmodule MDExNative.TagfilterTest do
  use ExUnit.Case, async: true

  alias MDExNative.Comrak

  @options [extension: [tagfilter: true], render: [unsafe: true]]
  @tags ~w(title textarea style xmp iframe noembed noframes script plaintext)

  test "deprecated tagfilter preserves inline and block filtering" do
    for tag <- @tags, name <- [tag, String.upcase(tag)] do
      html = "<#{name}>content</#{name}>"
      filtered = "&lt;#{name}>content&lt;/#{name}>"

      assert Comrak.markdown_to_html!("before #{html} after", @options) ==
               "<p>before #{filtered} after</p>\n"

      assert Comrak.markdown_to_html!("<div>#{html}</div>", @options) ==
               "<div>#{filtered}</div>\n"
    end
  end

  test "filters only whole tag names and preserves allowed HTML" do
    markdown = "<div>é<script src=\"x\"/><scripture>x</scripture><b>ok</b></div>"

    assert Comrak.markdown_to_html!(markdown, @options) ==
             "<div>é&lt;script src=\"x\"/><scripture>x</scripture><b>ok</b></div>\n"
  end

  test "remains disabled by default and when false or nil" do
    markdown = "<script>alert(1)</script>"

    for extension <- [[], [tagfilter: false], [tagfilter: nil]] do
      assert Comrak.markdown_to_html!(markdown, extension: extension, render: [unsafe: true]) ==
               markdown <> "\n"
    end
  end

  test "safe rendering and escape take precedence over tagfilter" do
    markdown = "<script>alert(1)</script>"

    assert Comrak.markdown_to_html!(markdown, extension: [tagfilter: true]) ==
             "<!-- raw HTML omitted -->\n"

    assert Comrak.markdown_to_html!(markdown,
             extension: [tagfilter: true],
             render: [unsafe: true, escape: true]
           ) == "&lt;script&gt;alert(1)&lt;/script&gt;\n"
  end

  test "does not filter code or change image alt text" do
    for markdown <- ["`<script>`", "```html\n<script>x</script>\n```", "![<script>x</script>](x)"] do
      assert Comrak.markdown_to_html!(markdown, @options) ==
               Comrak.markdown_to_html!(markdown, render: [unsafe: true])
    end
  end

  test "preserves filtering when rendering an existing document" do
    markdown = "before <script>x</script> after\n\n<div><style>x</style></div>"
    document = Comrak.parse_document!(markdown, @options)

    assert Comrak.document_to_html!(document, @options) ==
             "<p>before &lt;script>x&lt;/script> after</p>\n<div>&lt;style>x&lt;/style></div>\n"

    assert Comrak.document_to_html!(document, render: [unsafe: true]) =~ "<script>x</script>"
  end

  test "retains custom link and image attributes with tagfilter enabled" do
    options = [extension: [tagfilter: true, link_attributes: true], render: [unsafe: true]]

    assert Comrak.markdown_to_html!("[link](x){.external} ![image](x){width=100}", options) ==
             "<p><a href=\"x\" class=\"external\">link</a> <img src=\"x\" alt=\"image\" width=\"100\" /></p>\n"
  end
end
