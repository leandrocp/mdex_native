use comrak::nodes::{
    AstNode, Attributes, HeexNode, LineColumn, NodeHeexBlock, NodeTaskItem, NodeValue, Sourcepos,
};
use typed_arena::Arena as TypedArena;

mod atoms {
    rustler::atoms! {
        bullet,
        ordered,
        period,
        paren,
        none,
        left,
        center,
        right
    }
}

#[derive(Clone, Copy, Debug, Default, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Sourcepos"]
pub struct ExSourcepos {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl From<Sourcepos> for ExSourcepos {
    fn from(sourcepos: Sourcepos) -> Self {
        Self {
            start: (sourcepos.start.line, sourcepos.start.column),
            end: (sourcepos.end.line, sourcepos.end.column),
        }
    }
}

impl From<ExSourcepos> for Sourcepos {
    fn from(sourcepos: ExSourcepos) -> Self {
        Self {
            start: LineColumn {
                line: sourcepos.start.0,
                column: sourcepos.start.1,
            },
            end: LineColumn {
                line: sourcepos.end.0,
                column: sourcepos.end.1,
            },
        }
    }
}

#[derive(Clone, Debug, Default, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Attributes"]
pub struct ExAttributes {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub pairs: Vec<(String, String)>,
}

impl From<ExAttributes> for Attributes {
    fn from(attributes: ExAttributes) -> Self {
        Self {
            id: attributes.id,
            classes: attributes.classes,
            pairs: attributes.pairs,
        }
    }
}

impl From<&Attributes> for ExAttributes {
    fn from(attributes: &Attributes) -> Self {
        Self {
            id: attributes.id.clone(),
            classes: attributes.classes.clone(),
            pairs: attributes.pairs.clone(),
        }
    }
}

// https://docs.rs/mdex_native/latest/mdex_native/nodes/enum.NodeValue.html
#[derive(Clone, Debug, NifUntaggedEnum, PartialEq)]
pub enum NewNode {
    Document(ExDocument),
    FrontMatter(ExFrontMatter),
    BlockQuote(ExBlockQuote),
    List(ExList),
    ListItem(ExListItem),
    DescriptionList(ExDescriptionList),
    DescriptionItem(ExDescriptionItem),
    DescriptionTerm(ExDescriptionTerm),
    DescriptionDetails(ExDescriptionDetails),
    CodeBlock(ExCodeBlock),
    HtmlBlock(ExHtmlBlock),
    Paragraph(ExParagraph),
    Heading(ExHeading),
    ThematicBreak(ExThematicBreak),
    FootnoteDefinition(ExFootnoteDefinition),
    FootnoteReference(ExFootnoteReference),
    Table(ExTable),
    TableRow(ExTableRow),
    TableCell(ExTableCell),
    Text(ExText),
    TaskItem(ExTaskItem),
    SoftBreak(ExSoftBreak),
    LineBreak(ExLineBreak),
    Code(ExCode),
    HtmlInline(ExHtmlInline),
    Raw(ExRaw),
    Emph(ExEmph),
    Strong(ExStrong),
    Strikethrough(ExStrikethrough),
    Highlight(ExHighlight),
    Insert(ExInsert),
    Superscript(ExSuperscript),
    Link(ExLink),
    Image(ExImage),
    ShortCode(ExShortCode),
    Math(ExMath),
    MultilineBlockQuote(ExMultilineBlockQuote),
    Escaped(ExEscaped),
    WikiLink(ExWikiLink),
    Underline(ExUnderline),
    Subscript(ExSubscript),
    SpoileredText(ExSpoileredText),
    Subtext(ExSubtext),
    EscapedTag(ExEscapedTag),
    Alert(ExAlert),
    HeexBlock(ExHeexBlock),
    HeexInline(ExHeexInline),
    BlockDirective(ExBlockDirective),
}

impl NewNode {
    fn sourcepos(&self) -> ExSourcepos {
        match self {
            Self::Document(ExDocument { sourcepos, .. })
            | Self::FrontMatter(ExFrontMatter { sourcepos, .. })
            | Self::BlockQuote(ExBlockQuote { sourcepos, .. })
            | Self::List(ExList { sourcepos, .. })
            | Self::ListItem(ExListItem { sourcepos, .. })
            | Self::DescriptionList(ExDescriptionList { sourcepos, .. })
            | Self::DescriptionItem(ExDescriptionItem { sourcepos, .. })
            | Self::DescriptionTerm(ExDescriptionTerm { sourcepos, .. })
            | Self::DescriptionDetails(ExDescriptionDetails { sourcepos, .. })
            | Self::CodeBlock(ExCodeBlock { sourcepos, .. })
            | Self::HtmlBlock(ExHtmlBlock { sourcepos, .. })
            | Self::Paragraph(ExParagraph { sourcepos, .. })
            | Self::Heading(ExHeading { sourcepos, .. })
            | Self::ThematicBreak(ExThematicBreak { sourcepos })
            | Self::FootnoteDefinition(ExFootnoteDefinition { sourcepos, .. })
            | Self::FootnoteReference(ExFootnoteReference { sourcepos, .. })
            | Self::Table(ExTable { sourcepos, .. })
            | Self::TableRow(ExTableRow { sourcepos, .. })
            | Self::TableCell(ExTableCell { sourcepos, .. })
            | Self::Text(ExText { sourcepos, .. })
            | Self::TaskItem(ExTaskItem { sourcepos, .. })
            | Self::SoftBreak(ExSoftBreak { sourcepos })
            | Self::LineBreak(ExLineBreak { sourcepos })
            | Self::Code(ExCode { sourcepos, .. })
            | Self::HtmlInline(ExHtmlInline { sourcepos, .. })
            | Self::Raw(ExRaw { sourcepos, .. })
            | Self::Emph(ExEmph { sourcepos, .. })
            | Self::Strong(ExStrong { sourcepos, .. })
            | Self::Strikethrough(ExStrikethrough { sourcepos, .. })
            | Self::Highlight(ExHighlight { sourcepos, .. })
            | Self::Insert(ExInsert { sourcepos, .. })
            | Self::Superscript(ExSuperscript { sourcepos, .. })
            | Self::Link(ExLink { sourcepos, .. })
            | Self::Image(ExImage { sourcepos, .. })
            | Self::ShortCode(ExShortCode { sourcepos, .. })
            | Self::Math(ExMath { sourcepos, .. })
            | Self::MultilineBlockQuote(ExMultilineBlockQuote { sourcepos, .. })
            | Self::Escaped(ExEscaped { sourcepos })
            | Self::WikiLink(ExWikiLink { sourcepos, .. })
            | Self::Underline(ExUnderline { sourcepos, .. })
            | Self::Subscript(ExSubscript { sourcepos, .. })
            | Self::SpoileredText(ExSpoileredText { sourcepos, .. })
            | Self::Subtext(ExSubtext { sourcepos, .. })
            | Self::EscapedTag(ExEscapedTag { sourcepos, .. })
            | Self::Alert(ExAlert { sourcepos, .. })
            | Self::HeexBlock(ExHeexBlock { sourcepos, .. })
            | Self::HeexInline(ExHeexInline { sourcepos, .. })
            | Self::BlockDirective(ExBlockDirective { sourcepos, .. }) => *sourcepos,
        }
    }

    fn take_children(&mut self) -> Option<Vec<NewNode>> {
        match self {
            Self::EscapedTag(ExEscapedTag { nodes, literal, .. }) => {
                comrak_escaped_tag_literal(literal).map(|_| std::mem::take(nodes))
            }
            Self::Document(ExDocument { nodes, .. })
            | Self::BlockQuote(ExBlockQuote { nodes, .. })
            | Self::List(ExList { nodes, .. })
            | Self::ListItem(ExListItem { nodes, .. })
            | Self::DescriptionList(ExDescriptionList { nodes, .. })
            | Self::DescriptionItem(ExDescriptionItem { nodes, .. })
            | Self::DescriptionTerm(ExDescriptionTerm { nodes, .. })
            | Self::DescriptionDetails(ExDescriptionDetails { nodes, .. })
            | Self::CodeBlock(ExCodeBlock { nodes, .. })
            | Self::HtmlBlock(ExHtmlBlock { nodes, .. })
            | Self::Paragraph(ExParagraph { nodes, .. })
            | Self::Heading(ExHeading { nodes, .. })
            | Self::FootnoteDefinition(ExFootnoteDefinition { nodes, .. })
            | Self::Table(ExTable { nodes, .. })
            | Self::TableRow(ExTableRow { nodes, .. })
            | Self::TableCell(ExTableCell { nodes, .. })
            | Self::TaskItem(ExTaskItem { nodes, .. })
            | Self::Emph(ExEmph { nodes, .. })
            | Self::Strong(ExStrong { nodes, .. })
            | Self::Strikethrough(ExStrikethrough { nodes, .. })
            | Self::Highlight(ExHighlight { nodes, .. })
            | Self::Insert(ExInsert { nodes, .. })
            | Self::Superscript(ExSuperscript { nodes, .. })
            | Self::Link(ExLink { nodes, .. })
            | Self::Image(ExImage { nodes, .. })
            | Self::MultilineBlockQuote(ExMultilineBlockQuote { nodes, .. })
            | Self::WikiLink(ExWikiLink { nodes, .. })
            | Self::Underline(ExUnderline { nodes, .. })
            | Self::Subscript(ExSubscript { nodes, .. })
            | Self::SpoileredText(ExSpoileredText { nodes, .. })
            | Self::Subtext(ExSubtext { nodes, .. })
            | Self::Alert(ExAlert { nodes, .. })
            | Self::HeexBlock(ExHeexBlock { nodes, .. })
            | Self::BlockDirective(ExBlockDirective { nodes, .. }) => Some(std::mem::take(nodes)),
            _ => None,
        }
    }

    fn take_attrs(&mut self) -> Option<ExAttributes> {
        match self {
            Self::CodeBlock(ExCodeBlock { attrs, .. })
            | Self::Heading(ExHeading { attrs, .. })
            | Self::Code(ExCode { attrs, .. })
            | Self::Link(ExLink { attrs, .. })
            | Self::Image(ExImage { attrs, .. }) => attrs.take(),
            _ => None,
        }
    }
}

impl From<NewNode> for NodeValue {
    fn from(node: NewNode) -> Self {
        match node {
            NewNode::Document(n) => n.into(),
            NewNode::FrontMatter(n) => n.into(),
            NewNode::BlockQuote(n) => n.into(),
            NewNode::List(n) => n.into(),
            NewNode::ListItem(n) => n.into(),
            NewNode::DescriptionList(n) => n.into(),
            NewNode::DescriptionItem(n) => n.into(),
            NewNode::DescriptionTerm(n) => n.into(),
            NewNode::DescriptionDetails(n) => n.into(),
            NewNode::CodeBlock(n) => n.into(),
            NewNode::HtmlBlock(n) => n.into(),
            NewNode::Paragraph(n) => n.into(),
            NewNode::Heading(n) => n.into(),
            NewNode::ThematicBreak(n) => n.into(),
            NewNode::FootnoteDefinition(n) => n.into(),
            NewNode::FootnoteReference(n) => n.into(),
            NewNode::Table(n) => n.into(),
            NewNode::TableRow(n) => n.into(),
            NewNode::TableCell(n) => n.into(),
            NewNode::Text(n) => n.into(),
            NewNode::TaskItem(n) => n.into(),
            NewNode::SoftBreak(n) => n.into(),
            NewNode::LineBreak(n) => n.into(),
            NewNode::Code(n) => n.into(),
            NewNode::HtmlInline(n) => n.into(),
            NewNode::Raw(n) => n.into(),
            NewNode::Emph(n) => n.into(),
            NewNode::Strong(n) => n.into(),
            NewNode::Strikethrough(n) => n.into(),
            NewNode::Highlight(n) => n.into(),
            NewNode::Insert(n) => n.into(),
            NewNode::Superscript(n) => n.into(),
            NewNode::Link(n) => n.into(),
            NewNode::Image(n) => n.into(),
            NewNode::ShortCode(n) => n.into(),
            NewNode::Math(n) => n.into(),
            NewNode::MultilineBlockQuote(n) => n.into(),
            NewNode::Escaped(n) => n.into(),
            NewNode::WikiLink(n) => n.into(),
            NewNode::Underline(n) => n.into(),
            NewNode::Subscript(n) => n.into(),
            NewNode::SpoileredText(n) => n.into(),
            NewNode::Subtext(n) => n.into(),
            NewNode::EscapedTag(n) => n.into(),
            NewNode::Alert(n) => n.into(),
            NewNode::HeexBlock(n) => n.into(),
            NewNode::HeexInline(n) => n.into(),
            NewNode::BlockDirective(n) => n.into(),
        }
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Document"]
pub struct ExDocument {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExDocument> for NodeValue {
    fn from(_node: ExDocument) -> Self {
        NodeValue::Document
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.FrontMatter"]
pub struct ExFrontMatter {
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExFrontMatter> for NodeValue {
    fn from(node: ExFrontMatter) -> Self {
        NodeValue::FrontMatter(node.literal)
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.BlockQuote"]
pub struct ExBlockQuote {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExBlockQuote> for NodeValue {
    fn from(_node: ExBlockQuote) -> Self {
        NodeValue::BlockQuote
    }
}

#[derive(Clone, Debug, NifUnitEnum, PartialEq)]
pub enum ExListType {
    Bullet,
    Ordered,
}

#[derive(Clone, Debug, NifUnitEnum, PartialEq)]
pub enum ExListDelimType {
    Period,
    Paren,
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.List"]
pub struct ExList {
    pub nodes: Vec<NewNode>,
    pub list_type: ExListType,
    pub marker_offset: usize,
    pub padding: usize,
    pub start: usize,
    pub delimiter: ExListDelimType,
    pub bullet_char: String,
    pub tight: bool,
    pub is_task_list: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExList> for NodeValue {
    fn from(node: ExList) -> Self {
        NodeValue::List(comrak::nodes::NodeList {
            list_type: match node.list_type {
                ExListType::Bullet => comrak::nodes::ListType::Bullet,
                ExListType::Ordered => comrak::nodes::ListType::Ordered,
            },
            marker_offset: node.marker_offset,
            padding: node.padding,
            start: node.start,
            delimiter: match node.delimiter {
                ExListDelimType::Period => comrak::nodes::ListDelimType::Period,
                ExListDelimType::Paren => comrak::nodes::ListDelimType::Paren,
            },
            bullet_char: string_to_char(node.bullet_char),
            tight: node.tight,
            is_task_list: node.is_task_list,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.ListItem"]
pub struct ExListItem {
    pub nodes: Vec<NewNode>,
    pub list_type: ExListType,
    pub marker_offset: usize,
    pub padding: usize,
    pub start: usize,
    pub delimiter: ExListDelimType,
    pub bullet_char: String,
    pub tight: bool,
    pub is_task_list: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExListItem> for NodeValue {
    fn from(node: ExListItem) -> Self {
        NodeValue::Item(comrak::nodes::NodeList {
            list_type: match node.list_type {
                ExListType::Bullet => comrak::nodes::ListType::Bullet,
                ExListType::Ordered => comrak::nodes::ListType::Ordered,
            },
            marker_offset: node.marker_offset,
            padding: node.padding,
            start: node.start,
            delimiter: match node.delimiter {
                ExListDelimType::Period => comrak::nodes::ListDelimType::Period,
                ExListDelimType::Paren => comrak::nodes::ListDelimType::Paren,
            },
            bullet_char: string_to_char(node.bullet_char),
            tight: node.tight,
            is_task_list: node.is_task_list,
        })
    }
}
#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.DescriptionList"]
pub struct ExDescriptionList {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExDescriptionList> for NodeValue {
    fn from(_node: ExDescriptionList) -> Self {
        NodeValue::DescriptionList
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.DescriptionItem"]
pub struct ExDescriptionItem {
    pub nodes: Vec<NewNode>,
    pub marker_offset: usize,
    pub padding: usize,
    pub tight: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExDescriptionItem> for NodeValue {
    fn from(node: ExDescriptionItem) -> Self {
        NodeValue::DescriptionItem(comrak::nodes::NodeDescriptionItem {
            marker_offset: node.marker_offset,
            padding: node.padding,
            tight: node.tight,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.DescriptionTerm"]
pub struct ExDescriptionTerm {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExDescriptionTerm> for NodeValue {
    fn from(_node: ExDescriptionTerm) -> Self {
        NodeValue::DescriptionTerm
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.DescriptionDetails"]
pub struct ExDescriptionDetails {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExDescriptionDetails> for NodeValue {
    fn from(_node: ExDescriptionDetails) -> Self {
        NodeValue::DescriptionDetails
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.CodeBlock"]
pub struct ExCodeBlock {
    pub nodes: Vec<NewNode>,
    pub fenced: bool,
    pub fence_char: String,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub info: String,
    pub literal: String,
    pub closed: bool,
    pub attrs: Option<ExAttributes>,
    pub sourcepos: ExSourcepos,
}

impl From<ExCodeBlock> for NodeValue {
    fn from(node: ExCodeBlock) -> Self {
        NodeValue::CodeBlock(Box::new(comrak::nodes::NodeCodeBlock {
            fenced: node.fenced,
            fence_char: string_to_char(node.fence_char),
            fence_length: node.fence_length,
            fence_offset: node.fence_offset,
            info: node.info,
            literal: node.literal,
            closed: node.closed,
        }))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.HtmlBlock"]
pub struct ExHtmlBlock {
    pub nodes: Vec<NewNode>,
    pub block_type: u8,
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExHtmlBlock> for NodeValue {
    fn from(node: ExHtmlBlock) -> Self {
        NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
            block_type: node.block_type,
            literal: node.literal,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Paragraph"]
pub struct ExParagraph {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExParagraph> for NodeValue {
    fn from(_node: ExParagraph) -> Self {
        NodeValue::Paragraph
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Heading"]
pub struct ExHeading {
    pub nodes: Vec<NewNode>,
    pub level: u8,
    pub setext: bool,
    pub closed: bool,
    pub attrs: Option<ExAttributes>,
    pub sourcepos: ExSourcepos,
}

impl From<ExHeading> for NodeValue {
    fn from(node: ExHeading) -> Self {
        NodeValue::Heading(comrak::nodes::NodeHeading {
            level: node.level,
            setext: node.setext,
            closed: node.closed,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.ThematicBreak"]
pub struct ExThematicBreak {
    pub sourcepos: ExSourcepos,
}

impl From<ExThematicBreak> for NodeValue {
    fn from(_node: ExThematicBreak) -> Self {
        NodeValue::ThematicBreak
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.FootnoteDefinition"]
pub struct ExFootnoteDefinition {
    pub nodes: Vec<NewNode>,
    pub name: String,
    pub total_references: u32,
    pub sourcepos: ExSourcepos,
}

impl From<ExFootnoteDefinition> for NodeValue {
    fn from(node: ExFootnoteDefinition) -> Self {
        NodeValue::FootnoteDefinition(comrak::nodes::NodeFootnoteDefinition {
            name: node.name.to_string(),
            total_references: node.total_references,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.FootnoteReference"]
pub struct ExFootnoteReference {
    pub name: String,
    pub ref_num: u32,
    pub ix: u32,
    pub texts: Vec<(String, usize)>,
    pub sourcepos: ExSourcepos,
}

impl From<ExFootnoteReference> for NodeValue {
    fn from(node: ExFootnoteReference) -> Self {
        NodeValue::FootnoteReference(Box::new(comrak::nodes::NodeFootnoteReference {
            name: node.name.to_string(),
            ref_num: node.ref_num,
            ix: node.ix,
            texts: node.texts,
        }))
    }
}

#[derive(Clone, Debug, NifUnitEnum, PartialEq)]
pub enum ExTableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Table"]
pub struct ExTable {
    pub nodes: Vec<NewNode>,
    pub alignments: Vec<ExTableAlignment>,
    pub num_columns: usize,
    pub num_rows: usize,
    pub num_nonempty_cells: usize,
    pub sourcepos: ExSourcepos,
}

impl From<ExTable> for NodeValue {
    fn from(node: ExTable) -> Self {
        NodeValue::Table(Box::new(comrak::nodes::NodeTable {
            alignments: node
                .alignments
                .into_iter()
                .map(|a| match a {
                    ExTableAlignment::None => comrak::nodes::TableAlignment::None,
                    ExTableAlignment::Left => comrak::nodes::TableAlignment::Left,
                    ExTableAlignment::Center => comrak::nodes::TableAlignment::Center,
                    ExTableAlignment::Right => comrak::nodes::TableAlignment::Right,
                })
                .collect(),
            num_columns: node.num_columns,
            num_rows: node.num_rows,
            num_nonempty_cells: node.num_nonempty_cells,
        }))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.TableRow"]
pub struct ExTableRow {
    pub nodes: Vec<NewNode>,
    pub header: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExTableRow> for NodeValue {
    fn from(node: ExTableRow) -> Self {
        NodeValue::TableRow(node.header)
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.TableCell"]
pub struct ExTableCell {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExTableCell> for NodeValue {
    fn from(_node: ExTableCell) -> Self {
        NodeValue::TableCell
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Text"]
pub struct ExText {
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExText> for NodeValue {
    fn from(node: ExText) -> Self {
        NodeValue::Text(node.literal.into())
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.TaskItem"]
pub struct ExTaskItem {
    pub nodes: Vec<NewNode>,
    pub checked: bool,
    pub marker: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExTaskItem> for NodeValue {
    fn from(node: ExTaskItem) -> Self {
        NodeValue::TaskItem(NodeTaskItem {
            symbol: node.marker.chars().next(),
            symbol_sourcepos: Sourcepos {
                start: LineColumn::default(),
                end: LineColumn::default(),
            },
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.SoftBreak"]
pub struct ExSoftBreak {
    pub sourcepos: ExSourcepos,
}

impl From<ExSoftBreak> for NodeValue {
    fn from(_node: ExSoftBreak) -> Self {
        NodeValue::SoftBreak
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.LineBreak"]
pub struct ExLineBreak {
    pub sourcepos: ExSourcepos,
}

impl From<ExLineBreak> for NodeValue {
    fn from(_node: ExLineBreak) -> Self {
        NodeValue::LineBreak
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Code"]
pub struct ExCode {
    pub num_backticks: usize,
    pub literal: String,
    pub attrs: Option<ExAttributes>,
    pub sourcepos: ExSourcepos,
}

impl From<ExCode> for NodeValue {
    fn from(node: ExCode) -> Self {
        NodeValue::Code(comrak::nodes::NodeCode {
            num_backticks: node.num_backticks,
            literal: node.literal,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.HtmlInline"]
pub struct ExHtmlInline {
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExHtmlInline> for NodeValue {
    fn from(node: ExHtmlInline) -> Self {
        NodeValue::HtmlInline(node.literal.to_string())
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.HeexBlock"]
pub struct ExHeexBlock {
    pub nodes: Vec<NewNode>,
    pub literal: String,
    pub node: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExHeexBlock> for NodeValue {
    fn from(node: ExHeexBlock) -> Self {
        let heex_node = match node.node.as_str() {
            "directive" => HeexNode::Directive,
            "comment" => HeexNode::Comment,
            "multiline_comment" => HeexNode::MultilineComment,
            "expression" => HeexNode::Expression,
            tag => HeexNode::Tag(tag.to_string()),
        };
        NodeValue::HeexBlock(Box::new(NodeHeexBlock {
            literal: node.literal,
            node: heex_node,
        }))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.HeexInline"]
pub struct ExHeexInline {
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExHeexInline> for NodeValue {
    fn from(node: ExHeexInline) -> Self {
        NodeValue::HeexInline(node.literal)
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Raw"]
pub struct ExRaw {
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExRaw> for NodeValue {
    fn from(node: ExRaw) -> Self {
        NodeValue::Raw(node.literal.to_string())
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Emph"]
pub struct ExEmph {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExEmph> for NodeValue {
    fn from(_node: ExEmph) -> Self {
        NodeValue::Emph
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Strong"]
pub struct ExStrong {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExStrong> for NodeValue {
    fn from(_node: ExStrong) -> Self {
        NodeValue::Strong
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Strikethrough"]
pub struct ExStrikethrough {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExStrikethrough> for NodeValue {
    fn from(_node: ExStrikethrough) -> Self {
        NodeValue::Strikethrough
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Highlight"]
pub struct ExHighlight {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExHighlight> for NodeValue {
    fn from(_node: ExHighlight) -> Self {
        NodeValue::Highlight
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Insert"]
pub struct ExInsert {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExInsert> for NodeValue {
    fn from(_node: ExInsert) -> Self {
        NodeValue::Insert
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Superscript"]
pub struct ExSuperscript {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExSuperscript> for NodeValue {
    fn from(_node: ExSuperscript) -> Self {
        NodeValue::Superscript
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Link"]
pub struct ExLink {
    pub nodes: Vec<NewNode>,
    pub url: String,
    pub title: String,
    pub attrs: Option<ExAttributes>,
    pub sourcepos: ExSourcepos,
}

impl From<ExLink> for NodeValue {
    fn from(node: ExLink) -> Self {
        NodeValue::Link(Box::new(comrak::nodes::NodeLink {
            url: node.url,
            title: node.title,
        }))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Image"]
pub struct ExImage {
    pub nodes: Vec<NewNode>,
    pub url: String,
    pub title: String,
    pub attrs: Option<ExAttributes>,
    pub sourcepos: ExSourcepos,
}

impl From<ExImage> for NodeValue {
    fn from(node: ExImage) -> Self {
        NodeValue::Image(Box::new(comrak::nodes::NodeLink {
            url: node.url,
            title: node.title,
        }))
    }
}
#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.ShortCode"]
pub struct ExShortCode {
    pub code: String,
    pub emoji: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExShortCode> for NodeValue {
    fn from(node: ExShortCode) -> Self {
        NodeValue::ShortCode(Box::new(comrak::nodes::NodeShortCode {
            code: node.code.to_string(),
            emoji: node.emoji.to_string(),
        }))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Math"]
pub struct ExMath {
    pub dollar_math: bool,
    pub display_math: bool,
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExMath> for NodeValue {
    fn from(node: ExMath) -> Self {
        NodeValue::Math(comrak::nodes::NodeMath {
            dollar_math: node.dollar_math,
            display_math: node.display_math,
            literal: node.literal,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.MultilineBlockQuote"]
pub struct ExMultilineBlockQuote {
    pub nodes: Vec<NewNode>,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub sourcepos: ExSourcepos,
}

impl From<ExMultilineBlockQuote> for NodeValue {
    fn from(node: ExMultilineBlockQuote) -> Self {
        NodeValue::MultilineBlockQuote(comrak::nodes::NodeMultilineBlockQuote {
            fence_length: node.fence_length,
            fence_offset: node.fence_offset,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Escaped"]
pub struct ExEscaped {
    pub sourcepos: ExSourcepos,
}

impl From<ExEscaped> for NodeValue {
    fn from(_node: ExEscaped) -> Self {
        NodeValue::Escaped
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.WikiLink"]
pub struct ExWikiLink {
    pub nodes: Vec<NewNode>,
    pub url: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExWikiLink> for NodeValue {
    fn from(node: ExWikiLink) -> Self {
        NodeValue::WikiLink(comrak::nodes::NodeWikiLink {
            url: node.url.to_string(),
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Underline"]
pub struct ExUnderline {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExUnderline> for NodeValue {
    fn from(_node: ExUnderline) -> Self {
        NodeValue::Underline
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Subscript"]
pub struct ExSubscript {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExSubscript> for NodeValue {
    fn from(_node: ExSubscript) -> Self {
        NodeValue::Subscript
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.SpoileredText"]
pub struct ExSpoileredText {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExSpoileredText> for NodeValue {
    fn from(_node: ExSpoileredText) -> Self {
        NodeValue::SpoileredText
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Subtext"]
pub struct ExSubtext {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExSubtext> for NodeValue {
    fn from(_node: ExSubtext) -> Self {
        NodeValue::Subtext
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.EscapedTag"]
pub struct ExEscapedTag {
    pub nodes: Vec<NewNode>,
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExEscapedTag> for NodeValue {
    fn from(node: ExEscapedTag) -> Self {
        if let Some(literal) = comrak_escaped_tag_literal(&node.literal) {
            NodeValue::EscapedTag(literal)
        } else {
            NodeValue::Text(node.literal.into())
        }
    }
}

fn comrak_escaped_tag_literal(literal: &str) -> Option<&'static str> {
    match literal {
        "|" => Some("|"),
        "~" => Some("~"),
        "~~" => Some("~~"),
        "==" => Some("=="),
        "++" => Some("++"),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, NifUnitEnum, PartialEq)]
pub enum ExAlertType {
    #[default]
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Alert"]
pub struct ExAlert {
    pub nodes: Vec<NewNode>,
    pub alert_type: ExAlertType,
    pub title: Option<String>,
    pub multiline: bool,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub sourcepos: ExSourcepos,
}

impl From<ExAlert> for NodeValue {
    fn from(node: ExAlert) -> Self {
        NodeValue::Alert(Box::new(comrak::nodes::NodeAlert {
            alert_type: match node.alert_type {
                ExAlertType::Note => comrak::nodes::AlertType::Note,
                ExAlertType::Tip => comrak::nodes::AlertType::Tip,
                ExAlertType::Important => comrak::nodes::AlertType::Important,
                ExAlertType::Warning => comrak::nodes::AlertType::Warning,
                ExAlertType::Caution => comrak::nodes::AlertType::Caution,
            },
            title: node.title,
            multiline: node.multiline,
            fence_length: node.fence_length,
            fence_offset: node.fence_offset,
        }))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.BlockDirective"]
pub struct ExBlockDirective {
    pub nodes: Vec<NewNode>,
    pub info: String,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub sourcepos: ExSourcepos,
}

impl From<ExBlockDirective> for NodeValue {
    fn from(node: ExBlockDirective) -> Self {
        NodeValue::BlockDirective(Box::new(comrak::nodes::NodeBlockDirective {
            info: node.info,
            fence_length: node.fence_length,
            fence_offset: node.fence_offset,
        }))
    }
}

pub fn ex_document_to_comrak_ast<'a>(
    arena: &'a TypedArena<AstNode<'a>>,
    mut new_node: NewNode,
) -> &'a AstNode<'a> {
    let sourcepos = new_node.sourcepos();
    let children = new_node.take_children();
    let attrs = new_node.take_attrs();
    let node_value = NodeValue::from(new_node);
    let node_arena = arena.alloc(node_value.into());

    node_arena.data_mut().sourcepos = sourcepos.into();
    node_arena.data_mut().attrs = attrs.map(|attrs| Box::new(attrs.into()));

    if let Some(nodes) = children {
        for node in nodes {
            let child = ex_document_to_comrak_ast(arena, node);
            node_arena.append(child);
        }
    }

    node_arena
}

pub fn comrak_ast_to_ex_document_shallow<'a>(node: &'a AstNode<'a>) -> NewNode {
    comrak_ast_to_ex_document_with_children(node, Vec::new())
}

fn comrak_ast_to_ex_document_with_children<'a>(
    node: &'a AstNode<'a>,
    children: Vec<NewNode>,
) -> NewNode {
    let node_data = node.data();
    let sourcepos = node_data.sourcepos.into();

    match &node_data.value {
        NodeValue::Document => NewNode::Document(ExDocument {
            nodes: children,
            sourcepos,
        }),

        NodeValue::FrontMatter(ref literal) => NewNode::FrontMatter(ExFrontMatter {
            literal: literal.to_string(),
            sourcepos,
        }),

        NodeValue::BlockQuote => NewNode::BlockQuote(ExBlockQuote {
            nodes: children,
            sourcepos,
        }),

        NodeValue::List(ref attrs) => NewNode::List(ExList {
            nodes: children,
            list_type: match attrs.list_type {
                comrak::nodes::ListType::Bullet => ExListType::Bullet,
                comrak::nodes::ListType::Ordered => ExListType::Ordered,
            },
            marker_offset: attrs.marker_offset,
            padding: attrs.padding,
            start: attrs.start,
            delimiter: match attrs.delimiter {
                comrak::nodes::ListDelimType::Period => ExListDelimType::Period,
                comrak::nodes::ListDelimType::Paren => ExListDelimType::Paren,
            },
            bullet_char: char_to_string(attrs.bullet_char).unwrap_or_default(),
            tight: attrs.tight,
            is_task_list: attrs.is_task_list,
            sourcepos,
        }),

        NodeValue::Item(ref attrs) => NewNode::ListItem(ExListItem {
            nodes: children,
            list_type: match attrs.list_type {
                comrak::nodes::ListType::Bullet => ExListType::Bullet,
                comrak::nodes::ListType::Ordered => ExListType::Ordered,
            },
            marker_offset: attrs.marker_offset,
            padding: attrs.padding,
            start: attrs.start,
            delimiter: match attrs.delimiter {
                comrak::nodes::ListDelimType::Period => ExListDelimType::Period,
                comrak::nodes::ListDelimType::Paren => ExListDelimType::Paren,
            },
            bullet_char: char_to_string(attrs.bullet_char).unwrap_or_default(),
            tight: attrs.tight,
            is_task_list: attrs.is_task_list,
            sourcepos,
        }),

        NodeValue::DescriptionList => NewNode::DescriptionList(ExDescriptionList {
            nodes: children,
            sourcepos,
        }),

        NodeValue::DescriptionItem(ref attrs) => NewNode::DescriptionItem(ExDescriptionItem {
            nodes: children,
            marker_offset: attrs.marker_offset,
            padding: attrs.padding,
            tight: attrs.tight,
            sourcepos,
        }),

        NodeValue::DescriptionTerm => NewNode::DescriptionTerm(ExDescriptionTerm {
            nodes: children,
            sourcepos,
        }),

        NodeValue::DescriptionDetails => NewNode::DescriptionDetails(ExDescriptionDetails {
            nodes: children,
            sourcepos,
        }),

        NodeValue::CodeBlock(attrs) => NewNode::CodeBlock(ExCodeBlock {
            nodes: children,
            fenced: attrs.fenced,
            fence_char: char_to_string(attrs.fence_char).unwrap_or_default(),
            fence_length: attrs.fence_length,
            fence_offset: attrs.fence_offset,
            info: attrs.info.to_string(),
            literal: attrs.literal.to_string(),
            closed: attrs.closed,
            attrs: node_data.attrs.as_deref().map(Into::into),
            sourcepos,
        }),

        NodeValue::HtmlBlock(ref attrs) => NewNode::HtmlBlock(ExHtmlBlock {
            nodes: children,
            block_type: attrs.block_type,
            literal: attrs.literal.to_string(),
            sourcepos,
        }),

        NodeValue::Paragraph => NewNode::Paragraph(ExParagraph {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Heading(ref attrs) => NewNode::Heading(ExHeading {
            nodes: children,
            level: attrs.level,
            setext: attrs.setext,
            closed: attrs.closed,
            attrs: node_data.attrs.as_deref().map(Into::into),
            sourcepos,
        }),

        NodeValue::ThematicBreak => NewNode::ThematicBreak(ExThematicBreak { sourcepos }),

        NodeValue::FootnoteDefinition(ref attrs) => {
            NewNode::FootnoteDefinition(ExFootnoteDefinition {
                nodes: children,
                name: attrs.name.to_string(),
                total_references: attrs.total_references,
                sourcepos,
            })
        }

        NodeValue::FootnoteReference(ref attrs) => {
            NewNode::FootnoteReference(ExFootnoteReference {
                name: attrs.name.to_string(),
                ref_num: attrs.ref_num,
                ix: attrs.ix,
                texts: attrs.texts.clone(),
                sourcepos,
            })
        }

        NodeValue::Table(attrs) => NewNode::Table(ExTable {
            nodes: children,
            alignments: attrs
                .alignments
                .iter()
                .map(|a| match a {
                    comrak::nodes::TableAlignment::None => ExTableAlignment::None,
                    comrak::nodes::TableAlignment::Left => ExTableAlignment::Left,
                    comrak::nodes::TableAlignment::Center => ExTableAlignment::Center,
                    comrak::nodes::TableAlignment::Right => ExTableAlignment::Right,
                })
                .collect(),
            num_columns: attrs.num_columns,
            num_rows: attrs.num_rows,
            num_nonempty_cells: attrs.num_nonempty_cells,
            sourcepos,
        }),

        NodeValue::TableRow(header) => NewNode::TableRow(ExTableRow {
            nodes: children,
            header: *header,
            sourcepos,
        }),

        NodeValue::TableCell => NewNode::TableCell(ExTableCell {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Text(ref literal) => NewNode::Text(ExText {
            literal: literal.to_string(),
            sourcepos,
        }),

        NodeValue::TaskItem(marker) => NewNode::TaskItem(ExTaskItem {
            nodes: children,
            checked: marker.symbol.is_some(),
            marker: marker.symbol.map_or_else(String::new, |c| c.to_string()),
            sourcepos,
        }),

        NodeValue::SoftBreak => NewNode::SoftBreak(ExSoftBreak { sourcepos }),

        NodeValue::LineBreak => NewNode::LineBreak(ExLineBreak { sourcepos }),

        NodeValue::Code(ref attrs) => NewNode::Code(ExCode {
            num_backticks: attrs.num_backticks,
            literal: attrs.literal.to_string(),
            attrs: node_data.attrs.as_deref().map(Into::into),
            sourcepos,
        }),

        NodeValue::HtmlInline(ref literal) => NewNode::HtmlInline(ExHtmlInline {
            literal: literal.to_string(),
            sourcepos,
        }),

        NodeValue::Raw(ref literal) => NewNode::Raw(ExRaw {
            literal: literal.to_string(),
            sourcepos,
        }),

        NodeValue::Emph => NewNode::Emph(ExEmph {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Strong => NewNode::Strong(ExStrong {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Strikethrough => NewNode::Strikethrough(ExStrikethrough {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Highlight => NewNode::Highlight(ExHighlight {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Insert => NewNode::Insert(ExInsert {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Superscript => NewNode::Superscript(ExSuperscript {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Link(attrs) => NewNode::Link(ExLink {
            nodes: children,
            url: attrs.url.to_string(),
            title: attrs.title.to_string(),
            attrs: node_data.attrs.as_deref().map(Into::into),
            sourcepos,
        }),

        NodeValue::Image(attrs) => NewNode::Image(ExImage {
            nodes: children,
            url: attrs.url.to_string(),
            title: attrs.title.to_string(),
            attrs: node_data.attrs.as_deref().map(Into::into),
            sourcepos,
        }),

        NodeValue::ShortCode(attrs) => NewNode::ShortCode(ExShortCode {
            code: attrs.code.to_string(),
            emoji: attrs.emoji.to_string(),
            sourcepos,
        }),

        NodeValue::Math(ref attrs) => NewNode::Math(ExMath {
            dollar_math: attrs.dollar_math,
            display_math: attrs.display_math,
            literal: attrs.literal.to_string(),
            sourcepos,
        }),

        NodeValue::MultilineBlockQuote(ref attrs) => {
            NewNode::MultilineBlockQuote(ExMultilineBlockQuote {
                nodes: children,
                fence_length: attrs.fence_length,
                fence_offset: attrs.fence_offset,
                sourcepos,
            })
        }

        NodeValue::Escaped => NewNode::Escaped(ExEscaped { sourcepos }),

        NodeValue::WikiLink(ref attrs) => NewNode::WikiLink(ExWikiLink {
            nodes: children,
            url: attrs.url.to_string(),
            sourcepos,
        }),

        NodeValue::Underline => NewNode::Underline(ExUnderline {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Subscript => NewNode::Subscript(ExSubscript {
            nodes: children,
            sourcepos,
        }),

        NodeValue::SpoileredText => NewNode::SpoileredText(ExSpoileredText {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Subtext => NewNode::Subtext(ExSubtext {
            nodes: children,
            sourcepos,
        }),

        NodeValue::EscapedTag(ref literal) => NewNode::EscapedTag(ExEscapedTag {
            nodes: children,
            literal: literal.to_string(),
            sourcepos,
        }),

        NodeValue::Alert(attrs) => NewNode::Alert(ExAlert {
            nodes: children,
            alert_type: match attrs.alert_type {
                comrak::nodes::AlertType::Note => ExAlertType::Note,
                comrak::nodes::AlertType::Tip => ExAlertType::Tip,
                comrak::nodes::AlertType::Important => ExAlertType::Important,
                comrak::nodes::AlertType::Warning => ExAlertType::Warning,
                comrak::nodes::AlertType::Caution => ExAlertType::Caution,
            },
            title: attrs.title.to_owned(),
            multiline: attrs.multiline,
            fence_length: attrs.fence_length,
            fence_offset: attrs.fence_offset,
            sourcepos,
        }),

        NodeValue::HeexBlock(ref attrs) => NewNode::HeexBlock(ExHeexBlock {
            nodes: children,
            literal: attrs.literal.to_string(),
            node: match &attrs.node {
                HeexNode::Directive => "directive".to_string(),
                HeexNode::Comment => "comment".to_string(),
                HeexNode::MultilineComment => "multiline_comment".to_string(),
                HeexNode::Expression => "expression".to_string(),
                HeexNode::Tag(tag) => tag.clone(),
            },
            sourcepos,
        }),

        NodeValue::HeexInline(ref literal) => NewNode::HeexInline(ExHeexInline {
            literal: literal.to_string(),
            sourcepos,
        }),

        NodeValue::BlockDirective(ref attrs) => NewNode::BlockDirective(ExBlockDirective {
            nodes: children,
            info: attrs.info.clone(),
            fence_length: attrs.fence_length,
            fence_offset: attrs.fence_offset,
            sourcepos,
        }),
    }
}

fn string_to_char(s: String) -> u8 {
    s.bytes().next().unwrap_or(0)
}

fn char_to_string(c: u8) -> Result<String, &'static str> {
    if c == 0 {
        return Ok("".to_string());
    }

    match String::from_utf8(vec![c]) {
        Ok(s) => Ok(s),
        Err(_) => Err("failed to convert to string"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sourcepos() -> ExSourcepos {
        ExSourcepos::default()
    }

    #[test]
    fn escaped_tag_conversion_preserves_comrak_delimiter_tokens() {
        let arena = TypedArena::new();
        let node = ex_document_to_comrak_ast(
            &arena,
            NewNode::Document(ExDocument {
                nodes: vec![NewNode::Paragraph(ExParagraph {
                    nodes: vec![NewNode::EscapedTag(ExEscapedTag {
                        nodes: vec![NewNode::Text(ExText {
                            literal: "child".to_string(),
                            sourcepos: sourcepos(),
                        })],
                        literal: "|".to_string(),
                        sourcepos: sourcepos(),
                    })],
                    sourcepos: sourcepos(),
                })],
                sourcepos: sourcepos(),
            }),
        );
        let paragraph = node.first_child().expect("document should have a child");
        let child = paragraph
            .first_child()
            .expect("paragraph should have a child");

        assert!(matches!(child.data().value, NodeValue::EscapedTag("|")));
        assert!(matches!(
            child
                .first_child()
                .expect("escaped tag should keep children")
                .data()
                .value,
            NodeValue::Text(ref literal) if literal == "child"
        ));
    }

    #[test]
    fn escaped_tag_conversion_renders_arbitrary_literals_as_leaf_text() {
        let arena = TypedArena::new();
        let node = ex_document_to_comrak_ast(
            &arena,
            NewNode::Document(ExDocument {
                nodes: vec![NewNode::Paragraph(ExParagraph {
                    nodes: vec![NewNode::EscapedTag(ExEscapedTag {
                        nodes: vec![NewNode::Text(ExText {
                            literal: "child".to_string(),
                            sourcepos: sourcepos(),
                        })],
                        literal: "<script>alert(1)</script>".to_string(),
                        sourcepos: sourcepos(),
                    })],
                    sourcepos: sourcepos(),
                })],
                sourcepos: sourcepos(),
            }),
        );
        let paragraph = node.first_child().expect("document should have a child");
        let child = paragraph
            .first_child()
            .expect("paragraph should have a child");

        assert!(matches!(
            child.data().value,
            NodeValue::Text(ref literal) if literal == "<script>alert(1)</script>"
        ));
        assert!(
            child.first_child().is_none(),
            "fallback text nodes cannot retain escaped tag children"
        );
    }
}
