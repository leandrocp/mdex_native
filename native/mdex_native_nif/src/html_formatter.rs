//! HTML formatter that renders node attributes.
//!
//! comrak parses `{#id .class key=value}` into `Ast::attrs` but leaves rendering
//! to custom formatters (see comrak PR #814), so its own HTML formatter ignores
//! the field. This formatter delegates every node to `format_node_default`
//! except `Link` and `Image`, where it writes the same markup comrak would,
//! plus the attributes.
//!
//! Attribute pairs are written verbatim rather than `data-` prefixed, so
//! `{target=_blank}` renders as `target="_blank"`.
//!
//! Attributes are document content that can execute, in the same class as raw
//! HTML and `javascript:` links, so they follow the rule comrak applies to
//! those: nothing is written unless `render.unsafe` is set. Values are escaped
//! and keys that are not valid HTML attribute names are skipped even then,
//! since comrak never emits structurally broken markup.
//!
//! # Where the mirrored code comes from
//!
//! `render_link` and `render_image` below are comrak's own, with a call to
//! `write_attrs` added before the opening tag is closed. comrak keeps them
//! private, so they cannot be called or wrapped. Upstream at comrak 0.55.0,
//! which is the version `Cargo.toml` pins:
//!
//! - [`render_link`](https://github.com/kivikakk/comrak/blob/6fbe87fafde3953a9f3bc582804318593d703805/src/html.rs#L818-L852)
//! - [`render_image`](https://github.com/kivikakk/comrak/blob/6fbe87fafde3953a9f3bc582804318593d703805/src/html.rs#L743-L783)
//!
//! Both are byte-for-byte identical on `kivikakk/comrak@main` as of
//! 2026-09-25. When comrak is upgraded, re-read those two functions and mirror
//! any change here. `mirrors_comrak_for_empty_attrs` is what catches drift: it
//! renders links and images carrying empty attributes, so this code runs and
//! adds nothing, and compares the result against `comrak::format_html`.

use comrak::create_formatter;
use comrak::html::{self, ChildRendering, Context};
use comrak::nodes::{Node, NodeLink, NodeValue};
use std::fmt::{self, Write};

create_formatter!(MdexFormatter, {
    NodeValue::Image(ref nl) => |context, node, entering| {
        return render_image(context, node, entering, nl);
    },
    NodeValue::Link(ref nl) => |context, node, entering| {
        return render_link(context, node, entering, nl);
    },
});

/// Whether this node has attributes that are going to be written.
///
/// Attributes are document content that can execute, in the same class as raw
/// HTML and `javascript:` links, so they follow the rule comrak applies to
/// those: nothing is written unless `render.unsafe` is set.
///
/// When this is false there is nothing to add, and rendering is comrak's to do.
fn writes_attrs<T>(context: &Context<T>, node: Node<'_>) -> bool {
    context.options.render.r#unsafe && node.data().attrs.is_some()
}

/// Mirrors [comrak's `render_link`][upstream], adding attributes to the opening
/// tag.
///
/// Nodes without attributes go to `format_node_default`, the same function the
/// generated formatter falls back to for every node type not handled here, so
/// comrak keeps ownership of link rendering unless there is something to add.
///
/// [upstream]: https://github.com/kivikakk/comrak/blob/6fbe87fafde3953a9f3bc582804318593d703805/src/html.rs#L818-L852
fn render_link<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if !writes_attrs(context, node) {
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
        write_attrs(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</a>")?;
    }

    Ok(ChildRendering::HTML)
}

/// Mirrors [comrak's `render_image`][upstream], adding attributes to the `<img>`
/// tag.
///
/// The children of an image render as its `alt` text, which is why entering
/// returns `ChildRendering::Plain` and the tag is closed on exit. As with
/// `render_link`, nodes without attributes are left to comrak.
///
/// [upstream]: https://github.com/kivikakk/comrak/blob/6fbe87fafde3953a9f3bc582804318593d703805/src/html.rs#L743-L783
fn render_image<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if !writes_attrs(context, node) {
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
    write_attrs(context, node)?;
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

/// Writes `id`, `class` and key/value attributes from the node's `attrs`.
///
/// Only called when `writes_attrs` holds. Name/value pairs are written the way
/// comrak's `write_opening_tag` writes them — ` name="escaped value"` — which is
/// not reusable directly because it also emits the tag name and the closing `>`,
/// and an `href` needs `escape_href` rather than `escape`.
fn write_attrs<T>(context: &mut Context<T>, node: Node<'_>) -> fmt::Result {
    let ast = node.data();
    let Some(attrs) = ast.attrs.as_deref() else {
        return Ok(());
    };

    if let Some(id) = &attrs.id {
        context.write_str(" id=\"")?;
        context.escape(id)?;
        context.write_str("\"")?;
    }

    if !attrs.classes.is_empty() {
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
        if !is_attribute_name(key) {
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

/// Whether `name` is safe to write as an HTML attribute name.
///
/// The parser only produces names made of these characters, but `pairs` is a
/// plain list on the Elixir side, so a key like `x" onclick="alert(1)` would
/// otherwise escape the tag.
fn is_attribute_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.'))
        && name.starts_with(|c: char| c.is_ascii_alphabetic() || matches!(c, '_' | ':'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::nodes::{Attributes, NodeValue};
    use comrak::options::{Extension, Plugins, Render};
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

    /// Renders `markdown`, applying `attrs` to every node matching `matches`,
    /// which is how a plugin reaches the field from Elixir.
    fn render_with_attrs(
        markdown: &str,
        options: &Options,
        matches: fn(&NodeValue) -> bool,
        attrs: Attributes,
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
        MdexFormatter::format_document_with_plugins(
            root,
            options,
            &mut buffer,
            &Plugins::default(),
        )
        .unwrap();
        buffer
    }

    /// Attribute extensions on, `unsafe` on, which is what it takes for
    /// attributes to reach the output.
    fn attr_options() -> Options<'static> {
        Options {
            extension: Extension {
                header_attributes: true,
                fenced_code_attributes: true,
                inline_code_attributes: true,
                link_attributes: true,
                ..Default::default()
            },
            render: Render {
                r#unsafe: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// `unsafe` on, no extensions: the plugin case, where attributes are set on
    /// the AST rather than parsed.
    fn unsafe_options() -> Options<'static> {
        Options {
            render: Render {
                r#unsafe: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn attributes(id: Option<&str>, classes: &[&str], pairs: &[(&str, &str)]) -> Attributes {
        Attributes {
            id: id.map(str::to_string),
            classes: classes.iter().map(|c| c.to_string()).collect(),
            pairs: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
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

    /// Only Link and Image are handled, so inline code keeps comrak's output
    /// even with `inline_code_attributes` and `unsafe` on.
    #[test]
    fn leaves_inline_code_to_comrak() {
        assert_eq!(
            render("`:ok`{.language-elixir}\n", &attr_options()),
            "<p><code>:ok</code></p>\n"
        );
    }

    /// Without `render.unsafe` attributes are dropped, the way comrak drops raw
    /// HTML and `javascript:` hrefs. This is what keeps `{onclick=alert(1)}` out
    /// of the default output.
    #[test]
    fn omits_attributes_unless_unsafe() {
        let safe_options = Options {
            extension: attr_options().extension,
            ..Default::default()
        };
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

    #[test]
    fn skips_attribute_names_that_are_not_valid() {
        assert_eq!(
            render_with_attrs(
                "[link](https://example.com)\n",
                &unsafe_options(),
                |value| matches!(value, NodeValue::Link(..)),
                attributes(
                    None,
                    &[],
                    &[
                        ("x\" onclick=\"alert(1)", "y"),
                        ("", "empty"),
                        ("2bad", "leading digit"),
                        ("data-ok", "kept"),
                    ],
                ),
            ),
            "<p><a href=\"https://example.com\" data-ok=\"kept\">link</a></p>\n"
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
            Options {
                render: Render {
                    r#unsafe: true,
                    sourcepos: true,
                    figure_with_caption: true,
                    ..Default::default()
                },
                extension: Extension {
                    table: true,
                    header_id_prefix: Some("user-content-".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
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
    /// Links and images carry empty attributes here, so `writes_attrs` holds and
    /// the mirrored code runs, but it has nothing to add — meaning any byte it
    /// writes differently from comrak shows up as a failure. comrak ignores
    /// `attrs` entirely, so its output is the reference.
    #[test]
    fn mirrors_comrak_for_empty_attrs() {
        let markdown = concat!(
            "[plain](https://example.com) and [titled](https://example.com \"a title\").\n\n",
            "![img](image.png) and ![titled img](image.png \"image title\").\n\n",
            "[empty]() and [dangerous](javascript:alert(1)) and <https://auto.link>.\n\n",
            "![](\"\") and [nested [link](inner) text](outer).\n",
        );

        for options in [
            unsafe_options(),
            Options {
                render: Render {
                    r#unsafe: true,
                    sourcepos: true,
                    figure_with_caption: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            Options {
                render: Render {
                    sourcepos: true,
                    ignore_empty_links: true,
                    ..Default::default()
                },
                extension: Extension {
                    autolink: true,
                    ..Default::default()
                },
                parse: comrak::options::Parse {
                    relaxed_autolinks: true,
                    ..Default::default()
                },
            },
        ] {
            let with_empty_attrs = render_with_attrs(
                markdown,
                &options,
                |value| matches!(value, NodeValue::Link(..) | NodeValue::Image(..)),
                attributes(None, &[], &[]),
            );

            assert_eq!(
                with_empty_attrs,
                comrak_html(markdown, &options),
                "mirrored render_link/render_image diverged from comrak"
            );
        }
    }
}
