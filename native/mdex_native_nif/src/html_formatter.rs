//! HTML formatter that renders node attributes.
//!
//! comrak parses `{#id .class key=value}` into `Ast::attrs` but leaves rendering
//! to custom formatters (see comrak PR #814), so its own HTML formatter ignores
//! the field. This formatter delegates every node to `format_node_default`
//! except the ones that can carry attributes, where it writes the same markup
//! comrak would, plus the attributes.
//!
//! Attribute pairs are written verbatim rather than `data-` prefixed, so
//! `{target=_blank}` renders as `target="_blank"`. Values are escaped; keys that
//! are not valid HTML attribute names are skipped, since a plugin can put
//! anything in `pairs`.

use comrak::create_formatter;
use comrak::html::{self, ChildRendering, Context};
use comrak::nodes::{Node, NodeCode, NodeLink, NodeValue};
use std::fmt::{self, Write};

create_formatter!(MdexFormatter, {
    NodeValue::Code(ref nc) => |context, node, entering| {
        render_code(context, node, entering, nc)?;
    },
    NodeValue::Image(ref nl) => |context, node, entering| {
        return render_image(context, node, entering, nl);
    },
    NodeValue::Link(ref nl) => |context, node, entering| {
        return render_link(context, node, entering, nl);
    },
});

/// Mirrors comrak's `render_link`, adding attributes to the opening tag.
fn render_link<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
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

/// Mirrors comrak's `render_image`, adding attributes to the `<img>` tag.
///
/// The children of an image render as its `alt` text, which is why entering
/// returns `ChildRendering::Plain` and the tag is closed on exit.
fn render_image<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
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

/// Mirrors comrak's `render_code`, adding attributes to the `<code>` tag.
fn render_code<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nc: &NodeCode,
) -> fmt::Result {
    if entering {
        context.write_str("<code")?;
        html::render_sourcepos(context, node)?;
        write_attrs(context, node)?;
        context.write_str(">")?;
        context.escape(&nc.literal)?;
        context.write_str("</code>")?;
    }

    Ok(())
}

/// Writes `id`, `class` and key/value attributes from the node's `attrs`.
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

    fn attr_options() -> Options<'static> {
        Options {
            extension: Extension {
                header_attributes: true,
                fenced_code_attributes: true,
                inline_code_attributes: true,
                link_attributes: true,
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

    #[test]
    fn renders_parsed_inline_code_attributes() {
        assert_eq!(
            render("`:ok`{.language-elixir}\n", &attr_options()),
            "<p><code class=\"language-elixir\">:ok</code></p>\n"
        );
    }

    /// The case from mdex#412: attributes set on the AST, not parsed from
    /// markdown, with no extension enabled.
    #[test]
    fn renders_programmatic_link_attributes() {
        assert_eq!(
            render_with_attrs(
                "[link](https://example.com)\n",
                &Options::default(),
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
                &Options::default(),
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
                &Options::default(),
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
                &Options::default(),
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
}
