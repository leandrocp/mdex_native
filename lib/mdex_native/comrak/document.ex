defmodule MDExNative.Comrak.Sourcepos do
  @moduledoc """
  Source position information for AST nodes.

  Positions are represented as one-based `{line, column}` tuples. They refer to
  the original Markdown source and are not updated when a document is modified
  programmatically.

  See Comrak's [`Sourcepos`](https://docs.rs/comrak/latest/comrak/nodes/struct.Sourcepos.html).
  """

  @type t :: %__MODULE__{
          start: {pos_integer(), pos_integer()},
          end: {pos_integer(), pos_integer()}
        }
  defstruct start: {0, 0}, end: {0, 0}
end

defmodule MDExNative.Comrak.Attributes do
  @moduledoc """
  Attributes attached to supported Markdown nodes.

  Attribute extensions can add an ID, CSS classes, and arbitrary key-value pairs.
  """

  @type t :: %__MODULE__{
          id: String.t() | nil,
          classes: [String.t()],
          pairs: [{String.t(), String.t()}]
        }
  defstruct id: nil, classes: [], pairs: []
end

defmodule MDExNative.Comrak.Document do
  @moduledoc """
  Root of a parsed Comrak Markdown document.

  A document contains the block and inline nodes produced by
  `MDExNative.Comrak.parse_document/2`. Nodes containing child nodes expose
  them through their `:nodes` field, while every parsed node includes its
  original `MDExNative.Comrak.Sourcepos`.

  ## Example

      MDExNative.Comrak.parse_document("# Hello")
      %MDExNative.Comrak.Document{
        nodes: [
          %MDExNative.Comrak.Heading{
            nodes: [%MDExNative.Comrak.Text{literal: "Hello"}],
            level: 1
          }
        ]
      }

  Documents may be built or changed using the node structs in the
  `MDExNative.Comrak` namespace, then rendered with
  `MDExNative.Comrak.document_to_html/2`,
  `MDExNative.Comrak.document_to_commonmark/2`, or
  `MDExNative.Comrak.document_to_xml/2`.
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          sourcepos: MDExNative.Comrak.Sourcepos.t()
        }
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.FrontMatter do
  @moduledoc """
  Document metadata.
  """

  @type t :: %__MODULE__{literal: String.t(), sourcepos: MDExNative.Comrak.Sourcepos.t()}
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.BlockQuote do
  @moduledoc """
  A block quote marker.

  Spec: https://github.github.com/gfm/#block-quotes
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          sourcepos: MDExNative.Comrak.Sourcepos.t()
        }
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.List do
  @moduledoc """
  A List that contains `MDExNative.Comrak.ListItem`.

  Spec: https://github.github.com/gfm/#lists
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          list_type: :bullet | :ordered,
          marker_offset: non_neg_integer(),
          padding: non_neg_integer(),
          start: non_neg_integer(),
          delimiter: :period | :paren,
          bullet_char: String.t(),
          tight: boolean(),
          is_task_list: boolean()
        }
  defstruct nodes: [],
            list_type: :bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: :period,
            bullet_char: "-",
            tight: true,
            is_task_list: false,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.ListItem do
  @moduledoc """
  A List Item of a `MDExNative.Comrak.List`.

  Spec: https://github.github.com/gfm/#list-items
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          list_type: :bullet | :ordered,
          marker_offset: non_neg_integer(),
          padding: non_neg_integer(),
          start: non_neg_integer(),
          delimiter: :period | :paren,
          bullet_char: String.t(),
          tight: boolean(),
          is_task_list: boolean()
        }
  defstruct nodes: [],
            list_type: :bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: :period,
            bullet_char: "-",
            tight: true,
            is_task_list: false,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionList do
  @moduledoc """
  A description list.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionItem do
  @moduledoc """
  A description item of a description list.

  See `MDExNative.Comrak.DescriptionList`
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          marker_offset: non_neg_integer(),
          padding: non_neg_integer(),
          tight: boolean()
        }
  defstruct nodes: [],
            marker_offset: 0,
            padding: 0,
            tight: false,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionTerm do
  @moduledoc """
  A description term of a description item.

  See `MDExNative.Comrak.DescriptionList`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionDetails do
  @moduledoc """
  Description details of a description item.

  See `MDExNative.Comrak.DescriptionList`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.CodeBlock do
  @moduledoc """
  A code block, fenced or indented.

  Spec: https://github.github.com/gfm/#fenced-code-blocks and https://github.github.com/gfm/#indented-code-blocks
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          fenced: boolean(),
          fence_char: String.t(),
          fence_length: non_neg_integer(),
          fence_offset: non_neg_integer(),
          info: String.t(),
          literal: String.t(),
          closed: boolean(),
          attrs: MDExNative.Comrak.Attributes.t() | nil
        }
  defstruct nodes: [],
            fenced: true,
            fence_char: "`",
            fence_length: 3,
            fence_offset: 0,
            info: "",
            literal: "",
            closed: true,
            attrs: nil,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HtmlBlock do
  @moduledoc """
  A HTML block.

  Spec: https://github.github.com/gfm/#html-blocks
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          block_type: non_neg_integer(),
          literal: String.t()
        }
  defstruct nodes: [], block_type: 0, literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Paragraph do
  @moduledoc """
  A paragraph that contains nodes.

  Spec: https://github.github.com/gfm/#paragraphs
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Heading do
  @moduledoc """
  A heading, either ATX or setext.

  ATX is the most common heading, a line starting with 1-6 `#` characters,
  and setext is represented as one or more lines followed by a heading underline as `===` or `---`.

  Spec: https://github.github.com/gfm/#atx-headings and https://github.github.com/gfm/#setext-headings
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          level: pos_integer(),
          setext: boolean(),
          closed: boolean(),
          attrs: MDExNative.Comrak.Attributes.t() | nil
        }
  defstruct nodes: [],
            level: 1,
            setext: false,
            closed: false,
            attrs: nil,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.ThematicBreak do
  @moduledoc """
  A break between lines.

  Spec: https://github.github.com/gfm/#thematic-breaks
  """

  @type t :: %__MODULE__{}
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.FootnoteDefinition do
  @moduledoc """
  A footnote definition.
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          name: String.t(),
          total_references: non_neg_integer()
        }
  defstruct nodes: [], name: "", total_references: 0, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.FootnoteReference do
  @moduledoc """
  The reference to a footnote.
  """

  @type t :: %__MODULE__{
          name: String.t(),
          ref_num: non_neg_integer(),
          ix: non_neg_integer(),
          texts: [{String.t(), non_neg_integer()}]
        }
  defstruct name: "", ref_num: nil, ix: nil, texts: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Table do
  @moduledoc """
  A table with rows and columns.

  Spec: https://github.github.com/gfm/#tables-extension-
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          alignments: [:none | :left | :right | :center],
          num_columns: non_neg_integer(),
          num_rows: non_neg_integer(),
          num_nonempty_cells: non_neg_integer()
        }
  defstruct nodes: [],
            alignments: [],
            num_columns: 0,
            num_rows: 0,
            num_nonempty_cells: 0,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.TableRow do
  @moduledoc """
  A table row.

  See `MDExNative.Comrak.Table`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()], header: boolean()}
  defstruct nodes: [], header: false, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.TableCell do
  @moduledoc """
  A table cell inside a table row.

  See `MDExNative.Comrak.Table`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Text do
  @moduledoc """
  Literal text.

  Spec: https://github.github.com/gfm/#textual-content
  """

  @type t :: %__MODULE__{literal: String.t()}
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.TaskItem do
  @moduledoc """
  A task item inside a list.

  Spec: https://github.github.com/gfm/#task-list-items-extension-
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          checked: boolean(),
          marker: String.t()
        }
  defstruct nodes: [], checked: false, marker: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.SoftBreak do
  @moduledoc """
  A soft line break.

  Spec: https://github.github.com/gfm/#soft-line-breaks
  """

  @type t :: %__MODULE__{}
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.LineBreak do
  @moduledoc """
  A hard line break.

  Spec: https://github.github.com/gfm/#hard-line-breaks
  """

  @type t :: %__MODULE__{}
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Code do
  @moduledoc """
  Inline code span.

  Spec: https://github.github.com/gfm/#code-spans
  """

  @type t :: %__MODULE__{
          num_backticks: non_neg_integer(),
          literal: String.t(),
          attrs: MDExNative.Comrak.Attributes.t() | nil
        }
  defstruct num_backticks: 0,
            literal: "",
            attrs: nil,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HtmlInline do
  @moduledoc """
  Raw HTML.

  Spec: https://github.github.com/gfm/#raw-html
  """

  @type t :: %__MODULE__{literal: String.t()}
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Raw do
  @moduledoc """
  A Raw output node. This will be inserted verbatim into CommonMark and HTML output. It can only be created programmatically, and is never parsed from input.
  """

  @type t :: %__MODULE__{literal: String.t()}
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Emph do
  @moduledoc """
  Emphasis.

  Spec: https://github.github.com/gfm/#emphasis-and-strong-emphasis
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Strong do
  @moduledoc """
  Strong emphasis.

  Spec: https://github.github.com/gfm/#emphasis-and-strong-emphasis
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Strikethrough do
  @moduledoc """
  Strikethrough.

  Spec: https://github.github.com/gfm/#strikethrough-extension-
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Highlight do
  @moduledoc """
  Highlight (mark) text.

  Uses double equals syntax: `==highlighted text==`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Insert do
  @moduledoc """
  Inserted text.

  Uses double plus syntax: `++inserted text++`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Superscript do
  @moduledoc """
  Superscript.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Link do
  @moduledoc """
  Link to a URL.

  Spec: https://github.github.com/gfm/#links
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          url: String.t(),
          title: String.t() | nil,
          attrs: MDExNative.Comrak.Attributes.t() | nil
        }
  defstruct nodes: [],
            url: "",
            title: nil,
            attrs: nil,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Image do
  @moduledoc """
  An image.

  Spec: https://github.github.com/gfm/#images
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          url: String.t(),
          title: String.t() | nil,
          attrs: MDExNative.Comrak.Attributes.t() | nil
        }
  defstruct nodes: [],
            url: "",
            title: nil,
            attrs: nil,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.ShortCode do
  @moduledoc """
  Emoji generated from a shortcode.
  """

  @type t :: %__MODULE__{code: String.t(), emoji: String.t()}
  defstruct code: "", emoji: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Math do
  @moduledoc """
  Inline math span.
  """

  @type t :: %__MODULE__{dollar_math: boolean(), display_math: boolean(), literal: String.t()}
  defstruct dollar_math: false,
            display_math: false,
            literal: "",
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.MultilineBlockQuote do
  @moduledoc """
  A multiline block quote.

  Spec: https://github.github.com/gfm/#block-quotes
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          fence_length: non_neg_integer(),
          fence_offset: non_neg_integer()
        }
  defstruct nodes: [], fence_length: 0, fence_offset: 0, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Escaped do
  @moduledoc """
  An escaped character.

  Spec: https://github.github.com/gfm/#backslash-escapes
  """

  @type t :: %__MODULE__{}
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.WikiLink do
  @moduledoc """
  A link in the form of a wiki link.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()], url: String.t()}
  defstruct nodes: [], url: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Underline do
  @moduledoc """
  Underline.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Subscript do
  @moduledoc """
  Subscript.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.SpoileredText do
  @moduledoc """
  Spoilered text.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Subtext do
  @moduledoc """
  Discord-style subtext.

  Uses curly braces with hyphens syntax: `{-text-}`
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()]}
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.EscapedTag do
  @moduledoc """
  Escaped tag.
  """

  @type t :: %__MODULE__{nodes: [MDExNative.Comrak.ast_node()], literal: String.t()}
  defstruct nodes: [], literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Alert do
  @moduledoc """
  GitHub and GitLab style alerts / admonitions.

  See [GitHub](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts)
  and [GitLab](https://docs.gitlab.com/user/markdown/#alerts) docs.
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          alert_type: :note | :tip | :important | :warning | :caution,
          title: String.t() | nil,
          multiline: boolean(),
          fence_length: non_neg_integer(),
          fence_offset: non_neg_integer()
        }
  defstruct nodes: [],
            alert_type: :note,
            title: nil,
            multiline: false,
            fence_length: 0,
            fence_offset: 0,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.BlockDirective do
  @moduledoc """
  Container block directive.

  Uses `:::` syntax to create container blocks:

      :::warning
      A paragraph.

      - item one
      - item two
      :::

  Renders as `<div class="warning">...</div>`.
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          info: String.t(),
          fence_length: non_neg_integer(),
          fence_offset: non_neg_integer()
        }
  defstruct nodes: [],
            info: "",
            fence_length: 0,
            fence_offset: 0,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HeexBlock do
  @moduledoc """
  Phoenix LiveView HEEx block-level element.

  Used for HEEx components, directives, comments, and expressions in Markdown.
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          literal: String.t(),
          node: String.t()
        }
  defstruct nodes: [], literal: "", node: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HeexInline do
  @moduledoc """
  Phoenix LiveView HEEx inline element.

  Used for inline HEEx expressions in Markdown.
  """

  @type t :: %__MODULE__{literal: String.t()}
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end
