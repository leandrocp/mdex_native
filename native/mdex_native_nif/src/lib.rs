// Without NIF registration, BEAM-only entry points are unused in Rust tests.
#![cfg_attr(test, allow(dead_code))]

#[macro_use]
extern crate rustler;

mod html_formatter;
#[cfg(feature = "lumis")]
mod lumis_adapter;
#[cfg(feature = "lumis")]
mod lumis_render;
#[cfg(feature = "lumis")]
mod lumis_runtime;
mod memory;
mod types;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::nodes::AstNode;
use comrak::options::Plugins;
#[cfg(feature = "syntect")]
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Anchorizer, Arena, Options};
#[cfg(feature = "syntect")]
use html_formatter::InlineCodeHighlighter;
use html_formatter::{InlineCode, MdexFormatter};
use lol_html::html_content::ContentType;
use lol_html::{rewrite_str, text, RewriteStrSettings};
#[cfg(feature = "lumis")]
use lumis_adapter::LumisAdapter;
use rustler::types::list::ListIterator;
use rustler::{Encoder, Env, NifResult, Term};
#[cfg(any(feature = "lumis", feature = "syntect"))]
use std::sync::Arc;
use std::time::{Duration, Instant};
use types::{document::*, options::*};

// Standalone Rust tests have no BEAM host to resolve NIF symbols.
#[cfg(not(test))]
rustler::init!("Elixir.MDExNative.Native");

// Counts each thread's heap bytes for `budget: [parse_memory: bytes]`.
#[global_allocator]
static ALLOCATOR: memory::CountingAllocator = memory::CountingAllocator;

mod atoms {
    rustler::atoms! {
        nodes,
        lumis_error,
        parse_memory_exceeded,
        parse_time_exceeded
    }
}

enum TraversalStep<'a> {
    Enter(&'a AstNode<'a>),
    Exit {
        node: &'a AstNode<'a>,
        child_count: usize,
    },
}

fn document_term_to_comrak_ast<'a>(
    arena: &'a Arena<'a>,
    document: Term<'_>,
) -> NifResult<&'a AstNode<'a>> {
    let mut root = None;
    let mut stack: Vec<(Term, Option<&'a AstNode<'a>>)> = Vec::new();
    stack.try_reserve(1).map_err(|_| rustler::Error::BadArg)?;
    stack.push((document, None));

    while let Some((term, parent)) = stack.pop() {
        let (node, children) = decode_document_node(term)?;
        let node = ex_document_to_comrak_ast(arena, node);

        if let Some(parent) = parent {
            parent.append(node);
        } else if root.is_none() {
            root = Some(node);
        } else {
            return Err(rustler::Error::BadArg);
        }

        stack
            .try_reserve(children.len())
            .map_err(|_| rustler::Error::BadArg)?;

        for child in children.into_iter().rev() {
            stack.push((child, Some(node)));
        }
    }

    root.ok_or(rustler::Error::BadArg)
}

fn decode_document_node(term: Term) -> NifResult<(NewNode, Vec<Term>)> {
    let children = document_children_terms(term)?;
    let term = if children.is_empty() {
        term
    } else {
        document_term_with_empty_children(term)?
    };

    Ok((term.decode()?, children))
}

fn document_children_terms(term: Term) -> NifResult<Vec<Term>> {
    match term.map_get(atoms::nodes()) {
        Ok(nodes) => {
            let iter: ListIterator = nodes.decode()?;
            let mut children = Vec::new();

            for child in iter {
                children
                    .try_reserve(1)
                    .map_err(|_| rustler::Error::BadArg)?;
                children.push(child);
            }

            Ok(children)
        }
        Err(_) => Ok(Vec::new()),
    }
}

fn document_term_with_empty_children(term: Term) -> NifResult<Term> {
    term.map_put(atoms::nodes(), Term::list_new_empty(term.get_env()))
}

fn pop_child_terms_as_list<'a>(
    env: Env<'a>,
    terms: &mut Vec<Term<'a>>,
    count: usize,
) -> NifResult<Term<'a>> {
    let mut list = Term::list_new_empty(env);

    for _ in 0..count {
        let term = terms.pop().ok_or(rustler::Error::BadArg)?;
        list = list.list_prepend(term);
    }

    Ok(list)
}

fn comrak_ast_to_document_term<'a, 'b>(env: Env<'a>, root: &'b AstNode<'b>) -> NifResult<Term<'a>> {
    let mut stack = Vec::new();
    let mut terms = Vec::new();
    stack.try_reserve(1).map_err(|_| rustler::Error::BadArg)?;
    stack.push(TraversalStep::Enter(root));

    while let Some(step) = stack.pop() {
        match step {
            TraversalStep::Enter(node) => {
                let exit_index = stack.len();
                stack.try_reserve(1).map_err(|_| rustler::Error::BadArg)?;
                stack.push(TraversalStep::Exit {
                    node,
                    child_count: 0,
                });

                let mut count: usize = 0;

                for child in node.reverse_children() {
                    count = count.checked_add(1).ok_or(rustler::Error::BadArg)?;
                    stack.try_reserve(1).map_err(|_| rustler::Error::BadArg)?;
                    stack.push(TraversalStep::Enter(child));
                }

                if let Some(TraversalStep::Exit { child_count, .. }) = stack.get_mut(exit_index) {
                    *child_count = count;
                } else {
                    return Err(rustler::Error::BadArg);
                }
            }
            TraversalStep::Exit { node, child_count } => {
                let node = comrak_ast_to_ex_document_shallow(node);
                let term = node.encode(env);

                if child_count > 0 {
                    let children = pop_child_terms_as_list(env, &mut terms, child_count)?;
                    terms.try_reserve(1).map_err(|_| rustler::Error::BadArg)?;
                    terms.push(term.map_put(atoms::nodes(), children)?);
                } else {
                    terms.try_reserve(1).map_err(|_| rustler::Error::BadArg)?;
                    terms.push(term);
                }
            }
        }
    }

    if terms.len() == 1 {
        terms.pop().ok_or(rustler::Error::BadArg)
    } else {
        Err(rustler::Error::BadArg)
    }
}

/// Parses `md`, giving up once `parse_time` milliseconds have passed or the
/// parse has held more than `parse_memory` bytes of heap.
///
/// Comrak checks both limits cooperatively, every 256 lines, inlines or nodes,
/// so a parse can run somewhat past them. A cancelled parse returns no tree, so
/// the caller gets `{:error, {:parse_time_exceeded, parse_time}}` or
/// `{:error, {:parse_memory_exceeded, parse_memory}}` rather than partial output.
fn parse<'a>(
    arena: &'a Arena<'a>,
    md: &str,
    options: &Options,
    parse_time: Option<u64>,
    parse_memory: Option<u64>,
) -> NifResult<&'a AstNode<'a>> {
    // A limit too far out for `Instant` to represent is no limit at all.
    let deadline = parse_time.and_then(|ms| Instant::now().checked_add(Duration::from_millis(ms)));
    if deadline.is_none() && parse_memory.is_none() {
        return Ok(comrak::parse_document(arena, md, options));
    }

    let past_deadline = || deadline.is_some_and(|deadline| Instant::now() >= deadline);
    // Comrak parses on this thread, so the most the thread holds during the
    // parse, beyond what it held before, is the most the parse has held.
    let held_before = memory::start_peak();
    let over_memory = || {
        parse_memory.is_some_and(|max| {
            u64::try_from(memory::peak().wrapping_sub(held_before)).is_ok_and(|peak| peak > max)
        })
    };
    let should_cancel = || over_memory() || past_deadline();
    let exhausted = || {
        if over_memory() {
            rustler::Error::Term(Box::new((atoms::parse_memory_exceeded(), parse_memory)))
        } else {
            rustler::Error::Term(Box::new((atoms::parse_time_exceeded(), parse_time)))
        }
    };

    let root =
        comrak::try_parse_document(arena, md, options, &should_cancel).map_err(|_| exhausted())?;
    // Comrak only asks at its checkpoints, so a parse can pass the last one
    // within its limits and still finish past one. Check once more so it fails too.
    if should_cancel() {
        return Err(exhausted());
    }
    Ok(root)
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn parse_document<'a>(env: Env<'a>, md: &str, options: ExOptions) -> NifResult<Term<'a>> {
    let comrak_options = options.comrak_options();
    let arena = Arena::new();
    let root = parse(
        &arena,
        md,
        &comrak_options,
        options.parse_time_limit(),
        options.parse_memory_limit(),
    )?;
    comrak_ast_to_document_term(env, root)
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn markdown_to_html_with_options<'a>(
    env: Env<'a>,
    md: &str,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let tagfilter = options.tagfilter();
    let time_limit = options.parse_time_limit();
    let memory_limit = options.parse_memory_limit();
    let (comrak_options, lumis_adapter, inline_code, sanitize) = render_parts(options)?;
    let escape_curly_braces_in_code = comrak_options.extension.phoenix_heex;
    let arena = Arena::new();
    let root = parse(&arena, md, &comrak_options, time_limit, memory_limit)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    finish_render(
        MdexFormatter::format_html(
            root,
            &comrak_options,
            &mut buffer,
            &plugins,
            tagfilter,
            inline_code.clone(),
        ),
        &lumis_adapter,
    )?;
    Ok(do_safe_html(buffer, &sanitize, false, escape_curly_braces_in_code).encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn markdown_to_xml_with_options<'a>(
    env: Env<'a>,
    md: &str,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let time_limit = options.parse_time_limit();
    let memory_limit = options.parse_memory_limit();
    let (comrak_options, lumis_adapter, _inline_code, _sanitize) = render_parts(options)?;
    let arena = Arena::new();
    let root = parse(&arena, md, &comrak_options, time_limit, memory_limit)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    finish_render(
        comrak::format_xml_with_plugins(root, &comrak_options, &mut buffer, &plugins),
        &lumis_adapter,
    )?;
    let xml = buffer;
    Ok(xml.encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn document_to_commonmark<'a>(env: Env<'a>, ex_document: Term<'a>) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let mut buffer = String::new();
    let plugins = Plugins::default();
    comrak::format_commonmark_with_plugins(comrak_ast, &Options::default(), &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let commonmark = buffer;
    Ok(commonmark.encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn document_to_commonmark_with_options<'a>(
    env: Env<'a>,
    ex_document: Term<'a>,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let (comrak_options, lumis_adapter, _inline_code, _sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    finish_render(
        comrak::format_commonmark_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins),
        &lumis_adapter,
    )?;
    let document = buffer;
    Ok(document.encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn document_to_html<'a>(env: Env<'a>, ex_document: Term<'a>) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let mut buffer = String::new();
    let options = Options::default();
    MdexFormatter::format_document_with_plugins(
        comrak_ast,
        &options,
        &mut buffer,
        &Plugins::default(),
        None,
    )
    .expect("writing to String is infallible");
    Ok(buffer.encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn document_to_html_with_options<'a>(
    env: Env<'a>,
    ex_document: Term<'a>,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let tagfilter = options.tagfilter();
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let (comrak_options, lumis_adapter, inline_code, sanitize) = render_parts(options)?;
    let escape_curly_braces_in_code = comrak_options.extension.phoenix_heex;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    finish_render(
        MdexFormatter::format_html(
            comrak_ast,
            &comrak_options,
            &mut buffer,
            &plugins,
            tagfilter,
            inline_code.clone(),
        ),
        &lumis_adapter,
    )?;
    Ok(do_safe_html(buffer, &sanitize, false, escape_curly_braces_in_code).encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn document_to_xml<'a>(env: Env<'a>, ex_document: Term<'a>) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let mut buffer = String::new();
    let plugins = Plugins::default();
    comrak::format_xml_with_plugins(comrak_ast, &Options::default(), &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let xml = buffer;
    Ok(xml.encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
fn document_to_xml_with_options<'a>(
    env: Env<'a>,
    ex_document: Term<'a>,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let (comrak_options, lumis_adapter, _inline_code, _sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    finish_render(
        comrak::format_xml_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins),
        &lumis_adapter,
    )?;
    let xml = buffer;
    Ok(xml.encode(env))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
pub fn text_to_anchor(text: &str) -> String {
    Anchorizer::new().anchorize(text)
}

#[cfg_attr(not(test), rustler::nif)]
pub fn dangerous_url(url: &str) -> bool {
    comrak::html::dangerous_url(url)
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyCpu"))]
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
    InlineCode,
    Option<ExSanitizeOption>,
)> {
    let comrak_options = options.comrak_options();
    let (syntax_highlighter, inline_code) = match options.syntax_highlight {
        Some(syntax_highlight) => {
            let enabled = syntax_highlight.inline_code;
            let highlighter = syntax_highlighter(syntax_highlight)?;
            let inline_code = highlighter
                .as_ref()
                .filter(|_| enabled)
                .and_then(CodeFenceSyntaxHighlighter::inline_code);
            (highlighter, inline_code)
        }
        None => (None, None),
    };

    Ok((
        comrak_options,
        syntax_highlighter,
        inline_code,
        options.sanitize,
    ))
}

fn syntax_highlighter(
    syntax_highlight: ExSyntaxHighlightOptions,
) -> NifResult<Option<CodeFenceSyntaxHighlighter>> {
    match syntax_highlight.opts {
        ExSyntaxHighlightEngineOptions::Lumis(opts) => {
            #[cfg(feature = "lumis")]
            {
                Ok(Some(CodeFenceSyntaxHighlighter::Lumis(Arc::new(
                    LumisAdapter::new(
                        opts.formatter,
                        opts.rainbow_brackets,
                        opts.word_diff,
                        opts.budget,
                    ),
                ))))
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
                use once_cell::sync::Lazy;
                use parking_lot::Mutex;
                use std::collections::HashMap;

                static ADAPTERS: Lazy<Mutex<HashMap<Option<String>, Arc<SyntectAdapter>>>> =
                    Lazy::new(|| Mutex::new(HashMap::new()));

                // Serialize initialization, not rendering, so concurrent first use
                // builds only one syntax/theme set for each cached adapter.
                let mut adapters = ADAPTERS.lock();
                if let Some(adapter) = adapters.get(&opts.theme) {
                    return Ok(Some(CodeFenceSyntaxHighlighter::Syntect(Arc::clone(
                        adapter,
                    ))));
                }

                // Comrak's adapter panics on a theme it has not loaded. Render
                // plain text instead, the same as without a highlighter.
                if opts.theme.as_deref().is_some_and(|name| {
                    !two_face::theme::EmbeddedLazyThemeSet::theme_names()
                        .iter()
                        .any(|theme| theme.as_name() == name)
                }) {
                    return Ok(None);
                }

                let builder = SyntectAdapterBuilder::new()
                    .syntax_set(two_face::syntax::extra_newlines())
                    .theme_set(two_face::theme::extra().into());

                let adapter = Arc::new(match opts.theme.as_deref() {
                    Some(theme) => builder.theme(theme).build(),
                    None => builder.build(),
                });

                adapters.insert(opts.theme, Arc::clone(&adapter));

                Ok(Some(CodeFenceSyntaxHighlighter::Syntect(adapter)))
            }

            #[cfg(not(feature = "syntect"))]
            {
                let _ = opts;
                Err(rustler::Error::Atom("syntect_not_enabled"))
            }
        }
    }
}

/// Writing into a `String` cannot fail, so a render only ends in `Err` when the
/// syntax-highlighter adapter refused a code fence. Comrak's adapter trait can
/// only report that as a bare `fmt::Error`, so the reason is collected from the
/// adapter here rather than thrown away.
fn finish_render(
    render: std::fmt::Result,
    syntax_highlighter: &Option<CodeFenceSyntaxHighlighter>,
) -> NifResult<()> {
    render.map_err(|_| {
        let reason = syntax_highlighter
            .as_ref()
            .and_then(CodeFenceSyntaxHighlighter::take_failure)
            .unwrap_or_else(|| "a syntax highlighter refused a code block".to_string());

        rustler::Error::Term(Box::new((atoms::lumis_error(), reason)))
    })
}

impl CodeFenceSyntaxHighlighter {
    fn take_failure(&self) -> Option<String> {
        match self {
            #[cfg(feature = "lumis")]
            Self::Lumis(adapter) => adapter.take_failure(),
            #[cfg(feature = "syntect")]
            Self::Syntect(_) => None,
            #[cfg(not(any(feature = "lumis", feature = "syntect")))]
            _ => None,
        }
    }
}

enum CodeFenceSyntaxHighlighter {
    #[cfg(feature = "lumis")]
    Lumis(Arc<LumisAdapter>),
    #[cfg(feature = "syntect")]
    Syntect(Arc<SyntectAdapter>),
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

impl CodeFenceSyntaxHighlighter {
    fn inline_code(&self) -> InlineCode {
        match self {
            #[cfg(feature = "lumis")]
            Self::Lumis(adapter) => Some(adapter.clone()),
            #[cfg(feature = "syntect")]
            Self::Syntect(adapter) => Some(adapter.clone()),
            #[cfg(not(any(feature = "lumis", feature = "syntect")))]
            _ => None,
        }
    }
}

/// Comrak's adapter only highlights fences, so inline code reuses the pieces it
/// exposes: the spans `write_highlighted` writes, and the one attribute
/// `write_pre_tag` puts on a fence's `<pre>`, the theme's background or the
/// `syntax-highlighting` class, merged here with the span's own attributes.
#[cfg(feature = "syntect")]
impl InlineCodeHighlighter for SyntectAdapter {
    fn write_inline_code(
        &self,
        output: &mut dyn std::fmt::Write,
        source: &str,
        language: &str,
        attrs: &[(String, String)],
    ) -> std::fmt::Result {
        let mut pre = String::new();
        self.write_pre_tag(&mut pre, std::collections::HashMap::new())?;
        let (name, value) = pre
            .strip_prefix("<pre ")
            .and_then(|tag| tag.strip_suffix("\">"))
            .and_then(|tag| tag.split_once("=\""))
            .ok_or(std::fmt::Error)?;

        let mut attrs = attrs.to_vec();
        match attrs.iter_mut().find(|(key, _)| key == name) {
            Some((_, authored)) => *authored = format!("{value} {authored}"),
            None => attrs.insert(0, (name.to_string(), value.to_string())),
        }

        comrak::html::write_opening_tag(output, "code", attrs)?;
        self.write_highlighted(output, Some(language), source)?;
        output.write_str("</code>")
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn iterative_ast_roundtrip_handles_deep_nesting() {
        let markdown = format!("{}boom", "> ".repeat(5_000));
        let options = Options::default();
        let source_arena = Arena::new();
        let target_arena = Arena::new();
        let source_root = comrak::parse_document(&source_arena, &markdown, &options);
        let target_root = clone_ast_iteratively(&target_arena, source_root).unwrap();

        assert_eq!(
            render_html(target_root, &options),
            render_html(source_root, &options)
        );
    }

    #[test]
    fn iterative_ast_roundtrip_preserves_mixed_markdown() {
        let markdown = concat!(
            "# Release Notes\n\n",
            "Intro with **bold**, _emphasis_, [a link](https://example.com), and `inline code`.\n\n",
            "> A quote with nested content.\n",
            ">\n",
            "> - quoted item\n",
            "> - another quoted item\n\n",
            "- first\n",
            "- second\n\n",
            "```elixir\n",
            "IO.puts(\"hello\")\n",
            "```\n"
        );
        let options = Options::default();
        let source_arena = Arena::new();
        let target_arena = Arena::new();
        let source_root = comrak::parse_document(&source_arena, markdown, &options);
        let target_root = clone_ast_iteratively(&target_arena, source_root).unwrap();

        assert_eq!(
            render_html(target_root, &options),
            render_html(source_root, &options)
        );
    }

    fn clone_ast_iteratively<'a, 'b>(
        arena: &'a Arena<'a>,
        root: &'b AstNode<'b>,
    ) -> NifResult<&'a AstNode<'a>> {
        let mut cloned_root = None;
        let mut stack: Vec<(&'b AstNode<'b>, Option<&'a AstNode<'a>>)> = vec![(root, None)];

        while let Some((source_node, parent)) = stack.pop() {
            let target_node =
                ex_document_to_comrak_ast(arena, comrak_ast_to_ex_document_shallow(source_node));

            if let Some(parent) = parent {
                parent.append(target_node);
            } else if cloned_root.is_none() {
                cloned_root = Some(target_node);
            } else {
                return Err(rustler::Error::BadArg);
            }

            for child in source_node.reverse_children() {
                stack.push((child, Some(target_node)));
            }
        }

        cloned_root.ok_or(rustler::Error::BadArg)
    }

    fn render_html<'a>(root: &'a AstNode<'a>, options: &Options) -> String {
        let mut output = String::new();

        MdexFormatter::format_document_with_plugins(
            root,
            options,
            &mut output,
            &Plugins::default(),
            None,
        )
        .expect("writing to String is infallible");

        output
    }

    /// Comrak's adapter panics on a theme it has not loaded, so an unknown
    /// name gets no highlighter and the document renders as plain text.
    #[cfg(feature = "syntect")]
    #[test]
    fn unknown_syntect_theme_renders_plain_text() {
        let highlighter = |theme: &str| {
            syntax_highlighter(ExSyntaxHighlightOptions {
                opts: ExSyntaxHighlightEngineOptions::Syntect(ExSyntectOptions {
                    theme: Some(theme.to_string()),
                }),
                inline_code: true,
            })
            .unwrap()
        };

        assert!(highlighter("not-a-theme").is_none());
        assert!(highlighter("InspiredGitHub").is_some());
    }

    #[cfg(feature = "syntect")]
    #[test]
    fn highlights_inline_code_with_syntect() {
        let options = Options {
            extension: comrak::options::Extension {
                inline_code_attributes: true,
                ..Default::default()
            },
            render: comrak::options::Render {
                r#unsafe: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let render = |markdown: &str, adapter: SyntectAdapter| {
            let arena = Arena::new();
            let root = comrak::parse_document(&arena, markdown, &options);
            let mut output = String::new();
            MdexFormatter::format_document_with_plugins(
                root,
                &options,
                &mut output,
                &Plugins::default(),
                Some(Arc::new(adapter)),
            )
            .unwrap();
            output
        };
        let themed = || SyntectAdapterBuilder::new().theme("InspiredGitHub").build();

        assert_eq!(
            render("`let x = 1;{:rust}` `{:ok}`\n", themed()),
            "<p><code style=\"background-color:#ffffff;\"><span style=\"font-weight:bold;color:#a71d5d;\">let</span><span style=\"color:#323232;\"> x </span><span style=\"font-weight:bold;color:#a71d5d;\">= </span><span style=\"color:#0086b3;\">1</span><span style=\"color:#323232;\">;</span></code> <code>{:ok}</code></p>\n"
        );
        // The theme's background is merged into an authored style.
        assert!(
            render("`1`{.language-rust style=\"padding:0\"}\n", themed()).starts_with(
                "<p><code class=\"language-rust\" style=\"background-color:#ffffff; padding:0\">"
            )
        );
        assert!(render("`1{:rust}`\n", SyntectAdapterBuilder::new().build())
            .starts_with("<p><code class=\"syntax-highlighting\"><span class=\""));
    }
}
