defmodule MDExNative.Comrak.Sourcepos do
  @moduledoc """
  Source position information for AST nodes.
  """

  @type t :: %__MODULE__{
          start: {pos_integer(), pos_integer()},
          end: {pos_integer(), pos_integer()}
        }
  defstruct start: {0, 0}, end: {0, 0}
end

defmodule MDExNative.Comrak.Document do
  @moduledoc """
  Root of a parsed Markdown document.
  """

  @type t :: %__MODULE__{
          nodes: [MDExNative.Comrak.ast_node()],
          sourcepos: MDExNative.Comrak.Sourcepos.t()
        }
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.FrontMatter do
  @moduledoc false
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.BlockQuote do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.List do
  @moduledoc false
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
  @moduledoc false
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
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionItem do
  @moduledoc false
  defstruct nodes: [],
            marker_offset: 0,
            padding: 0,
            tight: false,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionTerm do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.DescriptionDetails do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.CodeBlock do
  @moduledoc false
  defstruct nodes: [],
            fenced: true,
            fence_char: "`",
            fence_length: 3,
            fence_offset: 0,
            info: "",
            literal: "",
            closed: true,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HtmlBlock do
  @moduledoc false
  defstruct nodes: [], block_type: 0, literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Paragraph do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Heading do
  @moduledoc false
  defstruct nodes: [],
            level: 1,
            setext: false,
            closed: false,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.ThematicBreak do
  @moduledoc false
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.FootnoteDefinition do
  @moduledoc false
  defstruct nodes: [], name: "", total_references: 0, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.FootnoteReference do
  @moduledoc false
  defstruct name: "", ref_num: nil, ix: nil, texts: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Table do
  @moduledoc false
  defstruct nodes: [],
            alignments: [],
            num_columns: 0,
            num_rows: 0,
            num_nonempty_cells: 0,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.TableRow do
  @moduledoc false
  defstruct nodes: [], header: false, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.TableCell do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Text do
  @moduledoc false
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.TaskItem do
  @moduledoc false
  defstruct nodes: [], checked: false, marker: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.SoftBreak do
  @moduledoc false
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.LineBreak do
  @moduledoc false
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Code do
  @moduledoc false
  defstruct num_backticks: 0, literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HtmlInline do
  @moduledoc false
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Raw do
  @moduledoc false
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Emph do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Strong do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Strikethrough do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Highlight do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Insert do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Superscript do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Link do
  @moduledoc false
  defstruct nodes: [], url: "", title: nil, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Image do
  @moduledoc false
  defstruct nodes: [], url: "", title: nil, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.ShortCode do
  @moduledoc false
  defstruct code: "", emoji: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Math do
  @moduledoc false
  defstruct dollar_math: false,
            display_math: false,
            literal: "",
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.MultilineBlockQuote do
  @moduledoc false
  defstruct nodes: [], fence_length: 0, fence_offset: 0, sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Escaped do
  @moduledoc false
  defstruct sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.WikiLink do
  @moduledoc false
  defstruct nodes: [], url: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Underline do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Subscript do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.SpoileredText do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Subtext do
  @moduledoc false
  defstruct nodes: [], sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.EscapedTag do
  @moduledoc false
  defstruct nodes: [], literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.Alert do
  @moduledoc false
  defstruct nodes: [],
            alert_type: :note,
            title: nil,
            multiline: false,
            fence_length: 0,
            fence_offset: 0,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.BlockDirective do
  @moduledoc false
  defstruct nodes: [],
            info: "",
            fence_length: 0,
            fence_offset: 0,
            sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HeexBlock do
  @moduledoc false
  defstruct nodes: [], literal: "", node: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end

defmodule MDExNative.Comrak.HeexInline do
  @moduledoc false
  defstruct literal: "", sourcepos: %MDExNative.Comrak.Sourcepos{}
end
