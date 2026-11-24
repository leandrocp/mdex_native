use comrak::nodes::{
    AstNode, Attributes, GridTableColumn, GridTableSection, HeexNode, LineColumn, ListDelimType,
    ListType, NodeAlert, NodeBlockDirective, NodeBlockId, NodeCallout, NodeCode, NodeCodeBlock,
    NodeComment, NodeCommentBlock, NodeDescriptionItem, NodeEmph, NodeFootnoteDefinition,
    NodeFootnoteReference, NodeGridTable, NodeGridTableCell, NodeHeading, NodeHeexBlock,
    NodeHtmlBlock, NodeLink, NodeList, NodeMath, NodeMathBlock, NodeMultilineBlockQuote,
    NodeShortCode, NodeStrikethrough, NodeStrong, NodeTable, NodeTableRow, NodeTaskItem,
    NodeThematicBreak, NodeValue, NodeWikiLink, Sourcepos, TableAlignment,
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

/// Keys that are not valid HTML attribute names are dropped: comrak writes keys
/// as they are, in HTML and in CommonMark, so a key like `x" onclick="alert(1)`
/// set from Elixir would otherwise escape the tag.
///
/// TODO: upstream `write_opening_tag` and the CommonMark attribute writer
/// should skip or reject keys their parser can't produce, so this filter can
/// go. comrak only checks keys while parsing:
///
/// - [`write_opening_tag`](https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/html.rs#L2145-L2158)
/// - [`format_attributes`](https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/cm.rs#L1376-L1396)
/// - [parser key rule](https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/parser/attributes.rs#L161-L175)
impl From<ExAttributes> for Attributes {
    fn from(attributes: ExAttributes) -> Self {
        let mut attrs = Self::default();
        attrs.id = attributes.id;
        attrs.classes = attributes.classes;
        attrs.pairs = attributes
            .pairs
            .into_iter()
            .filter(|(key, _)| is_attribute_name(key))
            .collect();
        attrs
    }
}

/// Whether `name` is an attribute key, as comrak's parser reads one: an ASCII
/// letter, `_` or `:`, then ASCII letters, digits, `-`, `_`, `:` and `.`.
fn is_attribute_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_alphabetic() || matches!(c, '_' | ':'))
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.'))
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
    GridTable(ExGridTable),
    GridTableRow(ExGridTableRow),
    GridTableCell(ExGridTableCell),
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
    MathBlock(ExMathBlock),
    MultilineBlockQuote(ExMultilineBlockQuote),
    Escaped(ExEscaped),
    WikiLink(ExWikiLink),
    Underline(ExUnderline),
    Subscript(ExSubscript),
    SpoileredText(ExSpoileredText),
    Subtext(ExSubtext),
    EscapedTag(ExEscapedTag),
    Alert(ExAlert),
    Callout(ExCallout),
    CalloutTitle(ExCalloutTitle),
    BlockId(ExBlockId),
    CommentInline(ExCommentInline),
    MultilineComment(ExMultilineComment),
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
            | Self::GridTable(ExGridTable { sourcepos, .. })
            | Self::GridTableRow(ExGridTableRow { sourcepos, .. })
            | Self::GridTableCell(ExGridTableCell { sourcepos, .. })
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
            | Self::MathBlock(ExMathBlock { sourcepos, .. })
            | Self::MultilineBlockQuote(ExMultilineBlockQuote { sourcepos, .. })
            | Self::Escaped(ExEscaped { sourcepos, .. })
            | Self::WikiLink(ExWikiLink { sourcepos, .. })
            | Self::Underline(ExUnderline { sourcepos, .. })
            | Self::Subscript(ExSubscript { sourcepos, .. })
            | Self::SpoileredText(ExSpoileredText { sourcepos, .. })
            | Self::Subtext(ExSubtext { sourcepos, .. })
            | Self::EscapedTag(ExEscapedTag { sourcepos, .. })
            | Self::Alert(ExAlert { sourcepos, .. })
            | Self::Callout(ExCallout { sourcepos, .. })
            | Self::CalloutTitle(ExCalloutTitle { sourcepos, .. })
            | Self::BlockId(ExBlockId { sourcepos, .. })
            | Self::CommentInline(ExCommentInline { sourcepos, .. })
            | Self::MultilineComment(ExMultilineComment { sourcepos, .. })
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
            | Self::GridTable(ExGridTable { nodes, .. })
            | Self::GridTableRow(ExGridTableRow { nodes, .. })
            | Self::GridTableCell(ExGridTableCell { nodes, .. })
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
            | Self::Escaped(ExEscaped { nodes, .. })
            | Self::WikiLink(ExWikiLink { nodes, .. })
            | Self::Underline(ExUnderline { nodes, .. })
            | Self::Subscript(ExSubscript { nodes, .. })
            | Self::SpoileredText(ExSpoileredText { nodes, .. })
            | Self::Subtext(ExSubtext { nodes, .. })
            | Self::Alert(ExAlert { nodes, .. })
            | Self::Callout(ExCallout { nodes, .. })
            | Self::CalloutTitle(ExCalloutTitle { nodes, .. })
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
            NewNode::GridTable(n) => n.into(),
            NewNode::GridTableRow(n) => n.into(),
            NewNode::GridTableCell(n) => n.into(),
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
            NewNode::MathBlock(n) => n.into(),
            NewNode::MultilineBlockQuote(n) => n.into(),
            NewNode::Escaped(n) => n.into(),
            NewNode::WikiLink(n) => n.into(),
            NewNode::Underline(n) => n.into(),
            NewNode::Subscript(n) => n.into(),
            NewNode::SpoileredText(n) => n.into(),
            NewNode::Subtext(n) => n.into(),
            NewNode::EscapedTag(n) => n.into(),
            NewNode::Alert(n) => n.into(),
            NewNode::Callout(n) => n.into(),
            NewNode::CalloutTitle(n) => n.into(),
            NewNode::BlockId(n) => n.into(),
            NewNode::CommentInline(n) => n.into(),
            NewNode::MultilineComment(n) => n.into(),
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
        NodeValue::List(node_list(
            &node.list_type,
            node.marker_offset,
            node.padding,
            node.start,
            &node.delimiter,
            node.bullet_char,
            node.tight,
            node.is_task_list,
        ))
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
        NodeValue::Item(node_list(
            &node.list_type,
            node.marker_offset,
            node.padding,
            node.start,
            &node.delimiter,
            node.bullet_char,
            node.tight,
            node.is_task_list,
        ))
    }
}
// `NodeList` is non-exhaustive, so it is built from its default.
#[allow(clippy::too_many_arguments)]
fn node_list(
    list_type: &ExListType,
    marker_offset: usize,
    padding: usize,
    start: usize,
    delimiter: &ExListDelimType,
    bullet_char: String,
    tight: bool,
    is_task_list: bool,
) -> NodeList {
    let mut list = NodeList::default();
    list.list_type = match list_type {
        ExListType::Bullet => ListType::Bullet,
        ExListType::Ordered => ListType::Ordered,
    };
    list.marker_offset = marker_offset;
    list.padding = padding;
    list.start = start;
    list.delimiter = match delimiter {
        ExListDelimType::Period => ListDelimType::Period,
        ExListDelimType::Paren => ListDelimType::Paren,
    };
    list.bullet_char = string_to_char(bullet_char);
    list.tight = tight;
    list.is_task_list = is_task_list;
    list
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
        let mut item = NodeDescriptionItem::new(node.tight);
        item.marker_offset = node.marker_offset;
        item.padding = node.padding;
        NodeValue::DescriptionItem(item)
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
        let mut code_block = NodeCodeBlock::default();
        code_block.fenced = node.fenced;
        code_block.fence_char = string_to_char(node.fence_char);
        code_block.fence_length = node.fence_length;
        code_block.fence_offset = node.fence_offset;
        code_block.info = node.info;
        code_block.literal = node.literal;
        code_block.closed = node.closed;
        NodeValue::CodeBlock(Box::new(code_block))
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
        let mut html_block = NodeHtmlBlock::new(node.literal);
        html_block.block_type = node.block_type;
        NodeValue::HtmlBlock(html_block)
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
        let mut heading = NodeHeading::new(node.level);
        heading.setext = node.setext;
        heading.closed = node.closed;
        NodeValue::Heading(heading)
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.ThematicBreak"]
pub struct ExThematicBreak {
    pub sourcepos: ExSourcepos,
}

impl From<ExThematicBreak> for NodeValue {
    fn from(_node: ExThematicBreak) -> Self {
        NodeValue::ThematicBreak(NodeThematicBreak::default())
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
        let mut definition = NodeFootnoteDefinition::new(node.name);
        definition.total_references = node.total_references;
        NodeValue::FootnoteDefinition(definition)
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
        let mut reference = NodeFootnoteReference::new(node.name);
        reference.ref_num = node.ref_num;
        reference.ix = node.ix;
        reference.texts = node.texts;
        NodeValue::FootnoteReference(Box::new(reference))
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
        let mut table = NodeTable::new(node.alignments.into_iter().map(Into::into).collect());
        table.num_columns = node.num_columns;
        table.num_rows = node.num_rows;
        table.num_nonempty_cells = node.num_nonempty_cells;
        NodeValue::Table(Box::new(table))
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
        NodeValue::TableRow(NodeTableRow::new(node.header))
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

impl From<ExTableAlignment> for TableAlignment {
    fn from(alignment: ExTableAlignment) -> Self {
        match alignment {
            ExTableAlignment::None => TableAlignment::None,
            ExTableAlignment::Left => TableAlignment::Left,
            ExTableAlignment::Center => TableAlignment::Center,
            ExTableAlignment::Right => TableAlignment::Right,
        }
    }
}

impl From<&TableAlignment> for ExTableAlignment {
    fn from(alignment: &TableAlignment) -> Self {
        match alignment {
            TableAlignment::None => ExTableAlignment::None,
            TableAlignment::Left => ExTableAlignment::Left,
            TableAlignment::Center => ExTableAlignment::Center,
            TableAlignment::Right => ExTableAlignment::Right,
        }
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.GridTableColumn"]
pub struct ExGridTableColumn {
    pub alignment: ExTableAlignment,
    pub width: usize,
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.GridTable"]
pub struct ExGridTable {
    pub nodes: Vec<NewNode>,
    pub columns: Vec<ExGridTableColumn>,
    pub sourcepos: ExSourcepos,
}

impl From<ExGridTable> for NodeValue {
    fn from(node: ExGridTable) -> Self {
        NodeValue::GridTable(Box::new(NodeGridTable::new(
            node.columns
                .into_iter()
                .map(|column| GridTableColumn::new(column.alignment.into(), column.width))
                .collect(),
        )))
    }
}

#[derive(Clone, Debug, NifUnitEnum, PartialEq)]
pub enum ExGridTableSection {
    Head,
    Body,
    Foot,
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.GridTableRow"]
pub struct ExGridTableRow {
    pub nodes: Vec<NewNode>,
    pub section: ExGridTableSection,
    pub sourcepos: ExSourcepos,
}

impl From<ExGridTableRow> for NodeValue {
    fn from(node: ExGridTableRow) -> Self {
        NodeValue::GridTableRow(match node.section {
            ExGridTableSection::Head => GridTableSection::Head,
            ExGridTableSection::Body => GridTableSection::Body,
            ExGridTableSection::Foot => GridTableSection::Foot,
        })
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.GridTableCell"]
pub struct ExGridTableCell {
    pub nodes: Vec<NewNode>,
    pub column: usize,
    pub colspan: usize,
    pub rowspan: usize,
    pub sourcepos: ExSourcepos,
}

impl From<ExGridTableCell> for NodeValue {
    fn from(node: ExGridTableCell) -> Self {
        NodeValue::GridTableCell(NodeGridTableCell::new(
            node.column,
            node.colspan,
            node.rowspan,
        ))
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
        NodeValue::TaskItem(NodeTaskItem::new(node.marker.chars().next()))
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
        let mut code = NodeCode::new(node.literal);
        code.num_backticks = node.num_backticks;
        NodeValue::Code(code)
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
        NodeValue::HeexBlock(Box::new(NodeHeexBlock::new(node.literal, heex_node)))
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
        NodeValue::Emph(NodeEmph::default())
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
        NodeValue::Strong(NodeStrong::default())
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
        NodeValue::Strikethrough(NodeStrikethrough::default())
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
        NodeValue::Link(Box::new(NodeLink::new(node.url, node.title)))
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
        NodeValue::Image(Box::new(NodeLink::new(node.url, node.title)))
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
        NodeValue::ShortCode(Box::new(NodeShortCode::new(node.code, node.emoji)))
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
        let mut math = NodeMath::new(node.literal, node.display_math);
        math.dollar_math = node.dollar_math;
        NodeValue::Math(math)
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.MathBlock"]
pub struct ExMathBlock {
    pub fence_offset: usize,
    pub literal: String,
    pub closed: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExMathBlock> for NodeValue {
    fn from(node: ExMathBlock) -> Self {
        let mut math = NodeMathBlock::new(node.literal);
        math.fence_offset = node.fence_offset;
        math.closed = node.closed;
        NodeValue::MathBlock(Box::new(math))
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
        let mut block_quote = NodeMultilineBlockQuote::default();
        block_quote.fence_length = node.fence_length;
        block_quote.fence_offset = node.fence_offset;
        NodeValue::MultilineBlockQuote(block_quote)
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Escaped"]
pub struct ExEscaped {
    pub nodes: Vec<NewNode>,
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
    pub embed: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExWikiLink> for NodeValue {
    fn from(node: ExWikiLink) -> Self {
        NodeValue::WikiLink(NodeWikiLink::new(node.url, node.embed))
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
        let alert_type = match node.alert_type {
            ExAlertType::Note => comrak::nodes::AlertType::Note,
            ExAlertType::Tip => comrak::nodes::AlertType::Tip,
            ExAlertType::Important => comrak::nodes::AlertType::Important,
            ExAlertType::Warning => comrak::nodes::AlertType::Warning,
            ExAlertType::Caution => comrak::nodes::AlertType::Caution,
        };
        let mut alert = NodeAlert::new(alert_type, node.title);
        alert.multiline = node.multiline;
        alert.fence_length = node.fence_length;
        alert.fence_offset = node.fence_offset;
        NodeValue::Alert(Box::new(alert))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.CommentInline"]
pub struct ExCommentInline {
    pub literal: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExCommentInline> for NodeValue {
    fn from(node: ExCommentInline) -> Self {
        NodeValue::CommentInline(NodeComment::new(node.literal))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.MultilineComment"]
pub struct ExMultilineComment {
    pub literal: String,
    pub closed: bool,
    pub sourcepos: ExSourcepos,
}

impl From<ExMultilineComment> for NodeValue {
    fn from(node: ExMultilineComment) -> Self {
        let mut comment = NodeCommentBlock::new(node.literal);
        comment.closed = node.closed;
        NodeValue::MultilineComment(comment)
    }
}

#[derive(Clone, Debug, NifUnitEnum, PartialEq)]
pub enum ExCalloutFold {
    None,
    Expanded,
    Collapsed,
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.Callout"]
pub struct ExCallout {
    pub nodes: Vec<NewNode>,
    pub callout_type: String,
    pub metadata: Option<String>,
    pub fold: ExCalloutFold,
    pub sourcepos: ExSourcepos,
}

impl From<ExCallout> for NodeValue {
    fn from(node: ExCallout) -> Self {
        let fold = match node.fold {
            ExCalloutFold::None => comrak::nodes::CalloutFold::None,
            ExCalloutFold::Expanded => comrak::nodes::CalloutFold::Expanded,
            ExCalloutFold::Collapsed => comrak::nodes::CalloutFold::Collapsed,
        };
        NodeValue::Callout(Box::new(NodeCallout::new(
            node.callout_type,
            node.metadata,
            fold,
        )))
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.CalloutTitle"]
pub struct ExCalloutTitle {
    pub nodes: Vec<NewNode>,
    pub sourcepos: ExSourcepos,
}

impl From<ExCalloutTitle> for NodeValue {
    fn from(_node: ExCalloutTitle) -> Self {
        NodeValue::CalloutTitle
    }
}

#[derive(Clone, Debug, NifStruct, PartialEq)]
#[module = "MDExNative.Comrak.BlockId"]
pub struct ExBlockId {
    pub name: String,
    pub sourcepos: ExSourcepos,
}

impl From<ExBlockId> for NodeValue {
    fn from(node: ExBlockId) -> Self {
        NodeValue::BlockId(Box::new(NodeBlockId::new(node.name)))
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
        let mut directive = NodeBlockDirective::new(node.info);
        directive.fence_length = node.fence_length;
        directive.fence_offset = node.fence_offset;
        NodeValue::BlockDirective(Box::new(directive))
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

        NodeValue::ThematicBreak(_) => NewNode::ThematicBreak(ExThematicBreak { sourcepos }),

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
            alignments: attrs.alignments.iter().map(Into::into).collect(),
            num_columns: attrs.num_columns,
            num_rows: attrs.num_rows,
            num_nonempty_cells: attrs.num_nonempty_cells,
            sourcepos,
        }),

        NodeValue::TableRow(row) => NewNode::TableRow(ExTableRow {
            nodes: children,
            header: row.header,
            sourcepos,
        }),

        NodeValue::TableCell => NewNode::TableCell(ExTableCell {
            nodes: children,
            sourcepos,
        }),

        NodeValue::GridTable(table) => NewNode::GridTable(ExGridTable {
            nodes: children,
            columns: table
                .columns
                .iter()
                .map(|column| ExGridTableColumn {
                    alignment: (&column.alignment).into(),
                    width: column.width,
                })
                .collect(),
            sourcepos,
        }),

        NodeValue::GridTableRow(section) => NewNode::GridTableRow(ExGridTableRow {
            nodes: children,
            section: match section {
                GridTableSection::Head => ExGridTableSection::Head,
                GridTableSection::Body => ExGridTableSection::Body,
                GridTableSection::Foot => ExGridTableSection::Foot,
            },
            sourcepos,
        }),

        NodeValue::GridTableCell(cell) => NewNode::GridTableCell(ExGridTableCell {
            nodes: children,
            column: cell.column,
            colspan: cell.colspan,
            rowspan: cell.rowspan,
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

        NodeValue::Emph(_) => NewNode::Emph(ExEmph {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Strong(_) => NewNode::Strong(ExStrong {
            nodes: children,
            sourcepos,
        }),

        NodeValue::Strikethrough(_) => NewNode::Strikethrough(ExStrikethrough {
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

        NodeValue::MathBlock(ref attrs) => NewNode::MathBlock(ExMathBlock {
            fence_offset: attrs.fence_offset,
            literal: attrs.literal.clone(),
            closed: attrs.closed,
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

        NodeValue::Escaped => NewNode::Escaped(ExEscaped {
            nodes: children,
            sourcepos,
        }),

        NodeValue::WikiLink(ref attrs) => NewNode::WikiLink(ExWikiLink {
            nodes: children,
            url: attrs.url.to_string(),
            embed: attrs.embed,
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
                other => unreachable!("comrak 1.x has no alert type {other:?}"),
            },
            title: attrs.title.to_owned(),
            multiline: attrs.multiline,
            fence_length: attrs.fence_length,
            fence_offset: attrs.fence_offset,
            sourcepos,
        }),

        NodeValue::Callout(ref attrs) => NewNode::Callout(ExCallout {
            nodes: children,
            callout_type: attrs.callout_type.clone(),
            metadata: attrs.metadata.clone(),
            fold: match attrs.fold {
                comrak::nodes::CalloutFold::None => ExCalloutFold::None,
                comrak::nodes::CalloutFold::Expanded => ExCalloutFold::Expanded,
                comrak::nodes::CalloutFold::Collapsed => ExCalloutFold::Collapsed,
            },
            sourcepos,
        }),

        NodeValue::CalloutTitle => NewNode::CalloutTitle(ExCalloutTitle {
            nodes: children,
            sourcepos,
        }),

        NodeValue::BlockId(ref attrs) => NewNode::BlockId(ExBlockId {
            name: attrs.name.clone(),
            sourcepos,
        }),

        NodeValue::CommentInline(ref attrs) => NewNode::CommentInline(ExCommentInline {
            literal: attrs.literal.clone(),
            sourcepos,
        }),

        NodeValue::MultilineComment(ref attrs) => NewNode::MultilineComment(ExMultilineComment {
            literal: attrs.literal.clone(),
            closed: attrs.closed,
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
                other => unreachable!("comrak 1.x has no HEEx node {other:?}"),
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

        // Comrak adds node types only with new options, and none reach here
        // that this module does not decode.
        other => unreachable!("comrak produced a node MDExNative does not know: {other:?}"),
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

    #[test]
    fn escaped_conversion_preserves_children() {
        let arena = TypedArena::new();
        let node = ex_document_to_comrak_ast(
            &arena,
            NewNode::Document(ExDocument {
                nodes: vec![NewNode::Paragraph(ExParagraph {
                    nodes: vec![NewNode::Escaped(ExEscaped {
                        nodes: vec![NewNode::Text(ExText {
                            literal: "*".to_string(),
                            sourcepos: sourcepos(),
                        })],
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

        assert!(matches!(child.data().value, NodeValue::Escaped));
        assert!(matches!(
            child
                .first_child()
                .expect("escaped should keep children")
                .data()
                .value,
            NodeValue::Text(ref literal) if literal == "*"
        ));
    }

    #[test]
    fn escaped_round_trip_keeps_children() {
        let arena = TypedArena::new();
        let node = ex_document_to_comrak_ast(
            &arena,
            NewNode::Escaped(ExEscaped {
                nodes: vec![NewNode::Text(ExText {
                    literal: "*".to_string(),
                    sourcepos: sourcepos(),
                })],
                sourcepos: sourcepos(),
            }),
        );
        let child = comrak_ast_to_ex_document_shallow(
            node.first_child().expect("escaped should have a child"),
        );

        assert_eq!(
            comrak_ast_to_ex_document_with_children(node, vec![child.clone()]),
            NewNode::Escaped(ExEscaped {
                nodes: vec![child],
                sourcepos: sourcepos(),
            })
        );
    }
}
