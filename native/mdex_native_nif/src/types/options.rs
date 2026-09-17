mod sanitize;

use comrak::options::{AlertStyleType, Extension, ListStyleType, Options, Parse, Render};
#[cfg(feature = "lumis")]
use lumis_core::elixir::ExFormatterOption;
use rustler::types::atom::{self, Atom};
use rustler::{Decoder, NifResult, NifUnitEnum, Term};
pub use sanitize::*;
use std::sync::Arc;

/// Every option key is read on every call, so they are interned once when the
/// NIF loads instead of being looked up from a string each time.
mod atoms {
    rustler::atoms! {
        alert_style,
        alerts,
        autolink,
        block_directive,
        cjk_friendly_emphasis,
        compact_html,
        default_info_string,
        description_lists,
        engine,
        escape,
        escaped_char_spans,
        experimental_minimize_commonmark,
        extension,
        fenced_code_attributes,
        figure_with_caption,
        footnotes,
        formatter,
        front_matter_delimiter,
        full_info_string,
        gfm_quirks,
        github_pre_lang,
        greentext,
        hardbreaks,
        header_attributes,
        header_id_prefix,
        header_id_prefix_in_href,
        highlight,
        ignore_empty_links,
        ignore_setext,
        image_url_rewriter,
        inline_code_attributes,
        inline_footnotes,
        insert,
        leave_footnote_definitions,
        link_attributes,
        link_url_rewriter,
        list_style,
        math_code,
        math_dollars,
        math_latex,
        multiline_block_quotes,
        ol_width,
        opts,
        parse,
        phoenix_heex,
        prefer_fenced,
        rainbow_brackets,
        relaxed_autolinks,
        relaxed_tasklist_matching,
        render,
        sanitize,
        shortcodes,
        smart,
        sourcepos,
        sourcepos_chars,
        spoiler,
        strikethrough,
        subscript,
        subtext,
        superscript,
        syntax_highlight,
        table,
        tagfilter,
        tasklist,
        tasklist_classes,
        tasklist_in_table,
        theme,
        underline,
        unsafe_ = "unsafe",
        width,
        wikilinks_title_after_pipe,
        wikilinks_title_before_pipe
    }
}

/// Overwrites `target` when Elixir sent a value for the field, and leaves it
/// at its Comrak default when it did not. Every `apply` below is a list of
/// these, so the rule lives in one place instead of once per field.
fn overwrite<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}

/// Same rule for a field that is itself optional: `nil` from Elixir means
/// "leave it alone", not "clear it".
fn overwrite_optional<T>(target: &mut Option<T>, value: Option<T>) {
    if value.is_some() {
        *target = value;
    }
}

fn is_atom(term: Term, expected: Atom) -> bool {
    Atom::decode(term).is_ok_and(|atom| atom == expected)
}

fn optional_field<'a, T>(term: Term<'a>, key: Atom) -> NifResult<Option<T>>
where
    T: Decoder<'a>,
{
    match term.map_get(key) {
        Ok(value) => {
            if is_atom(value, atom::nil()) {
                Ok(None)
            } else {
                value.decode()
            }
        }
        Err(_) => Ok(None),
    }
}

fn syntax_highlight_field(term: Term) -> NifResult<Option<ExSyntaxHighlightOptions>> {
    match term.map_get(atoms::syntax_highlight()) {
        Ok(value) => {
            if is_atom(value, atom::nil()) || is_atom(value, atom::false_()) {
                Ok(None)
            } else {
                value.decode().map(Some)
            }
        }
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
    pub math_latex: Option<bool>,
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
    pub header_attributes: Option<bool>,
    pub fenced_code_attributes: Option<bool>,
    pub inline_code_attributes: Option<bool>,
    pub link_attributes: Option<bool>,
}

impl<'a> Decoder<'a> for ExExtensionOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            strikethrough: optional_field(term, atoms::strikethrough())?,
            tagfilter: optional_field(term, atoms::tagfilter())?,
            table: optional_field(term, atoms::table())?,
            autolink: optional_field(term, atoms::autolink())?,
            tasklist: optional_field(term, atoms::tasklist())?,
            superscript: optional_field(term, atoms::superscript())?,
            header_id_prefix: optional_field(term, atoms::header_id_prefix())?,
            header_id_prefix_in_href: optional_field(term, atoms::header_id_prefix_in_href())?,
            footnotes: optional_field(term, atoms::footnotes())?,
            inline_footnotes: optional_field(term, atoms::inline_footnotes())?,
            description_lists: optional_field(term, atoms::description_lists())?,
            front_matter_delimiter: optional_field(term, atoms::front_matter_delimiter())?,
            multiline_block_quotes: optional_field(term, atoms::multiline_block_quotes())?,
            alerts: optional_field(term, atoms::alerts())?,
            math_dollars: optional_field(term, atoms::math_dollars())?,
            math_latex: optional_field(term, atoms::math_latex())?,
            math_code: optional_field(term, atoms::math_code())?,
            shortcodes: optional_field(term, atoms::shortcodes())?,
            wikilinks_title_after_pipe: optional_field(term, atoms::wikilinks_title_after_pipe())?,
            wikilinks_title_before_pipe: optional_field(
                term,
                atoms::wikilinks_title_before_pipe(),
            )?,
            underline: optional_field(term, atoms::underline())?,
            subscript: optional_field(term, atoms::subscript())?,
            spoiler: optional_field(term, atoms::spoiler())?,
            greentext: optional_field(term, atoms::greentext())?,
            subtext: optional_field(term, atoms::subtext())?,
            highlight: optional_field(term, atoms::highlight())?,
            insert: optional_field(term, atoms::insert())?,
            image_url_rewriter: optional_field(term, atoms::image_url_rewriter())?,
            link_url_rewriter: optional_field(term, atoms::link_url_rewriter())?,
            cjk_friendly_emphasis: optional_field(term, atoms::cjk_friendly_emphasis())?,
            phoenix_heex: optional_field(term, atoms::phoenix_heex())?,
            block_directive: optional_field(term, atoms::block_directive())?,
            header_attributes: optional_field(term, atoms::header_attributes())?,
            fenced_code_attributes: optional_field(term, atoms::fenced_code_attributes())?,
            inline_code_attributes: optional_field(term, atoms::inline_code_attributes())?,
            link_attributes: optional_field(term, atoms::link_attributes())?,
        })
    }
}

#[allow(deprecated)]
impl ExExtensionOptions {
    pub fn apply(mut self, extension: &mut Extension<'static>) {
        self.apply_common_options(extension);
        self.apply_additional_options(extension);
        self.apply_rewriters_and_attributes(extension);
    }

    fn apply_common_options(&mut self, extension: &mut Extension<'static>) {
        overwrite(&mut extension.strikethrough, self.strikethrough);
        overwrite(&mut extension.tagfilter, self.tagfilter);
        overwrite(&mut extension.table, self.table);
        overwrite(&mut extension.autolink, self.autolink);
        overwrite(&mut extension.tasklist, self.tasklist);
        overwrite(&mut extension.superscript, self.superscript);
        overwrite_optional(
            &mut extension.header_id_prefix,
            self.header_id_prefix.take(),
        );
        overwrite(
            &mut extension.header_id_prefix_in_href,
            self.header_id_prefix_in_href,
        );
        overwrite(&mut extension.footnotes, self.footnotes);
        overwrite(&mut extension.inline_footnotes, self.inline_footnotes);
    }

    fn apply_additional_options(&mut self, extension: &mut Extension<'static>) {
        overwrite(&mut extension.description_lists, self.description_lists);
        overwrite_optional(
            &mut extension.front_matter_delimiter,
            self.front_matter_delimiter.take(),
        );
        overwrite(
            &mut extension.multiline_block_quotes,
            self.multiline_block_quotes,
        );
        overwrite(&mut extension.alerts, self.alerts);
        overwrite(&mut extension.math_dollars, self.math_dollars);
        overwrite(&mut extension.math_latex, self.math_latex);
        overwrite(&mut extension.math_code, self.math_code);
        overwrite(&mut extension.shortcodes, self.shortcodes);
        overwrite(
            &mut extension.wikilinks_title_after_pipe,
            self.wikilinks_title_after_pipe,
        );
        overwrite(
            &mut extension.wikilinks_title_before_pipe,
            self.wikilinks_title_before_pipe,
        );
        overwrite(&mut extension.underline, self.underline);
        overwrite(&mut extension.subscript, self.subscript);
        overwrite(&mut extension.spoiler, self.spoiler);
        overwrite(&mut extension.greentext, self.greentext);
        overwrite(&mut extension.subtext, self.subtext);
        overwrite(&mut extension.highlight, self.highlight);
        overwrite(&mut extension.insert, self.insert);
    }

    fn apply_rewriters_and_attributes(&mut self, extension: &mut Extension<'static>) {
        if let Some(rewrite) = self.image_url_rewriter.take() {
            extension.image_url_rewriter =
                Some(Arc::new(move |url: &str| rewrite.replace("{@url}", url)));
        }
        if let Some(rewrite) = self.link_url_rewriter.take() {
            extension.link_url_rewriter =
                Some(Arc::new(move |url: &str| rewrite.replace("{@url}", url)));
        }
        overwrite(
            &mut extension.cjk_friendly_emphasis,
            self.cjk_friendly_emphasis,
        );
        overwrite(&mut extension.phoenix_heex, self.phoenix_heex);
        overwrite(&mut extension.block_directive, self.block_directive);
        overwrite(&mut extension.header_attributes, self.header_attributes);
        overwrite(
            &mut extension.fenced_code_attributes,
            self.fenced_code_attributes,
        );
        overwrite(
            &mut extension.inline_code_attributes,
            self.inline_code_attributes,
        );
        overwrite(&mut extension.link_attributes, self.link_attributes);
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
            smart: optional_field(term, atoms::smart())?,
            default_info_string: optional_field(term, atoms::default_info_string())?,
            relaxed_tasklist_matching: optional_field(term, atoms::relaxed_tasklist_matching())?,
            relaxed_autolinks: optional_field(term, atoms::relaxed_autolinks())?,
            ignore_setext: optional_field(term, atoms::ignore_setext())?,
            tasklist_in_table: optional_field(term, atoms::tasklist_in_table())?,
            leave_footnote_definitions: optional_field(term, atoms::leave_footnote_definitions())?,
            escaped_char_spans: optional_field(term, atoms::escaped_char_spans())?,
            sourcepos_chars: optional_field(term, atoms::sourcepos_chars())?,
        })
    }
}

impl ExParseOptions {
    pub fn apply(self, parse: &mut Parse<'static>) {
        overwrite(&mut parse.smart, self.smart);
        overwrite_optional(&mut parse.default_info_string, self.default_info_string);
        overwrite(
            &mut parse.relaxed_tasklist_matching,
            self.relaxed_tasklist_matching,
        );
        overwrite(&mut parse.relaxed_autolinks, self.relaxed_autolinks);
        overwrite(&mut parse.ignore_setext, self.ignore_setext);
        overwrite(&mut parse.tasklist_in_table, self.tasklist_in_table);
        overwrite(
            &mut parse.leave_footnote_definitions,
            self.leave_footnote_definitions,
        );
        overwrite(&mut parse.escaped_char_spans, self.escaped_char_spans);
        overwrite(&mut parse.sourcepos_chars, self.sourcepos_chars);
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

#[derive(Clone, Debug, Default, NifUnitEnum)]
pub enum ExAlertStyleType {
    #[default]
    Specific,
    Semantic,
}

impl From<ExAlertStyleType> for AlertStyleType {
    fn from(alert_style_type: ExAlertStyleType) -> Self {
        match alert_style_type {
            ExAlertStyleType::Specific => AlertStyleType::Specific,
            ExAlertStyleType::Semantic => AlertStyleType::Semantic,
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
    pub alert_style: Option<ExAlertStyleType>,
}

impl<'a> Decoder<'a> for ExRenderOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            hardbreaks: optional_field(term, atoms::hardbreaks())?,
            github_pre_lang: optional_field(term, atoms::github_pre_lang())?,
            full_info_string: optional_field(term, atoms::full_info_string())?,
            width: optional_field(term, atoms::width())?,
            r#unsafe: optional_field(term, atoms::unsafe_())?,
            escape: optional_field(term, atoms::escape())?,
            list_style: optional_field(term, atoms::list_style())?,
            sourcepos: optional_field(term, atoms::sourcepos())?,
            escaped_char_spans: optional_field(term, atoms::escaped_char_spans())?,
            ignore_empty_links: optional_field(term, atoms::ignore_empty_links())?,
            gfm_quirks: optional_field(term, atoms::gfm_quirks())?,
            prefer_fenced: optional_field(term, atoms::prefer_fenced())?,
            figure_with_caption: optional_field(term, atoms::figure_with_caption())?,
            tasklist_classes: optional_field(term, atoms::tasklist_classes())?,
            ol_width: optional_field(term, atoms::ol_width())?,
            experimental_minimize_commonmark: optional_field(
                term,
                atoms::experimental_minimize_commonmark(),
            )?,
            compact_html: optional_field(term, atoms::compact_html())?,
            alert_style: optional_field(term, atoms::alert_style())?,
        })
    }
}

impl ExRenderOptions {
    pub fn apply(self, render: &mut Render) {
        overwrite(&mut render.hardbreaks, self.hardbreaks);
        overwrite(&mut render.github_pre_lang, self.github_pre_lang);
        overwrite(&mut render.full_info_string, self.full_info_string);
        overwrite(&mut render.width, self.width);
        overwrite(&mut render.r#unsafe, self.r#unsafe);
        overwrite(&mut render.escape, self.escape);
        overwrite(
            &mut render.list_style,
            self.list_style.map(ListStyleType::from),
        );
        overwrite(&mut render.sourcepos, self.sourcepos);
        overwrite(&mut render.escaped_char_spans, self.escaped_char_spans);
        overwrite(&mut render.ignore_empty_links, self.ignore_empty_links);
        overwrite(&mut render.gfm_quirks, self.gfm_quirks);
        overwrite(&mut render.prefer_fenced, self.prefer_fenced);
        overwrite(&mut render.figure_with_caption, self.figure_with_caption);
        overwrite(&mut render.tasklist_classes, self.tasklist_classes);
        overwrite(&mut render.ol_width, self.ol_width);
        overwrite(
            &mut render.experimental_minimize_commonmark,
            self.experimental_minimize_commonmark,
        );
        overwrite(&mut render.compact_html, self.compact_html);
        overwrite(
            &mut render.alert_style,
            self.alert_style.map(AlertStyleType::from),
        );
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
            extension: optional_field(term, atoms::extension())?,
            parse: optional_field(term, atoms::parse())?,
            render: optional_field(term, atoms::render())?,
            syntax_highlight: syntax_highlight_field(term)?,
            sanitize: optional_field(term, atoms::sanitize())?,
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

#[derive(Debug, Default, NifUnitEnum)]
pub enum ExSyntaxHighlightEngine {
    #[default]
    Lumis,
    Syntect,
}

#[derive(Debug, Default)]
pub struct ExLumisOptions {
    #[cfg(feature = "lumis")]
    pub formatter: ExFormatterOption,
    // Lumis moved this off the formatter in 0.8; it arrives alongside it.
    #[cfg(feature = "lumis")]
    pub rainbow_brackets: bool,
}

#[derive(Debug, Default)]
pub struct ExSyntectOptions {
    #[cfg(feature = "syntect")]
    pub theme: Option<String>,
}

#[cfg(feature = "syntect")]
impl<'a> Decoder<'a> for ExSyntectOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            theme: optional_field(term, atoms::theme())?,
        })
    }
}

#[cfg(not(feature = "syntect"))]
impl<'a> Decoder<'a> for ExSyntectOptions {
    fn decode(_term: Term<'a>) -> NifResult<Self> {
        Ok(Self {})
    }
}

#[derive(Debug)]
pub enum ExSyntaxHighlightEngineOptions {
    Lumis(Box<ExLumisOptions>),
    Syntect(ExSyntectOptions),
}

#[cfg(feature = "lumis")]
impl<'a> Decoder<'a> for ExLumisOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            formatter: optional_field(term, atoms::formatter())?.unwrap_or_default(),
            rainbow_brackets: optional_field(term, atoms::rainbow_brackets())?.unwrap_or(false),
        })
    }
}

#[cfg(not(feature = "lumis"))]
impl<'a> Decoder<'a> for ExLumisOptions {
    fn decode(_term: Term<'a>) -> NifResult<Self> {
        Ok(Self {})
    }
}

#[derive(Debug)]
pub struct ExSyntaxHighlightOptions {
    pub opts: ExSyntaxHighlightEngineOptions,
}

impl<'a> Decoder<'a> for ExSyntaxHighlightOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if let Some(engine) = optional_field(term, atoms::engine())? {
            let opts = match engine {
                ExSyntaxHighlightEngine::Lumis => {
                    let opts = optional_field(term, atoms::opts())?.unwrap_or_default();
                    ExSyntaxHighlightEngineOptions::Lumis(Box::new(opts))
                }
                ExSyntaxHighlightEngine::Syntect => {
                    let opts = optional_field(term, atoms::opts())?.unwrap_or_default();
                    ExSyntaxHighlightEngineOptions::Syntect(opts)
                }
            };

            return Ok(Self { opts });
        }

        // Legacy shape: syntax_highlight: [formatter: ...]
        Ok(Self {
            opts: ExSyntaxHighlightEngineOptions::Lumis(Box::new(term.decode()?)),
        })
    }
}
