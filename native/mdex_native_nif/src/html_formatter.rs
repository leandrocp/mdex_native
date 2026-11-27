//! HTML formatter that renders node attributes.
//!
//! comrak renders parsed attributes itself when both `render.attributes` and
//! `render.unsafe` are set, on headings, code blocks, inline code, links and
//! images. With `render.unsafe` alone, this formatter renders them on links and
//! images: it delegates every node to `format_node_default` except `Link` and
//! `Image`, where, when there are attributes to write and comrak won't, it
//! writes the same markup comrak would, plus the attributes.
//! The optional compatibility entry point also retains deprecated tag filtering
//! for raw HTML nodes. Inline code that names its language goes to the
//! [`InlineCodeHighlighter`] passed as the render's user data, if any.
//!
//! Attribute pairs are written verbatim rather than `data-` prefixed, so
//! `{target=_blank}` renders as `target="_blank"`. As in comrak, an attribute
//! comrak writes itself, such as a link's `href` or `data-sourcepos`, takes
//! precedence over one with the same name, which is skipped.
//!
//! Attributes are document content that can execute, in the same class as raw
//! HTML and `javascript:` links, so they follow the rule comrak applies to
//! those: nothing is written unless `render.unsafe` is set. Values are escaped.
//! Keys are valid HTML attribute names: the parser only produces those, and
//! attributes set from Elixir are filtered when the document is converted.
//!
//! # Where the mirrored code comes from
//!
//! `render_link` and `render_image` below are comrak's own, writing the
//! attributes without checking `render.attributes`. comrak keeps them private,
//! so they cannot be called or wrapped. The source is comrak 1.2.0:
//!
//! - [`render_link`](https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/html.rs#L1047-L1091)
//! - [`render_image`](https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/html.rs#L961-L1011)
//!
//! `mirrors_comrak_attributes` renders links and images with attributes through
//! this code and through comrak with `render.attributes`, and fails when they
//! differ.

use comrak::create_formatter;
use comrak::html::{self, ChildRendering, Context};
use comrak::nodes::{Attributes, Node, NodeCode, NodeLink, NodeValue};
use std::fmt::{self, Write};
use std::sync::Arc;

mod tagfilter;

/// Writes inline code that names its language as a highlighted `<code>`.
pub trait InlineCodeHighlighter {
    fn write_inline_code(
        &self,
        output: &mut dyn Write,
        source: &str,
        language: &str,
        attrs: &[(String, String)],
    ) -> fmt::Result;
}

/// Closes what a syntax highlighter opened around a fence outside its `<pre>`,
/// such as a Lumis header, once Comrak has written the fence's `</code></pre>`.
pub trait CodeBlockCloser {
    fn close_code_block(&self, output: &mut dyn Write) -> fmt::Result;
}

/// The highlighters a render passes to the formatter as its user data.
#[derive(Clone, Default)]
pub struct Highlighters {
    pub inline_code: Option<Arc<dyn InlineCodeHighlighter>>,
    pub code_block: Option<Arc<dyn CodeBlockCloser>>,
}

create_formatter!(MdexFormatter<Highlighters>, {
    NodeValue::CodeBlock(_) => |context, node, entering| {
        return render_code_block(context, node, entering);
    },
    NodeValue::Code(ref nc) => |context, node, entering| {
        return render_code(context, node, entering, nc);
    },
    NodeValue::Image(ref nl) => |context, node, entering| {
        return render_image(context, node, entering, nl);
    },
    NodeValue::Link(ref nl) => |context, node, entering| {
        return render_link(context, node, entering, nl);
    },
});

impl MdexFormatter {
    /// Formats HTML, applying the deprecated tag filter only when requested.
    pub fn format_html<'a, 'o, 'c: 'o>(
        root: Node<'a>,
        options: &'o comrak::Options<'c>,
        output: &'o mut dyn Write,
        plugins: &'o comrak::options::Plugins<'o>,
        tagfilter: bool,
        highlighters: Highlighters,
    ) -> fmt::Result {
        if !tagfilter || !options.render.r#unsafe || options.render.escape {
            return Self::format_document_with_plugins(
                root,
                options,
                output,
                plugins,
                highlighters,
            )
            .map(drop);
        }
        html::format_document_with_formatter(
            root,
            options,
            output,
            plugins,
            |context, node, entering| {
                match &node.data().value {
                    NodeValue::HtmlBlock(block) => {
                        return tagfilter::render_block(context, entering, &block.literal);
                    }
                    NodeValue::HtmlInline(literal) => {
                        return tagfilter::render_inline(context, entering, literal);
                    }
                    _ => {}
                }
                Self::formatter(context, node, entering)
            },
            highlighters,
        )
        .map(drop)
    }
}

/// Whether this node has attributes that are going to be written.
///
/// Attributes are document content that can execute, in the same class as raw
/// HTML and `javascript:` links, so they follow the rule comrak applies to
/// those: nothing is written unless `render.unsafe` is set.
fn writes_attrs<T>(context: &Context<T>, node: Node<'_>) -> bool {
    context.options.render.r#unsafe && node.data().attrs.is_some()
}

/// Whether this formatter writes the attributes of a link or image itself.
///
/// With `render.attributes` comrak writes them, so rendering is comrak's to do,
/// as it is when there is nothing to add.
fn mirrors_attrs<T>(context: &Context<T>, node: Node<'_>) -> bool {
    writes_attrs(context, node) && !context.options.render.attributes
}

/// The names comrak wrote on the tag itself, which a node's attributes cannot
/// override: `names`, plus `data-sourcepos` when source positions are on.
fn written<'n, T>(context: &Context<T>, names: &[&'n str]) -> Vec<&'n str> {
    let mut written = names.to_vec();
    if context.options.render.sourcepos {
        written.push("data-sourcepos");
    }
    written
}

/// Highlights inline code that names its language, as rehype-lumis and
/// markdown-it-lumis do: a `language-*` class or `data-language` attribute, or
/// else a trailing `{:lang}` marker, which is removed. Anything else, including
/// `nohighlight`, keeps comrak's output.
/// Renders a code block as Comrak does, then lets the syntax highlighter close
/// whatever it opened around the fence, which Comrak's adapter cannot.
fn render_code_block(
    context: &mut Context<Highlighters>,
    node: Node<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let rendering = html::format_node_default(context, node, entering)?;

    if let (true, Some(closer)) = (entering, context.user.code_block.clone()) {
        closer.close_code_block(context)?;
    }

    Ok(rendering)
}

fn render_code(
    context: &mut Context<Highlighters>,
    node: Node<'_>,
    entering: bool,
    nc: &NodeCode,
) -> Result<ChildRendering, fmt::Error> {
    let ast = node.data();
    let marked = context
        .user
        .inline_code
        .clone()
        .zip(inline_code_language(&nc.literal, ast.attrs.as_deref()));
    let (Some((highlighter, (source, language))), true) = (marked, entering) else {
        return html::format_node_default(context, node, entering);
    };

    let mut attrs = Vec::new();
    if context.options.render.sourcepos && ast.sourcepos.start.line > 0 {
        attrs.push(("data-sourcepos".to_string(), ast.sourcepos.to_string()));
    }
    if let (true, Some(authored)) = (writes_attrs(context, node), ast.attrs.as_deref()) {
        let written = written(context, &[]);
        attrs.extend(authored.id.iter().map(|id| ("id".to_string(), id.clone())));
        if !authored.classes.is_empty() {
            attrs.push(("class".to_string(), authored.classes.join(" ")));
        }
        attrs.extend(
            authored
                .pairs
                .iter()
                .filter(|(key, _)| !is_taken(&written, key))
                .cloned(),
        );
    }

    highlighter.write_inline_code(context, source, language, &attrs)?;
    Ok(ChildRendering::HTML)
}

fn inline_code_language<'a>(
    literal: &'a str,
    attrs: Option<&'a Attributes>,
) -> Option<(&'a str, &'a str)> {
    let classes = attrs.map_or(&[][..], |attrs| &attrs.classes);
    if classes
        .iter()
        .any(|class| class == "nohighlight" || class == "no-highlight")
    {
        return None;
    }

    let language = classes
        .iter()
        .find_map(|class| class.strip_prefix("language-"))
        .or_else(|| {
            attrs?
                .pairs
                .iter()
                .find_map(|(key, value)| (key == "data-language").then_some(value.as_str()))
        })
        .filter(|language| !language.is_empty());

    match language {
        Some(language) => Some((literal, language)),
        None => split_language_suffix(literal),
    }
}

/// Splits `` `x = 1{:python}` `` into its code and language. A name starting
/// with `.` is rehype-pretty-code's highlight scope, not a language.
fn split_language_suffix(literal: &str) -> Option<(&str, &str)> {
    let (code, language) = literal.strip_suffix('}')?.rsplit_once("{:")?;
    let valid = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '#' | '-');

    (!code.is_empty()
        && language.starts_with(valid)
        && language.chars().all(|c| valid(c) || c == '.'))
    .then_some((code, language))
}

/// Mirrors [comrak's `render_link`][upstream], writing attributes on the
/// opening tag without `render.attributes`.
///
/// Nodes this formatter has nothing to add to go to `format_node_default`, the
/// same function the generated formatter falls back to for every node type not
/// handled here, so comrak keeps ownership of link rendering.
///
/// [upstream]: https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/html.rs#L1047-L1091
fn render_link<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if !mirrors_attrs(context, node) {
        return html::format_node_default(context, node, entering);
    }

    let parent_is_link = node
        .parent()
        .is_some_and(|parent| matches!(parent.data().value, NodeValue::Link(..)));

    if context.options.parse.relaxed_autolinks && parent_is_link {
        return Ok(ChildRendering::HTML);
    }

    if entering {
        context.write_str("<a")?;
        html::render_sourcepos(context, node)?;
        context.write_str(" href=\"")?;
        if context.options.render.r#unsafe || !html::dangerous_url(&nl.url) {
            match &context.options.extension.link_url_rewriter {
                Some(rewriter) => context.escape_href(&rewriter.to_html(&nl.url))?,
                None => context.escape_href(&nl.url)?,
            }
        }
        context.write_str("\"")?;
        if !nl.title.is_empty() {
            context.write_str(" title=\"")?;
            context.escape(&nl.title)?;
            context.write_str("\"")?;
        }
        let names: &[&str] = if nl.title.is_empty() {
            &["href"]
        } else {
            &["href", "title"]
        };
        write_attrs(context, node, &written(context, names))?;
        context.write_str(">")?;
    } else {
        context.write_str("</a>")?;
    }

    Ok(ChildRendering::HTML)
}

/// Mirrors [comrak's `render_image`][upstream], writing attributes on the
/// `<img>` tag without `render.attributes`.
///
/// The children of an image render as its `alt` text, which is why entering
/// returns `ChildRendering::Plain` and the tag is closed on exit. As with
/// `render_link`, nodes this formatter has nothing to add to are left to comrak.
///
/// [upstream]: https://github.com/kivikakk/comrak/blob/d9cf1705672f5a34e959a8db7c388a998c5d1b6b/src/html.rs#L961-L1011
fn render_image<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if !mirrors_attrs(context, node) {
        return html::format_node_default(context, node, entering);
    }

    if entering {
        if context.options.render.figure_with_caption {
            context.write_str("<figure>")?;
        }
        context.write_str("<img")?;
        html::render_sourcepos(context, node)?;
        context.write_str(" src=\"")?;
        if context.options.render.r#unsafe || !html::dangerous_url(&nl.url) {
            match &context.options.extension.image_url_rewriter {
                Some(rewriter) => context.escape_href(&rewriter.to_html(&nl.url))?,
                None => context.escape_href(&nl.url)?,
            }
        }
        context.write_str("\" alt=\"")?;
        return Ok(ChildRendering::Plain);
    }

    context.write_str("\"")?;
    if !nl.title.is_empty() {
        context.write_str(" title=\"")?;
        context.escape(&nl.title)?;
        context.write_str("\"")?;
    }
    let names: &[&str] = if nl.title.is_empty() {
        &["src", "alt"]
    } else {
        &["src", "alt", "title"]
    };
    write_attrs(context, node, &written(context, names))?;
    context.write_str(" />")?;

    if context.options.render.figure_with_caption {
        if !nl.title.is_empty() {
            context.write_str("<figcaption>")?;
            context.escape(&nl.title)?;
            context.write_str("</figcaption>")?;
        }
        context.write_str("</figure>")?;
    }

    Ok(ChildRendering::HTML)
}

/// Writes `id`, `class` and key/value attributes from the node's `attrs`,
/// skipping any named in `written`, as comrak's own `write_attributes` does.
///
/// Only called when `mirrors_attrs` holds. Name/value pairs are written the way
/// comrak's `write_opening_tag` writes them — ` name="escaped value"` — which is
/// not reusable directly because it also emits the tag name and the closing `>`,
/// and an `href` needs `escape_href` rather than `escape`.
fn write_attrs<T>(context: &mut Context<T>, node: Node<'_>, written: &[&str]) -> fmt::Result {
    let ast = node.data();
    let Some(attrs) = ast.attrs.as_deref() else {
        return Ok(());
    };

    if let Some(id) = attrs.id.as_ref().filter(|_| !is_taken(written, "id")) {
        context.write_str(" id=\"")?;
        context.escape(id)?;
        context.write_str("\"")?;
    }

    if !attrs.classes.is_empty() && !is_taken(written, "class") {
        context.write_str(" class=\"")?;
        for (index, class) in attrs.classes.iter().enumerate() {
            if index > 0 {
                context.write_str(" ")?;
            }
            context.escape(class)?;
        }
        context.write_str("\"")?;
    }

    for (key, value) in &attrs.pairs {
        if is_taken(written, key) {
            continue;
        }
        context.write_str(" ")?;
        context.write_str(key)?;
        context.write_str("=\"")?;
        context.escape(value)?;
        context.write_str("\"")?;
    }

    Ok(())
}

/// Whether comrak already wrote `name`. HTML attribute names are ASCII
/// case-insensitive, so `HREF` is as taken as `href`.
fn is_taken(written: &[&str], name: &str) -> bool {
    written.iter().any(|taken| taken.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::nodes::{Attributes, NodeValue};
    use comrak::options::Plugins;
    use comrak::{Arena, Options};
    use pretty_assertions::assert_eq;

    fn render(markdown: &str, options: &Options) -> String {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, markdown, options);
        let mut buffer = String::new();
        MdexFormatter::format_document_with_plugins(
            root,
            options,
            &mut buffer,
            &Plugins::default(),
            Highlighters::default(),
        )
        .unwrap();
        buffer
    }

    fn comrak_html(markdown: &str, options: &Options) -> String {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, markdown, options);
        let mut buffer = String::new();
        comrak::format_html(root, options, &mut buffer).unwrap();
        buffer
    }

    /// Parses `markdown` and sets `attrs` on every node matching `matches`,
    /// which is how a plugin reaches the field from Elixir, then renders it
    /// with `format`.
    fn format_with_attrs(
        markdown: &str,
        options: &Options,
        matches: fn(&NodeValue) -> bool,
        attrs: &Attributes,
        format: fn(Node<'_>, &Options, &mut String) -> fmt::Result,
    ) -> String {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, markdown, options);

        for node in root.descendants() {
            let mut ast = node.data_mut();
            if matches(&ast.value) {
                ast.attrs = Some(Box::new(attrs.clone()));
            }
        }

        let mut buffer = String::new();
        format(root, options, &mut buffer).unwrap();
        buffer
    }

    fn render_with_attrs(
        markdown: &str,
        options: &Options,
        matches: fn(&NodeValue) -> bool,
        attrs: Attributes,
    ) -> String {
        format_with_attrs(
            markdown,
            options,
            matches,
            &attrs,
            |root, options, buffer| {
                MdexFormatter::format_document_with_plugins(
                    root,
                    options,
                    buffer,
                    &Plugins::default(),
                    Highlighters::default(),
                )
                .map(drop)
            },
        )
    }

    fn options(configure: impl FnOnce(&mut Options<'static>)) -> Options<'static> {
        let mut options = Options::default();
        configure(&mut options);
        options
    }

    /// Attribute extensions on, `unsafe` on, which is what it takes for
    /// attributes on links and images to reach the output.
    fn attr_options() -> Options<'static> {
        options(|options| {
            options.extension.header_attributes = true;
            options.extension.fenced_code_attributes = true;
            options.extension.inline_code_attributes = true;
            options.extension.link_attributes = true;
            options.render.r#unsafe = true;
        })
    }

    /// `unsafe` on, no extensions: the plugin case, where attributes are set on
    /// the AST rather than parsed.
    fn unsafe_options() -> Options<'static> {
        options(|options| options.render.r#unsafe = true)
    }

    fn attributes(id: Option<&str>, classes: &[&str], pairs: &[(&str, &str)]) -> Attributes {
        let mut attrs = Attributes::default();
        attrs.id = id.map(str::to_string);
        attrs.classes = classes.iter().map(|c| c.to_string()).collect();
        attrs.pairs = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        attrs
    }

    #[test]
    fn renders_parsed_link_attributes() {
        assert_eq!(
            render(
                "[link](https://example.com){#docs .external rel=nofollow}\n",
                &attr_options()
            ),
            "<p><a href=\"https://example.com\" id=\"docs\" class=\"external\" rel=\"nofollow\">link</a></p>\n"
        );
    }

    #[test]
    fn renders_parsed_image_attributes() {
        assert_eq!(
            render("![cat](cat.png){width=100%}\n", &attr_options()),
            "<p><img src=\"cat.png\" alt=\"cat\" width=\"100%\" /></p>\n"
        );
    }

    /// Only Link and Image are handled, so without `render.attributes` inline
    /// code keeps comrak's output even with `inline_code_attributes` and
    /// `unsafe` on.
    #[test]
    fn leaves_inline_code_to_comrak() {
        assert_eq!(
            render("`:ok`{.language-elixir}\n", &attr_options()),
            "<p><code>:ok</code></p>\n"
        );
    }

    /// With `render.attributes` comrak writes every node's attributes, links
    /// and images included, so each is written once.
    #[test]
    fn leaves_attributes_to_comrak_with_render_attributes() {
        let options = options(|options| {
            *options = attr_options();
            options.render.attributes = true;
        });
        let markdown = "# Title {#t}\n\n[a](/u){.x} ![i](i.png){width=1} `c`{.y}\n";

        assert_eq!(
            render(markdown, &options),
            "<h1 id=\"t\">Title</h1>\n\
             <p><a href=\"/u\" class=\"x\">a</a> <img src=\"i.png\" alt=\"i\" width=\"1\" /> \
             <code class=\"y\">c</code></p>\n"
        );
        assert_eq!(render(markdown, &options), comrak_html(markdown, &options));
    }

    /// Writes what the formatter handed over, so a test sees the split.
    struct Stub;

    impl InlineCodeHighlighter for Stub {
        fn write_inline_code(
            &self,
            output: &mut dyn Write,
            source: &str,
            language: &str,
            attrs: &[(String, String)],
        ) -> fmt::Result {
            write!(output, "<hl {language} {attrs:?}>{source}</hl>")
        }
    }

    fn render_inline(markdown: &str, options: &Options) -> String {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, markdown, options);
        let mut buffer = String::new();
        MdexFormatter::format_html(
            root,
            options,
            &mut buffer,
            &Plugins::default(),
            false,
            Highlighters {
                inline_code: Some(Arc::new(Stub)),
                code_block: None,
            },
        )
        .unwrap();
        buffer
    }

    /// The cases of lumis's `fixtures/inline-code-markers.json`, which
    /// rehype-lumis and markdown-it-lumis test against too.
    #[test]
    fn splits_the_language_suffix_like_the_lumis_integrations() {
        for (text, expected) in [
            (
                "const x = 1{:javascript}",
                Some(("const x = 1", "javascript")),
            ),
            ("%{a: 1} {:elixir}", Some(("%{a: 1} ", "elixir"))),
            (
                r#"{"a": "{:x}"}{:json}"#,
                Some((r#"{"a": "{:x}"}"#, "json")),
            ),
            ("{:js}", None),
            ("x{:js} y", None),
            ("x{:}", None),
            ("x{:.keyword}", None),
            ("x{:java script}", None),
            ("const x = 1", None),
        ] {
            assert_eq!(split_language_suffix(text), expected, "{text}");
        }
    }

    #[test]
    fn highlights_inline_code_that_names_its_language() {
        let safe = options(|options| options.extension = attr_options().extension);

        assert_eq!(
            render_inline("`:ok{:elixir}` `{:ok}`\n", &safe),
            "<p><hl elixir []>:ok</hl> <code>{:ok}</code></p>\n"
        );
        // A class names the language and keeps the suffix as code; authored
        // attributes go through only with `unsafe`, as on links.
        assert_eq!(
            render_inline("`a{:x}`{#i .language-rust k=v}\n", &safe),
            "<p><hl rust []>a{:x}</hl></p>\n"
        );
        assert_eq!(
            render_inline("`a`{#i .language-rust data-language=go}\n", &attr_options()),
            r#"<p><hl rust [("id", "i"), ("class", "language-rust"), ("data-language", "go")]>a</hl></p>"#
                .to_string()
                + "\n"
        );
        assert_eq!(
            render_inline("`a`{data-language=go}\n", &safe),
            "<p><hl go []>a</hl></p>\n"
        );
        assert_eq!(
            render_inline("`a{:rust}`{.nohighlight}\n", &safe),
            "<p><code>a{:rust}</code></p>\n"
        );
    }

    /// Comrak's own `data-sourcepos` takes precedence over an authored one.
    #[test]
    fn keeps_sourcepos_on_highlighted_inline_code() {
        let options = options(|options| {
            *options = attr_options();
            options.render.sourcepos = true;
        });

        assert_eq!(
            render_inline("`a{:rust}`{data-sourcepos=x k=v}\n", &options),
            r#"<p data-sourcepos="1:1-1:32"><hl rust [("data-sourcepos", "1:1-1:32"), ("k", "v")]>a</hl></p>"#
                .to_string()
                + "\n"
        );
    }

    /// Without `render.unsafe` attributes are dropped, the way comrak drops raw
    /// HTML and `javascript:` hrefs. This is what keeps `{onclick=alert(1)}` out
    /// of the default output.
    #[test]
    fn omits_attributes_unless_unsafe() {
        let safe_options = options(|options| {
            options.extension = attr_options().extension;
            options.render.attributes = true;
        });
        let markdown = "[link](https://example.com){#docs .external rel=nofollow}\n";

        assert_eq!(
            render(markdown, &safe_options),
            "<p><a href=\"https://example.com\">link</a></p>\n"
        );

        // With nothing to add the node goes to `format_node_default`, so this is
        // comrak's own output rather than a copy of it.
        assert_eq!(
            render(markdown, &safe_options),
            comrak_html(markdown, &safe_options)
        );

        assert_eq!(
            render_with_attrs(
                "[link](https://example.com)\n",
                &Options::default(),
                |value| matches!(value, NodeValue::Link(..)),
                attributes(Some("i"), &["c"], &[("onclick", "alert(1)")]),
            ),
            "<p><a href=\"https://example.com\">link</a></p>\n"
        );
    }

    /// The case from mdex#412: attributes set on the AST, not parsed from
    /// markdown, with no extension enabled.
    #[test]
    fn renders_programmatic_link_attributes() {
        assert_eq!(
            render_with_attrs(
                "[link](https://example.com)\n",
                &unsafe_options(),
                |value| matches!(value, NodeValue::Link(..)),
                attributes(None, &[], &[("target", "_blank"), ("rel", "noopener")]),
            ),
            "<p><a href=\"https://example.com\" target=\"_blank\" rel=\"noopener\">link</a></p>\n"
        );
    }

    #[test]
    fn escapes_attribute_values() {
        assert_eq!(
            render_with_attrs(
                "[link](https://example.com)\n",
                &unsafe_options(),
                |value| matches!(value, NodeValue::Link(..)),
                attributes(
                    Some("a\"b"),
                    &["c<d"],
                    &[("data-x", "\" onclick=\"alert(1)")]
                ),
            ),
            "<p><a href=\"https://example.com\" id=\"a&quot;b\" class=\"c&lt;d\" \
             data-x=\"&quot; onclick=&quot;alert(1)\">link</a></p>\n"
        );
    }

    /// As in comrak, attributes comrak writes itself win over authored ones.
    #[test]
    fn skips_attributes_comrak_writes() {
        assert_eq!(
            render(
                "[a](/u \"Title\"){href=/v title=Other data-x=1} [b](/u){title=Used}\n",
                &attr_options()
            ),
            "<p><a href=\"/u\" title=\"Title\" data-x=\"1\">a</a> <a href=\"/u\" title=\"Used\">b</a></p>\n"
        );
        assert_eq!(
            render("![i](i.png){SRC=x Alt=y w=1}\n", &attr_options()),
            "<p><img src=\"i.png\" alt=\"i\" w=\"1\" /></p>\n"
        );
    }

    #[test]
    fn renders_titles_and_attributes_together() {
        assert_eq!(
            render_with_attrs(
                "[link](https://example.com \"A title\")\n",
                &unsafe_options(),
                |value| matches!(value, NodeValue::Link(..)),
                attributes(None, &[], &[("target", "_blank")]),
            ),
            "<p><a href=\"https://example.com\" title=\"A title\" target=\"_blank\">link</a></p>\n"
        );
    }

    /// Nodes without attributes must render exactly as comrak renders them.
    #[test]
    fn matches_comrak_without_attributes() {
        let markdown = concat!(
            "# Heading\n\n",
            "Text with [a link](https://example.com \"title\"), `inline code`,\n",
            "![an image](image.png \"image title\"), <span>raw</span>, and *emphasis*.\n\n",
            "[empty]()\n\n",
            "[dangerous](javascript:alert(1))\n\n",
            "```elixir\n:ok\n```\n\n",
            "> quote\n\n",
            "| a | b |\n| - | - |\n| 1 | 2 |\n",
        );

        for options in [
            Options::default(),
            attr_options(),
            options(|options| {
                options.render.r#unsafe = true;
                options.render.sourcepos = true;
                options.render.figure_with_caption = true;
                options.extension.table = true;
                options.extension.header_id_prefix = Some("user-content-".to_string());
            }),
        ] {
            assert_eq!(
                render(markdown, &options),
                comrak_html(markdown, &options),
                "output diverged from comrak"
            );
        }
    }

    /// Catches drift in the copies of comrak's `render_link` and `render_image`.
    ///
    /// The same links and images, carrying the same attributes, render once
    /// through the mirrored code (`unsafe` alone) and once through comrak's own
    /// (`unsafe` and `render.attributes`). Any byte the copies write differently
    /// shows up as a failure, including which conflicting names are skipped.
    #[test]
    fn mirrors_comrak_attributes() {
        let markdown = concat!(
            "[plain](https://example.com) and [titled](https://example.com \"a title\").\n\n",
            "![img](image.png) and ![titled img](image.png \"image title\").\n\n",
            "[empty]() and [dangerous](javascript:alert(1)) and <https://auto.link>.\n\n",
            "![](\"\") and [nested [link](inner) text](outer).\n",
        );
        let matches: fn(&NodeValue) -> bool =
            |value| matches!(value, NodeValue::Link(..) | NodeValue::Image(..));

        for attrs in [
            attributes(None, &[], &[]),
            attributes(
                Some("i"),
                &["a", "b"],
                &[
                    ("HREF", "/x"),
                    ("src", "x.png"),
                    ("Alt", "x"),
                    ("title", "t"),
                    ("data-sourcepos", "9:9-9:9"),
                    ("target", "_blank"),
                ],
            ),
        ] {
            for mirrored in [
                unsafe_options(),
                options(|options| {
                    options.render.r#unsafe = true;
                    options.render.sourcepos = true;
                    options.render.figure_with_caption = true;
                }),
                options(|options| {
                    options.render.r#unsafe = true;
                    options.render.sourcepos = true;
                    options.render.ignore_empty_links = true;
                    options.extension.autolink = true;
                    options.parse.relaxed_autolinks = true;
                }),
            ] {
                let upstream = options(|options| {
                    *options = mirrored.clone();
                    options.render.attributes = true;
                });

                assert_eq!(
                    render_with_attrs(markdown, &mirrored, matches, attrs.clone()),
                    format_with_attrs(
                        markdown,
                        &upstream,
                        matches,
                        &attrs,
                        |root, options, buffer| { comrak::format_html(root, options, buffer) }
                    ),
                    "mirrored render_link/render_image diverged from comrak"
                );
            }
        }
    }
}
