#[macro_use]
extern crate rustler;

#[cfg(feature = "lumis")]
mod lumis_adapter;
mod types;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::format_html_with_plugins;
use comrak::nodes::AstNode;
use comrak::options::Plugins;
#[cfg(feature = "syntect")]
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Anchorizer, Arena, Options};
use lol_html::html_content::ContentType;
use lol_html::{rewrite_str, text, RewriteStrSettings};
#[cfg(feature = "lumis")]
use lumis_adapter::LumisAdapter;
use rustler::types::list::ListIterator;
use rustler::{Encoder, Env, NifResult, Term};
use types::{document::*, options::*};

rustler::init!("Elixir.MDExNative.Native");

mod atoms {
    rustler::atoms! {
        nodes
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

#[rustler::nif(schedule = "DirtyCpu")]
fn parse_document<'a>(env: Env<'a>, md: &str, options: ExOptions) -> NifResult<Term<'a>> {
    let comrak_options = options.comrak_options();
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, md, &comrak_options);
    comrak_ast_to_document_term(env, root)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn markdown_to_html_with_options<'a>(
    env: Env<'a>,
    md: &str,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let escape_curly_braces_in_code = options.escape_curly_braces_in_code.unwrap_or(true);
    let (comrak_options, lumis_adapter, sanitize) = render_parts(options)?;
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, md, &comrak_options);
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    format_html_with_plugins(root, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    Ok(do_safe_html(buffer, &sanitize, false, escape_curly_braces_in_code).encode(env))
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

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_commonmark_with_options<'a>(
    env: Env<'a>,
    ex_document: Term<'a>,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let (comrak_options, lumis_adapter, _sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    comrak::format_commonmark_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    let document = buffer;
    Ok(document.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_html<'a>(env: Env<'a>, ex_document: Term<'a>) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let mut buffer = String::new();
    let options = Options::default();
    format_html_with_plugins(comrak_ast, &options, &mut buffer, &Plugins::default())
        .expect("writing to String is infallible");
    Ok(buffer.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_html_with_options<'a>(
    env: Env<'a>,
    ex_document: Term<'a>,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let escape_curly_braces_in_code = options.escape_curly_braces_in_code.unwrap_or(true);
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
    let (comrak_options, lumis_adapter, sanitize) = render_parts(options)?;
    let mut buffer = String::new();
    let plugins = plugins(&lumis_adapter);

    format_html_with_plugins(comrak_ast, &comrak_options, &mut buffer, &plugins)
        .expect("writing to String is infallible");
    Ok(do_safe_html(buffer, &sanitize, false, escape_curly_braces_in_code).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
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

#[rustler::nif(schedule = "DirtyCpu")]
fn document_to_xml_with_options<'a>(
    env: Env<'a>,
    ex_document: Term<'a>,
    options: ExOptions,
) -> NifResult<Term<'a>> {
    let arena = Arena::new();
    let comrak_ast = document_term_to_comrak_ast(&arena, ex_document)?;
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

#[rustler::nif]
pub fn dangerous_url(url: &str) -> bool {
    comrak::html::dangerous_url(url)
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
        Some(syntax_highlight) => Some(syntax_highlighter(
            syntax_highlight,
            comrak_options.render.r#unsafe,
        )?),
        None => None,
    };

    Ok((comrak_options, syntax_highlighter, options.sanitize))
}

fn syntax_highlighter(
    syntax_highlight: ExSyntaxHighlightOptions,
    render_unsafe: bool,
) -> NifResult<CodeFenceSyntaxHighlighter> {
    #[cfg(not(feature = "lumis"))]
    let _ = render_unsafe;

    match syntax_highlight.opts {
        ExSyntaxHighlightEngineOptions::Lumis(opts) => {
            #[cfg(feature = "lumis")]
            {
                Ok(CodeFenceSyntaxHighlighter::Lumis(LumisAdapter::new(
                    opts.formatter,
                    render_unsafe,
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

        format_html_with_plugins(root, options, &mut output, &Plugins::default())
            .expect("writing to String is infallible");

        output
    }
}
