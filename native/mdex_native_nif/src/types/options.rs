mod sanitize;

use super::elixir_types::ExFormatterOption;
use comrak::options::{Extension, ListStyleType, Options, Parse, Render};
use rustler::types::atom::Atom;
use rustler::{Decoder, NifResult, Term};
pub use sanitize::*;
use std::sync::Arc;

fn optional_field<'a, T>(term: Term<'a>, key: &str) -> NifResult<Option<T>>
where
    T: Decoder<'a>,
{
    match term.map_get(Atom::from_str(term.get_env(), key)?) {
        Ok(value) => value.decode(),
        Err(_) => Ok(None),
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExExtensionOptions {
    pub strikethrough: Option<bool>,
    pub tagfilter: Option<bool>,
    pub table: Option<bool>,
    pub autolink: Option<bool>,
    pub tasklist: Option<bool>,
    pub superscript: Option<bool>,
    pub header_id_prefix: Option<String>,
    pub header_id_prefix_in_href: Option<bool>,
    pub footnotes: Option<bool>,
    pub inline_footnotes: Option<bool>,
    pub description_lists: Option<bool>,
    pub front_matter_delimiter: Option<String>,
    pub multiline_block_quotes: Option<bool>,
    pub alerts: Option<bool>,
    pub math_dollars: Option<bool>,
    pub math_code: Option<bool>,
    pub shortcodes: Option<bool>,
    pub wikilinks_title_after_pipe: Option<bool>,
    pub wikilinks_title_before_pipe: Option<bool>,
    pub underline: Option<bool>,
    pub subscript: Option<bool>,
    pub spoiler: Option<bool>,
    pub greentext: Option<bool>,
    pub subtext: Option<bool>,
    pub highlight: Option<bool>,
    pub insert: Option<bool>,
    pub image_url_rewriter: Option<String>,
    pub link_url_rewriter: Option<String>,
    pub cjk_friendly_emphasis: Option<bool>,
    pub phoenix_heex: Option<bool>,
    pub block_directive: Option<bool>,
}

impl<'a> Decoder<'a> for ExExtensionOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            strikethrough: optional_field(term, "strikethrough")?,
            tagfilter: optional_field(term, "tagfilter")?,
            table: optional_field(term, "table")?,
            autolink: optional_field(term, "autolink")?,
            tasklist: optional_field(term, "tasklist")?,
            superscript: optional_field(term, "superscript")?,
            header_id_prefix: optional_field(term, "header_id_prefix")?,
            header_id_prefix_in_href: optional_field(term, "header_id_prefix_in_href")?,
            footnotes: optional_field(term, "footnotes")?,
            inline_footnotes: optional_field(term, "inline_footnotes")?,
            description_lists: optional_field(term, "description_lists")?,
            front_matter_delimiter: optional_field(term, "front_matter_delimiter")?,
            multiline_block_quotes: optional_field(term, "multiline_block_quotes")?,
            alerts: optional_field(term, "alerts")?,
            math_dollars: optional_field(term, "math_dollars")?,
            math_code: optional_field(term, "math_code")?,
            shortcodes: optional_field(term, "shortcodes")?,
            wikilinks_title_after_pipe: optional_field(term, "wikilinks_title_after_pipe")?,
            wikilinks_title_before_pipe: optional_field(term, "wikilinks_title_before_pipe")?,
            underline: optional_field(term, "underline")?,
            subscript: optional_field(term, "subscript")?,
            spoiler: optional_field(term, "spoiler")?,
            greentext: optional_field(term, "greentext")?,
            subtext: optional_field(term, "subtext")?,
            highlight: optional_field(term, "highlight")?,
            insert: optional_field(term, "insert")?,
            image_url_rewriter: optional_field(term, "image_url_rewriter")?,
            link_url_rewriter: optional_field(term, "link_url_rewriter")?,
            cjk_friendly_emphasis: optional_field(term, "cjk_friendly_emphasis")?,
            phoenix_heex: optional_field(term, "phoenix_heex")?,
            block_directive: optional_field(term, "block_directive")?,
        })
    }
}

#[allow(deprecated)]
impl ExExtensionOptions {
    pub fn apply(self, extension: &mut Extension<'static>) {
        if let Some(value) = self.strikethrough {
            extension.strikethrough = value;
        }
        if let Some(value) = self.tagfilter {
            extension.tagfilter = value;
        }
        if let Some(value) = self.table {
            extension.table = value;
        }
        if let Some(value) = self.autolink {
            extension.autolink = value;
        }
        if let Some(value) = self.tasklist {
            extension.tasklist = value;
        }
        if let Some(value) = self.superscript {
            extension.superscript = value;
        }
        if self.header_id_prefix.is_some() {
            extension.header_id_prefix = self.header_id_prefix;
        }
        if let Some(value) = self.header_id_prefix_in_href {
            extension.header_id_prefix_in_href = value;
        }
        if let Some(value) = self.footnotes {
            extension.footnotes = value;
        }
        if let Some(value) = self.inline_footnotes {
            extension.inline_footnotes = value;
        }
        if let Some(value) = self.description_lists {
            extension.description_lists = value;
        }
        if self.front_matter_delimiter.is_some() {
            extension.front_matter_delimiter = self.front_matter_delimiter;
        }
        if let Some(value) = self.multiline_block_quotes {
            extension.multiline_block_quotes = value;
        }
        if let Some(value) = self.alerts {
            extension.alerts = value;
        }
        if let Some(value) = self.math_dollars {
            extension.math_dollars = value;
        }
        if let Some(value) = self.math_code {
            extension.math_code = value;
        }
        if let Some(value) = self.shortcodes {
            extension.shortcodes = value;
        }
        if let Some(value) = self.wikilinks_title_after_pipe {
            extension.wikilinks_title_after_pipe = value;
        }
        if let Some(value) = self.wikilinks_title_before_pipe {
            extension.wikilinks_title_before_pipe = value;
        }
        if let Some(value) = self.underline {
            extension.underline = value;
        }
        if let Some(value) = self.subscript {
            extension.subscript = value;
        }
        if let Some(value) = self.spoiler {
            extension.spoiler = value;
        }
        if let Some(value) = self.greentext {
            extension.greentext = value;
        }
        if let Some(value) = self.subtext {
            extension.subtext = value;
        }
        if let Some(value) = self.highlight {
            extension.highlight = value;
        }
        if let Some(value) = self.insert {
            extension.insert = value;
        }
        if let Some(rewrite) = self.image_url_rewriter {
            extension.image_url_rewriter =
                Some(Arc::new(move |url: &str| rewrite.replace("{@url}", url)));
        }
        if let Some(rewrite) = self.link_url_rewriter {
            extension.link_url_rewriter =
                Some(Arc::new(move |url: &str| rewrite.replace("{@url}", url)));
        }
        if let Some(value) = self.cjk_friendly_emphasis {
            extension.cjk_friendly_emphasis = value;
        }
        if let Some(value) = self.phoenix_heex {
            extension.phoenix_heex = value;
        }
        if let Some(value) = self.block_directive {
            extension.block_directive = value;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExParseOptions {
    pub smart: Option<bool>,
    pub default_info_string: Option<String>,
    pub relaxed_tasklist_matching: Option<bool>,
    pub relaxed_autolinks: Option<bool>,
    pub ignore_setext: Option<bool>,
    pub tasklist_in_table: Option<bool>,
    pub leave_footnote_definitions: Option<bool>,
    pub escaped_char_spans: Option<bool>,
    pub sourcepos_chars: Option<bool>,
}

impl<'a> Decoder<'a> for ExParseOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            smart: optional_field(term, "smart")?,
            default_info_string: optional_field(term, "default_info_string")?,
            relaxed_tasklist_matching: optional_field(term, "relaxed_tasklist_matching")?,
            relaxed_autolinks: optional_field(term, "relaxed_autolinks")?,
            ignore_setext: optional_field(term, "ignore_setext")?,
            tasklist_in_table: optional_field(term, "tasklist_in_table")?,
            leave_footnote_definitions: optional_field(term, "leave_footnote_definitions")?,
            escaped_char_spans: optional_field(term, "escaped_char_spans")?,
            sourcepos_chars: optional_field(term, "sourcepos_chars")?,
        })
    }
}

impl ExParseOptions {
    pub fn apply(self, parse: &mut Parse<'static>) {
        if let Some(value) = self.smart {
            parse.smart = value;
        }
        if self.default_info_string.is_some() {
            parse.default_info_string = self.default_info_string;
        }
        if let Some(value) = self.relaxed_tasklist_matching {
            parse.relaxed_tasklist_matching = value;
        }
        if let Some(value) = self.relaxed_autolinks {
            parse.relaxed_autolinks = value;
        }
        if let Some(value) = self.ignore_setext {
            parse.ignore_setext = value;
        }
        if let Some(value) = self.tasklist_in_table {
            parse.tasklist_in_table = value;
        }
        if let Some(value) = self.leave_footnote_definitions {
            parse.leave_footnote_definitions = value;
        }
        if let Some(value) = self.escaped_char_spans {
            parse.escaped_char_spans = value;
        }
        if let Some(value) = self.sourcepos_chars {
            parse.sourcepos_chars = value;
        }
    }
}

#[derive(Clone, Debug, Default, NifUnitEnum)]
pub enum ExListStyleType {
    #[default]
    Dash,
    Plus,
    Star,
}

impl From<ExListStyleType> for ListStyleType {
    fn from(list_style_type: ExListStyleType) -> Self {
        match list_style_type {
            ExListStyleType::Dash => ListStyleType::Dash,
            ExListStyleType::Plus => ListStyleType::Plus,
            ExListStyleType::Star => ListStyleType::Star,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExRenderOptions {
    pub hardbreaks: Option<bool>,
    pub github_pre_lang: Option<bool>,
    pub full_info_string: Option<bool>,
    pub width: Option<usize>,
    pub r#unsafe: Option<bool>,
    pub escape: Option<bool>,
    pub list_style: Option<ExListStyleType>,
    pub sourcepos: Option<bool>,
    pub escaped_char_spans: Option<bool>,
    pub ignore_empty_links: Option<bool>,
    pub gfm_quirks: Option<bool>,
    pub prefer_fenced: Option<bool>,
    pub figure_with_caption: Option<bool>,
    pub tasklist_classes: Option<bool>,
    pub ol_width: Option<usize>,
    pub experimental_minimize_commonmark: Option<bool>,
    pub compact_html: Option<bool>,
}

impl<'a> Decoder<'a> for ExRenderOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            hardbreaks: optional_field(term, "hardbreaks")?,
            github_pre_lang: optional_field(term, "github_pre_lang")?,
            full_info_string: optional_field(term, "full_info_string")?,
            width: optional_field(term, "width")?,
            r#unsafe: optional_field(term, "unsafe")?,
            escape: optional_field(term, "escape")?,
            list_style: optional_field(term, "list_style")?,
            sourcepos: optional_field(term, "sourcepos")?,
            escaped_char_spans: optional_field(term, "escaped_char_spans")?,
            ignore_empty_links: optional_field(term, "ignore_empty_links")?,
            gfm_quirks: optional_field(term, "gfm_quirks")?,
            prefer_fenced: optional_field(term, "prefer_fenced")?,
            figure_with_caption: optional_field(term, "figure_with_caption")?,
            tasklist_classes: optional_field(term, "tasklist_classes")?,
            ol_width: optional_field(term, "ol_width")?,
            experimental_minimize_commonmark: optional_field(
                term,
                "experimental_minimize_commonmark",
            )?,
            compact_html: optional_field(term, "compact_html")?,
        })
    }
}

impl ExRenderOptions {
    pub fn apply(self, render: &mut Render) {
        if let Some(value) = self.hardbreaks {
            render.hardbreaks = value;
        }
        if let Some(value) = self.github_pre_lang {
            render.github_pre_lang = value;
        }
        if let Some(value) = self.full_info_string {
            render.full_info_string = value;
        }
        if let Some(value) = self.width {
            render.width = value;
        }
        if let Some(value) = self.r#unsafe {
            render.r#unsafe = value;
        }
        if let Some(value) = self.escape {
            render.escape = value;
        }
        if let Some(value) = self.list_style {
            render.list_style = ListStyleType::from(value);
        }
        if let Some(value) = self.sourcepos {
            render.sourcepos = value;
        }
        if let Some(value) = self.escaped_char_spans {
            render.escaped_char_spans = value;
        }
        if let Some(value) = self.ignore_empty_links {
            render.ignore_empty_links = value;
        }
        if let Some(value) = self.gfm_quirks {
            render.gfm_quirks = value;
        }
        if let Some(value) = self.prefer_fenced {
            render.prefer_fenced = value;
        }
        if let Some(value) = self.figure_with_caption {
            render.figure_with_caption = value;
        }
        if let Some(value) = self.tasklist_classes {
            render.tasklist_classes = value;
        }
        if let Some(value) = self.ol_width {
            render.ol_width = value;
        }
        if let Some(value) = self.experimental_minimize_commonmark {
            render.experimental_minimize_commonmark = value;
        }
        if let Some(value) = self.compact_html {
            render.compact_html = value;
        }
    }
}

#[derive(Debug, Default)]
pub struct ExOptions {
    pub extension: Option<ExExtensionOptions>,
    pub parse: Option<ExParseOptions>,
    pub render: Option<ExRenderOptions>,
    pub syntax_highlight: Option<ExSyntaxHighlightOptions>,
    pub sanitize: Option<ExSanitizeOption>,
}

impl<'a> Decoder<'a> for ExOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            extension: optional_field(term, "extension")?,
            parse: optional_field(term, "parse")?,
            render: optional_field(term, "render")?,
            syntax_highlight: optional_field(term, "syntax_highlight")?,
            sanitize: optional_field(term, "sanitize")?,
        })
    }
}

impl ExOptions {
    pub fn comrak_options(&self) -> Options<'static> {
        let mut options = Options::default();
        if let Some(extension) = self.extension.clone() {
            extension.apply(&mut options.extension);
        }
        if let Some(parse) = self.parse.clone() {
            parse.apply(&mut options.parse);
        }
        if let Some(render) = self.render.clone() {
            render.apply(&mut options.render);
        }
        options
    }
}

#[derive(Debug, Default, NifTaggedEnum)]
pub enum ExSanitizeOption {
    #[default]
    Clean,
    Custom(Box<ExSanitizeCustom>),
}

impl ExSanitizeOption {
    pub(crate) fn clean(&self, html: &str) -> String {
        match self {
            ExSanitizeOption::Clean => ammonia::clean(html),
            ExSanitizeOption::Custom(custom) => custom.to_ammonia().clean(html).to_string(),
        }
    }
}

#[derive(Debug, Default, NifMap)]
pub struct ExSyntaxHighlightOptions {
    pub formatter: ExFormatterOption,
    pub language: Option<String>,
}
