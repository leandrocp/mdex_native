#[macro_use]
extern crate rustler;

#[cfg(feature = "lumis")]
mod lumis_adapter;
mod types;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::format_html_with_plugins;
use comrak::options::Plugins;
#[cfg(feature = "syntect")]
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Anchorizer, Arena, Options};
use lol_html::html_content::ContentType;
use lol_html::{rewrite_str, text, RewriteStrSettings};
#[cfg(feature = "lumis")]
use lumis_adapter::LumisAdapter;
use rustler::{Encoder, Env, NifResult, Term};
use types::{document::*, options::*};

rustler::init!("Elixir.MDExNative.Native");

#[rustler::nif(schedule = "DirtyCpu")]
fn parse_document<'a>(env: Env<'a>, md: &str, options: ExOptions) -> NifResult<Term<'a>> {
    let comrak_options = options.comrak_options();
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, md, &comrak_options);
    let ex_document = comrak_ast_to_ex_document(root);
    Ok(ex_document.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn markdown_to_html_with_options<'a>(
    env: Env<'a>,
    md: &str,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let (comrak_options, lumis_adapter, sanitize) = render_parts(options)?;
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, md, &comrak_options);
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    format_html_with_plugins(root, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    Ok(do_safe_html(buffer, &sanitize, false, true).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn markdown_to_xml_with_options<'a>(
    env: Env<'a>,
    md: &str,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let (comrak_options, lumis_adapter, _sanitize) = render_parts(options)?;
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, md, &comrak_options);
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    comrak::format_xml_with_plugins(root, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let xml = buffer;
    Ok(xml.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_commonmark(env: Env<'_>, ex_document: ExDocument) -> NifResult<Term<'_>> {
    let arena = Arena::new();
    let ex_node = NewNode::Document(ex_document);
    let comrak_ast = ex_document_to_comrak_ast(&arena, ex_node);
    let mut buffer = String::new();
    let plugins = Plugins::default();
    comrak::format_commonmark_with_plugins(comrak_ast, &Options::default(), &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let commonmark = buffer;
    Ok(commonmark.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_commonmark_with_options<'a>(
    env: Env<'a>,
    ex_document: ExDocument,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let ex_node = NewNode::Document(ex_document);
    let comrak_ast = ex_document_to_comrak_ast(&arena, ex_node);
    let (comrak_options, lumis_adapter, _sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    comrak::format_commonmark_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let document = buffer;
    Ok(document.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_html(env: Env<'_>, ex_document: ExDocument) -> NifResult<Term<'_>> {
    let arena = Arena::new();
    let ex_node = NewNode::Document(ex_document);
    let comrak_ast = ex_document_to_comrak_ast(&arena, ex_node);
    let mut buffer = String::new();
    let options = Options::default();
    format_html_with_plugins(comrak_ast, &options, &mut buffer, &Plugins::default())
        .expect("writing to String is infallible");
    Ok(buffer.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_html_with_options<'a>(
    env: Env<'a>,
    ex_document: ExDocument,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let ex_node = NewNode::Document(ex_document);
    let comrak_ast = ex_document_to_comrak_ast(&arena, ex_node);
    let (comrak_options, lumis_adapter, sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    format_html_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    Ok(do_safe_html(buffer, &sanitize, false, true).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_xml(env: Env<'_>, ex_document: ExDocument) -> NifResult<Term<'_>> {
    let arena = Arena::new();
    let ex_node = NewNode::Document(ex_document);
    let comrak_ast = ex_document_to_comrak_ast(&arena, ex_node);
    let mut buffer = String::new();
    let plugins = Plugins::default();
    comrak::format_xml_with_plugins(comrak_ast, &Options::default(), &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let xml = buffer;
    Ok(xml.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_xml_with_options<'a>(
    env: Env<'a>,
    ex_document: ExDocument,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let ex_node = NewNode::Document(ex_document);
    let comrak_ast = ex_document_to_comrak_ast(&arena, ex_node);
    let (comrak_options, lumis_adapter, _sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    comrak::format_xml_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let xml = buffer;
    Ok(xml.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn text_to_anchor(text: &str) -> String {
    Anchorizer::new().anchorize(text)
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn safe_html(
    env: Env<'_>,
    unsafe_html: String,
    sanitize: Option<ExSanitizeOption>,
    escape_content: bool,
    escape_curly_braces_in_code: bool,
) -> NifResult<Term<'_>> {
    Ok(do_safe_html(
        unsafe_html,
        &sanitize,
        escape_content,
        escape_curly_braces_in_code,
    )
    .encode(env))
}

fn render_parts(
    options: ExOptions,
) -> NifResult<(
    Options<'static>,
    Option<CodeFenceSyntaxHighlighter>,
    Option<ExSanitizeOption>,
)> {
    let comrak_options = options.comrak_options();
    let syntax_highlighter = match options.syntax_highlight {
        Some(syntax_highlight) => Some(syntax_highlighter(syntax_highlight)?),
        None => None,
    };

    Ok((comrak_options, syntax_highlighter, options.sanitize))
}

fn syntax_highlighter(
    syntax_highlight: ExSyntaxHighlightOptions,
) -> NifResult<CodeFenceSyntaxHighlighter> {
    match syntax_highlight.opts {
        ExSyntaxHighlightEngineOptions::Lumis(opts) => {
            #[cfg(feature = "lumis")]
            {
                Ok(CodeFenceSyntaxHighlighter::Lumis(LumisAdapter::new(
                    opts.formatter,
                )))
            }

            #[cfg(not(feature = "lumis"))]
            {
                let _ = opts;
                Err(rustler::Error::Atom("lumis_not_enabled"))
            }
        }
        ExSyntaxHighlightEngineOptions::Syntect(opts) => {
            #[cfg(feature = "syntect")]
            {
                let builder = SyntectAdapterBuilder::new()
                    .syntax_set(two_face::syntax::extra_newlines())
                    .theme_set(two_face::theme::extra().into());

                let adapter = match opts.theme.as_deref() {
                    Some(theme) => builder.theme(theme).build(),
                    None => builder.build(),
                };

                Ok(CodeFenceSyntaxHighlighter::Syntect(adapter))
            }

            #[cfg(not(feature = "syntect"))]
            {
                let _ = opts;
                Err(rustler::Error::Atom("syntect_not_enabled"))
            }
        }
    }
}

enum CodeFenceSyntaxHighlighter {
    #[cfg(feature = "lumis")]
    Lumis(LumisAdapter),
    #[cfg(feature = "syntect")]
    Syntect(SyntectAdapter),
}

#[cfg(any(feature = "lumis", feature = "syntect"))]
impl SyntaxHighlighterAdapter for CodeFenceSyntaxHighlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn std::fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::fmt::Result {
        match self {
            #[cfg(feature = "lumis")]
            Self::Lumis(adapter) => adapter.write_highlighted(output, lang, code),
            #[cfg(feature = "syntect")]
            Self::Syntect(adapter) => adapter.write_highlighted(output, lang, code),
        }
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: std::collections::HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> std::fmt::Result {
        match self {
            #[cfg(feature = "lumis")]
            Self::Lumis(adapter) => adapter.write_pre_tag(output, attributes),
            #[cfg(feature = "syntect")]
            Self::Syntect(adapter) => adapter.write_pre_tag(output, attributes),
        }
    }

    fn write_code_tag(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: std::collections::HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> std::fmt::Result {
        match self {
            #[cfg(feature = "lumis")]
            Self::Lumis(adapter) => adapter.write_code_tag(output, attributes),
            #[cfg(feature = "syntect")]
            Self::Syntect(adapter) => adapter.write_code_tag(output, attributes),
        }
    }
}

#[cfg(not(any(feature = "lumis", feature = "syntect")))]
impl SyntaxHighlighterAdapter for CodeFenceSyntaxHighlighter {
    fn write_highlighted(
        &self,
        _output: &mut dyn std::fmt::Write,
        _lang: Option<&str>,
        _code: &str,
    ) -> std::fmt::Result {
        match *self {}
    }

    fn write_pre_tag(
        &self,
        _output: &mut dyn std::fmt::Write,
        _attributes: std::collections::HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> std::fmt::Result {
        match *self {}
    }

    fn write_code_tag(
        &self,
        _output: &mut dyn std::fmt::Write,
        _attributes: std::collections::HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> std::fmt::Result {
        match *self {}
    }
}

fn plugins(syntax_highlighter: &Option<CodeFenceSyntaxHighlighter>) -> Plugins<'_> {
    let mut plugins = Plugins::default();

    if let Some(syntax_highlighter) = syntax_highlighter {
        plugins.render.codefence_syntax_highlighter = Some(syntax_highlighter);
    }

    plugins
}

fn do_safe_html(
    unsafe_html: String,
    sanitize: &Option<ExSanitizeOption>,
    escape_content: bool,
    escape_curly_braces_in_code: bool,
) -> String {
    let html = match sanitize {
        None => unsafe_html,
        Some(sanitize_option) => sanitize_option.clean(&unsafe_html),
    };

    let html = match escape_curly_braces_in_code {
        true => rewrite_str(
            &html,
            RewriteStrSettings::new().append_element_content_handler(text!("code", |chunk| {
                chunk.replace(
                    &chunk
                        .as_str()
                        .replace('{', "&lbrace;")
                        .replace('}', "&rbrace;"),
                    ContentType::Html,
                );

                Ok(())
            })),
        )
        .unwrap_or(html),
        false => html,
    };

    let html = match escape_content {
        true => v_htmlescape::escape_fmt(&html).to_string(),
        false => html,
    };

    html.replace("&amp;lbrace;", "&lbrace;")
        .replace("&amp;rbrace;", "&rbrace;")
}
