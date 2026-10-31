//! Rendering a Comrak code fence with Lumis.
//!
//! Comrak owns the closing `</code></pre>`, and its adapter contract predates
//! Lumis's formatters, so the output has to be trimmed to match. Everything
//! else here is MDEx decorator policy: which info-string attributes override
//! which formatter option, and what an unset one defaults to.

use std::collections::HashMap;

use lumis_core::events::HighlightEvent;
use lumis_core::formatter::html::{multi_themes_pre_attrs, open_tag, pre_attrs, AttrValue};
use lumis_core::formatter::BudgetExhausted;
use lumis_core::languages::Language;

use crate::types::options::ExBudget;
use lumis_core::elixir::{
    ExAppearance, ExAttrValue, ExFormatterOption, ExHtmlInlineHighlightLines,
    ExHtmlInlineHighlightLinesStyle, ExHtmlLinkedHighlightLines, ExHtmlStructure, ExLineSpec,
    ThemeOrString,
};

pub fn render_code_fence(
    source: &str,
    language: Option<&str>,
    formatter: Option<ExFormatterOption>,
    rainbow_brackets: bool,
    word_diff: bool,
    budget: ExBudget,
    attributes: &HashMap<String, String>,
) -> Result<String, String> {
    // Comrak includes the code-fence terminator's newline in the literal. Its
    // adapter contract historically did not turn that into another visual line.
    let source = source.strip_suffix('\n').unwrap_or(source);
    // A fence without an info string stays plain. Content detection would let a
    // shebang or a doctype inside a bare fence pick a language MDEx never named.
    let language = Language::guess(Some(language.unwrap_or("plaintext")), source);
    let formatter = with_mdex_attributes(formatter.unwrap_or_default(), attributes);
    let formatter = formatter.into_formatter(language)?;

    let (events, exhausted) = highlight(source, language, rainbow_brackets, word_diff, budget);

    let mut output = Vec::new();
    formatter
        .render_budgeted_or(source, &events, &mut output, exhausted)
        .map_err(|error| error.to_string())?;
    let output = String::from_utf8(output).map_err(|error| error.to_string())?;

    output
        .strip_suffix("</code></pre>")
        .map(str::to_owned)
        .ok_or_else(|| "Lumis formatter did not produce HTML code-fence output".to_string())
}

/// Renders inline code as a whole `<code>` element: the token spans of the
/// formatter's inline structure, inside a tag carrying the `lumis` class and the
/// theme's colors merged with `attrs`, as a block's `<pre>` would carry them.
///
/// The formatter goes through the same mapping as a fence, so a terminal or
/// BBCode formatter renders as inline HTML here too. Its `pre_class` and
/// `pre_attrs` belong to blocks and are left out. A language that does not
/// name a known one renders as plain text; nothing is guessed from content.
pub fn render_inline_code(
    source: &str,
    language: &str,
    formatter: Option<ExFormatterOption>,
    rainbow_brackets: bool,
    word_diff: bool,
    budget: ExBudget,
    attrs: &[(String, String)],
) -> Result<String, String> {
    let language = language.parse().unwrap_or(Language::PlainText);
    let mut formatter = with_mdex_attributes(formatter.unwrap_or_default(), &HashMap::new());
    let attrs: Vec<(String, AttrValue)> = attrs
        .iter()
        .map(|(name, value)| (name.clone(), AttrValue::Value(value.clone())))
        .collect();

    let code_attrs = match &mut formatter {
        ExFormatterOption::HtmlInline {
            structure, theme, ..
        } => {
            *structure = ExHtmlStructure::Inline;
            let theme = theme.clone().and_then(|theme| match theme {
                ThemeOrString::Theme(theme) => Some(theme.into()),
                ThemeOrString::String(name) => lumis_core::themes::get(&name).ok(),
            });
            pre_attrs(None, theme.as_ref(), &attrs)
        }
        ExFormatterOption::HtmlMultiThemes {
            structure,
            themes,
            default_theme,
            css_variable_prefix,
            ..
        } => {
            *structure = ExHtmlStructure::Inline;
            let themes = themes
                .iter()
                .map(|(name, theme)| (name.clone(), theme.clone().into()))
                .collect();
            multi_themes_pre_attrs(
                None,
                &themes,
                default_theme.as_deref(),
                css_variable_prefix.as_deref().unwrap_or("--lumis"),
                &attrs,
            )
        }
        ExFormatterOption::HtmlLinked { structure, .. } => {
            *structure = ExHtmlStructure::Inline;
            pre_attrs(None, None, &attrs)
        }
        // `with_mdex_attributes` only returns the HTML formatters.
        _ => pre_attrs(None, None, &attrs),
    };

    let (events, exhausted) = highlight(source, language, rainbow_brackets, word_diff, budget);
    let mut output = Vec::new();
    open_tag(&mut output, "code", &code_attrs).map_err(|error| error.to_string())?;
    formatter
        .into_formatter(language)?
        .render_budgeted_or(source, &events, &mut output, exhausted)
        .map_err(|error| error.to_string())?;
    output.extend_from_slice(b"</code>");

    String::from_utf8(output).map_err(|error| error.to_string())
}

/// Highlight events for `source`, nested the way Lumis renders them.
///
/// Missing parsers, broken queries, and runtime failures cost only this code's
/// highlighting, not the surrounding document: the caller keeps its formatter
/// so it still escapes and styles the plain source.
fn highlight(
    source: &str,
    language: Language,
    rainbow_brackets: bool,
    word_diff: bool,
    budget: ExBudget,
) -> (Vec<HighlightEvent<'static>>, Option<BudgetExhausted>) {
    let plain = || {
        vec![HighlightEvent::Source {
            start: 0,
            end: source.len(),
        }]
    };

    if language == Language::PlainText {
        return (plain(), None);
    }

    let highlighted = crate::lumis_runtime::executor().ok().and_then(|executor| {
        executor
            .highlight(
                source,
                language.id_name(),
                rainbow_brackets,
                word_diff,
                budget,
            )
            .ok()
    });

    match highlighted {
        Some(output) => (output.events, output.budget),
        None => (plain(), None),
    }
}

/// Applies MDEx's info-string decorators over the formatter the caller asked for.
///
/// A decorator wins over the option, an unset option falls back to what MDEx
/// has always rendered, and `header` is dropped because Comrak writes the
/// closing tags and an outer header could never be closed. A code fence is a
/// block, so `structure` is always `:block`: inline output has no `</pre>` for
/// Comrak's closing tags to match.
fn with_mdex_attributes(
    formatter: ExFormatterOption,
    attributes: &HashMap<String, String>,
) -> ExFormatterOption {
    use ExFormatterOption as F;

    match formatter {
        F::HtmlInline {
            structure: _,
            theme,
            pre_class,
            pre_attrs,
            code_attrs,
            italic,
            include_highlights,
            highlight_lines,
            line_numbers,
            header: _,
        } => html_inline_with_attributes(
            InlineFormatter {
                theme,
                pre_class,
                pre_attrs,
                code_attrs,
                italic,
                include_highlights,
                highlight_lines,
                line_numbers,
            },
            attributes,
        ),
        F::HtmlLinked {
            structure: _,
            pre_class,
            pre_attrs,
            code_attrs,
            highlight_lines,
            line_numbers,
            header: _,
        } => F::HtmlLinked {
            structure: ExHtmlStructure::Block,
            pre_class: mdex_attribute(attributes, "pre_class").or(pre_class),
            pre_attrs,
            code_attrs,
            highlight_lines: linked_highlight_lines(attributes).or(highlight_lines),
            line_numbers,
            header: None,
        },
        F::HtmlMultiThemes {
            structure: _,
            themes,
            default_theme,
            css_variable_prefix,
            pre_class,
            pre_attrs,
            code_attrs,
            italic,
            include_highlights,
            highlight_lines,
            line_numbers,
            header: _,
        } => F::HtmlMultiThemes {
            structure: ExHtmlStructure::Block,
            themes,
            default_theme,
            css_variable_prefix,
            pre_class: mdex_attribute(attributes, "pre_class").or(pre_class),
            pre_attrs,
            code_attrs,
            italic,
            include_highlights: include_highlights || attributes.contains_key("include_highlights"),
            highlight_lines: multi_theme_highlight_lines(highlight_lines, attributes),
            line_numbers,
            header: None,
        },
        // A terminal formatter cannot render into a code fence; MDEx has always
        // treated it as the inline HTML one.
        F::Terminal {
            theme,
            line_numbers,
            ..
        } => html_inline_with_attributes(
            InlineFormatter {
                theme,
                line_numbers,
                ..InlineFormatter::default()
            },
            attributes,
        ),
        // BBCode carries its own highlight-line shape, which the inline HTML
        // formatter cannot take. A fence's `highlight_lines` attribute is read
        // back from the decorator instead.
        F::BbcodeScoped { .. } => F::HtmlInline {
            structure: ExHtmlStructure::Block,
            theme: None,
            pre_class: mdex_attribute(attributes, "pre_class"),
            pre_attrs: Vec::new(),
            code_attrs: Vec::new(),
            italic: false,
            include_highlights: attributes.contains_key("include_highlights"),
            highlight_lines: inline_highlight_lines(attributes, None),
            line_numbers: false,
            header: None,
        },
    }
}

/// The inline HTML formatter's settings, as they stand before MDEx's
/// info-string decorators are applied over them.
#[derive(Default)]
struct InlineFormatter {
    theme: Option<ThemeOrString>,
    pre_class: Option<String>,
    pre_attrs: Vec<(String, ExAttrValue)>,
    code_attrs: Vec<(String, ExAttrValue)>,
    italic: bool,
    include_highlights: bool,
    highlight_lines: Option<ExHtmlInlineHighlightLines>,
    line_numbers: bool,
}

/// `header` is always dropped: Comrak owns the closing tags, so an outer header
/// could never be closed by its syntax-highlighter adapter contract.
fn html_inline_with_attributes(
    formatter: InlineFormatter,
    attributes: &HashMap<String, String>,
) -> ExFormatterOption {
    let theme = attributes
        .get("theme")
        .map(|name| ThemeOrString::String(name.clone()))
        .or(formatter.theme)
        .or_else(|| Some(ThemeOrString::String("onedark".to_string())));

    ExFormatterOption::HtmlInline {
        structure: ExHtmlStructure::Block,
        pre_class: mdex_attribute(attributes, "pre_class").or(formatter.pre_class),
        pre_attrs: formatter.pre_attrs,
        code_attrs: formatter.code_attrs,
        italic: formatter.italic,
        include_highlights: formatter.include_highlights
            || attributes.contains_key("include_highlights"),
        highlight_lines: inline_highlight_lines(attributes, Some(line_background(&theme)))
            .or(formatter.highlight_lines),
        line_numbers: formatter.line_numbers,
        theme,
        header: None,
    }
}

/// A multi-theme block resolves `style: :theme` against the decorator's theme,
/// which the formatter itself cannot see.
fn multi_theme_highlight_lines(
    highlight_lines: Option<ExHtmlInlineHighlightLines>,
    attributes: &HashMap<String, String>,
) -> Option<ExHtmlInlineHighlightLines> {
    let background = line_background_from_name(attributes.get("theme"));

    let mut lines =
        inline_highlight_lines(attributes, Some(background.clone())).or(highlight_lines)?;

    // `:theme` resolves against the one theme a formatter renders with, and a
    // multi-theme block has no such theme. Both the decorator's own
    // `highlight_lines_style="theme"` and an inherited one land here.
    if matches!(lines.style, Some(ExHtmlInlineHighlightLinesStyle::Theme)) {
        lines.style = Some(ExHtmlInlineHighlightLinesStyle::Style { style: background });
    }

    Some(lines)
}

fn inline_highlight_lines(
    attributes: &HashMap<String, String>,
    default_style: Option<String>,
) -> Option<ExHtmlInlineHighlightLines> {
    let lines = parse_highlight_lines(attributes.get("highlight_lines")?)?;
    let style = attributes
        .get("highlight_lines_style")
        .map(|style| match style.as_str() {
            "theme" => ExHtmlInlineHighlightLinesStyle::Theme,
            style => ExHtmlInlineHighlightLinesStyle::Style {
                style: style.to_string(),
            },
        })
        .or_else(|| default_style.map(|style| ExHtmlInlineHighlightLinesStyle::Style { style }));

    Some(ExHtmlInlineHighlightLines {
        lines,
        style,
        class: mdex_attribute(attributes, "highlight_lines_class"),
    })
}

/// Values go through verbatim. `lumis-core` escapes every attribute it writes,
/// so escaping here too would render a quote as `&amp;quot;`.
fn mdex_attribute(attributes: &HashMap<String, String>, name: &str) -> Option<String> {
    attributes.get(name).cloned()
}

fn line_background(theme: &Option<ThemeOrString>) -> String {
    let is_light = match theme {
        Some(ThemeOrString::Theme(theme)) => theme.appearance == ExAppearance::Light,
        Some(ThemeOrString::String(name)) => lumis_core::themes::get(name)
            .map(|theme| theme.appearance == lumis_core::themes::Appearance::Light)
            .unwrap_or_else(|_| name.to_lowercase().contains("light")),
        None => false,
    };

    if is_light {
        "background-color: #e7eaf0;".to_string()
    } else {
        "background-color: #3b4252;".to_string()
    }
}

fn line_background_from_name(theme: Option<&String>) -> String {
    let theme = theme.map(|name| ThemeOrString::String(name.clone()));
    line_background(&theme)
}

fn linked_highlight_lines(
    attributes: &HashMap<String, String>,
) -> Option<ExHtmlLinkedHighlightLines> {
    Some(ExHtmlLinkedHighlightLines {
        lines: parse_highlight_lines(attributes.get("highlight_lines")?)?,
        class: mdex_attribute(attributes, "highlight_lines_class")
            .unwrap_or_else(|| "highlighted".to_string()),
    })
}

fn parse_highlight_lines(spec: &str) -> Option<Vec<ExLineSpec>> {
    let mut lines = Vec::new();

    for part in spec
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        // A part that does not parse is dropped on its own. One typo in a
        // decorator must not silently turn off highlighting for the whole fence.
        if let Some((start, end)) = part.split_once('-') {
            let (Ok(start), Ok(end)) = (start.trim().parse(), end.trim().parse()) else {
                continue;
            };
            if start == 0 || start > end {
                continue;
            }
            lines.push(ExLineSpec::Range {
                start,
                end,
                step: 1,
            });
        } else if let Ok(line) = part.parse() {
            if line > 0 {
                lines.push(ExLineSpec::Single(line));
            }
        }
    }

    Some(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(
        source: &str,
        language: Option<&str>,
        formatter: Option<ExFormatterOption>,
        attributes: &[(&str, &str)],
    ) -> String {
        let rainbow_brackets = false;
        let attributes = attributes
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        render_code_fence(
            source,
            language,
            formatter,
            rainbow_brackets,
            false,
            ExBudget::default(),
            &attributes,
        )
        .unwrap()
    }

    #[test]
    fn parses_single_lines_and_ranges() {
        let lines = parse_highlight_lines("1, 3-5, 0, 7-6").unwrap();
        assert_eq!(lines.len(), 2);
        assert!(matches!(lines[0], ExLineSpec::Single(1)));
        assert!(matches!(
            lines[1],
            ExLineSpec::Range {
                start: 3,
                end: 5,
                step: 1
            }
        ));
    }

    #[test]
    fn keeps_parsable_parts_when_one_part_is_malformed() {
        let lines = parse_highlight_lines("1, oops, 3-5, 7-x").unwrap();
        assert_eq!(lines.len(), 2);
        assert!(matches!(lines[0], ExLineSpec::Single(1)));
        assert!(matches!(
            lines[1],
            ExLineSpec::Range {
                start: 3,
                end: 5,
                step: 1
            }
        ));
    }

    #[test]
    fn omits_the_closing_tags_comrak_writes_itself() {
        let html = render("hello\n", Some("plaintext"), None, &[]);
        assert!(html.starts_with("<pre"), "{html}");
        assert!(!html.contains("</code></pre>"), "{html}");
    }

    #[test]
    fn does_not_render_the_code_fence_terminator_as_a_line() {
        let one_line = render("hello\n", Some("plaintext"), None, &[]);
        assert_eq!(one_line.matches("data-line=").count(), 1, "{one_line}");
    }

    #[test]
    fn a_fence_without_an_info_string_stays_plain() {
        let html = render("#!/usr/bin/env python\nx = 1\n", None, None, &[]);
        assert!(html.contains("language-plaintext"), "{html}");
        assert!(!html.contains("language-python"), "{html}");
    }

    #[test]
    fn decorator_attributes_override_the_formatter() {
        let html = render(
            "hello\n",
            Some("plaintext"),
            None,
            &[("pre_class", "custom-class"), ("highlight_lines", "1")],
        );

        assert!(html.contains("custom-class"), "{html}");
        assert!(html.contains("background-color: #3b4252;"), "{html}");
    }

    #[test]
    fn a_light_theme_decorator_picks_the_light_line_background() {
        let html = render(
            "hello\n",
            Some("plaintext"),
            None,
            &[("theme", "github_light"), ("highlight_lines", "1")],
        );

        assert!(html.contains("background-color: #e7eaf0;"), "{html}");
    }

    #[test]
    fn escapes_decorator_attributes_in_the_rendered_html() {
        let injection = "x\" onmouseover=\"alert(1)";
        let html = render(
            "hello\n",
            Some("plaintext"),
            None,
            &[
                ("pre_class", injection),
                ("highlight_lines", "1"),
                ("highlight_lines_class", injection),
                ("highlight_lines_style", injection),
            ],
        );

        assert!(!html.contains("onmouseover=\"alert(1)"), "{html}");
        assert!(html.contains("&quot;"), "{html}");
    }

    #[test]
    fn preserves_decorator_attributes_when_rendering_unsafe() {
        let html = render(
            "hello\n",
            Some("plaintext"),
            None,
            &[
                ("highlight_lines", "1"),
                ("highlight_lines_style", "color: red;"),
            ],
        );

        assert!(html.contains("color: red;"), "{html}");
    }

    #[test]
    fn a_linked_formatter_defaults_the_highlighted_line_class() {
        let formatter = ExFormatterOption::HtmlLinked {
            structure: ExHtmlStructure::Block,
            pre_class: None,
            pre_attrs: Vec::new(),
            code_attrs: Vec::new(),
            highlight_lines: None,
            line_numbers: false,
            header: None,
        };
        let html = render(
            "hello\n",
            Some("plaintext"),
            Some(formatter),
            &[("highlight_lines", "1")],
        );

        assert!(html.contains("highlighted"), "{html}");
    }

    #[test]
    fn an_out_of_range_highlight_line_highlights_nothing() {
        let html = render(
            "hello\n",
            Some("plaintext"),
            None,
            &[("highlight_lines", "5-9")],
        );

        assert!(!html.contains("background-color: #3b4252;"), "{html}");
    }

    #[test]
    fn a_header_never_survives_the_bridge() {
        // Comrak writes the closing tags, so an outer header could never be closed.
        let html = render(
            "hello\n",
            Some("plaintext"),
            None,
            &[("pre_class", "plain")],
        );

        assert!(html.starts_with("<pre"), "{html}");
    }

    #[test]
    fn a_multi_theme_block_resolves_a_decorator_theme_style() {
        let attributes = HashMap::from([
            ("theme".to_string(), "github_light".to_string()),
            ("highlight_lines".to_string(), "1".to_string()),
            ("highlight_lines_style".to_string(), "theme".to_string()),
        ]);

        let lines = multi_theme_highlight_lines(None, &attributes).unwrap();

        // `:theme` has no meaning to a multi-theme formatter, so the decorator's
        // own theme has to be resolved into a colour here.
        assert!(matches!(
            lines.style,
            Some(ExHtmlInlineHighlightLinesStyle::Style { .. })
        ));
    }

    #[test]
    fn passes_decorator_attributes_through_for_the_formatter_to_escape() {
        // `lumis-core` escapes every attribute value it writes. Escaping here
        // as well rendered a quote as `&amp;quot;`.
        let attributes = HashMap::from([
            ("highlight_lines".to_string(), "1".to_string()),
            (
                "highlight_lines_style".to_string(),
                "color: red;\" onmouseover=\"alert(1)".to_string(),
            ),
            (
                "highlight_lines_class".to_string(),
                "line\" onmouseover=\"alert(1)".to_string(),
            ),
        ]);

        let inline = inline_highlight_lines(&attributes, None).unwrap();
        assert!(matches!(
            inline.style,
            Some(ExHtmlInlineHighlightLinesStyle::Style { style })
                if style == attributes["highlight_lines_style"]
        ));
        assert_eq!(
            inline.class.as_deref(),
            Some(attributes["highlight_lines_class"].as_str())
        );

        let linked = linked_highlight_lines(&attributes).unwrap();
        assert_eq!(linked.class, attributes["highlight_lines_class"]);
    }
}
