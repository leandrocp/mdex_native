//! Rendering a Comrak code fence with Lumis.
//!
//! Comrak owns the closing `</code></pre>`, and its adapter contract predates
//! Lumis's formatters, so the output has to be trimmed to match. Everything
//! else here is MDEx decorator policy: which info-string attributes override
//! which formatter option, and what an unset one defaults to.

use std::collections::HashMap;

use lumis_core::events::HighlightEvent;
use lumis_core::formatter::html;
use lumis_core::languages::Language;
use lumis_wasm_runtime::RuntimeError;

use lumis_core::elixir::{
    ExAppearance, ExFormatterOption, ExHtmlInlineHighlightLines, ExHtmlInlineHighlightLinesStyle,
    ExHtmlLinkedHighlightLines, ExLineSpec, ThemeOrString,
};

pub fn render_code_fence(
    source: &str,
    language: Option<&str>,
    formatter: Option<ExFormatterOption>,
    attributes: &HashMap<String, String>,
    render_unsafe: bool,
) -> Result<String, String> {
    // Comrak includes the code-fence terminator's newline in the literal. Its
    // adapter contract historically did not turn that into another visual line.
    let source = source.strip_suffix('\n').unwrap_or(source);
    // A fence without an info string stays plain. Content detection would let a
    // shebang or a doctype inside a bare fence pick a language MDEx never named.
    let language = Language::guess(Some(language.unwrap_or("plaintext")), source);
    let formatter = with_mdex_attributes(formatter.unwrap_or_default(), attributes, render_unsafe);
    let (formatter, rainbow_brackets) = formatter.into_formatter(language)?;

    let events = if language == Language::PlainText {
        vec![HighlightEvent::Source {
            start: 0,
            end: source.len(),
        }]
    } else {
        let executor = crate::lumis_runtime::executor().map_err(|reason| format!("{reason:#}"))?;
        match executor.highlight(source, language.id_name(), rainbow_brackets) {
            Ok(events) => flatten_events(source, events),
            Err(RuntimeError::LanguageNotLoaded(language)) => {
                return Err(format!("language {language} is not loaded"));
            }
            Err(runtime_error) => return Err(runtime_error.to_string()),
        }
    };

    let mut output = Vec::new();
    formatter
        .render(source, &events, &mut output)
        .map_err(|error| error.to_string())?;
    let output = String::from_utf8(output).map_err(|error| error.to_string())?;

    output
        .strip_suffix("</code></pre>")
        .map(str::to_owned)
        .ok_or_else(|| "Lumis formatter did not produce HTML code-fence output".to_string())
}

/// Applies MDEx's info-string decorators over the formatter the caller asked for.
///
/// A decorator wins over the option, an unset option falls back to what MDEx
/// has always rendered, and `header` is dropped because Comrak writes the
/// closing tags and an outer header could never be closed.
fn with_mdex_attributes(
    formatter: ExFormatterOption,
    attributes: &HashMap<String, String>,
    render_unsafe: bool,
) -> ExFormatterOption {
    use ExFormatterOption as F;

    match formatter {
        F::HtmlInline {
            theme,
            pre_class,
            italic,
            include_highlights,
            rainbow_brackets,
            highlight_lines,
            header: _,
        } => html_inline_with_attributes(
            theme,
            pre_class,
            italic,
            include_highlights,
            rainbow_brackets,
            highlight_lines,
            attributes,
            render_unsafe,
        ),
        F::HtmlLinked {
            pre_class,
            rainbow_brackets,
            highlight_lines,
            header: _,
        } => F::HtmlLinked {
            pre_class: mdex_attribute(attributes, "pre_class", render_unsafe).or(pre_class),
            rainbow_brackets,
            highlight_lines: linked_highlight_lines(attributes, render_unsafe).or(highlight_lines),
            header: None,
        },
        F::HtmlMultiThemes {
            themes,
            default_theme,
            css_variable_prefix,
            pre_class,
            italic,
            include_highlights,
            rainbow_brackets,
            highlight_lines,
            header: _,
        } => F::HtmlMultiThemes {
            themes,
            default_theme,
            css_variable_prefix,
            pre_class: mdex_attribute(attributes, "pre_class", render_unsafe).or(pre_class),
            italic,
            include_highlights: include_highlights || attributes.contains_key("include_highlights"),
            rainbow_brackets,
            highlight_lines: multi_theme_highlight_lines(
                highlight_lines,
                attributes,
                render_unsafe,
            ),
            header: None,
        },
        // A terminal formatter cannot render into a code fence; MDEx has always
        // treated it as the inline HTML one.
        F::Terminal {
            theme,
            rainbow_brackets,
            ..
        } => html_inline_with_attributes(
            theme,
            None,
            false,
            false,
            rainbow_brackets,
            None,
            attributes,
            render_unsafe,
        ),
        F::BbcodeScoped { rainbow_brackets } => F::HtmlInline {
            theme: None,
            pre_class: mdex_attribute(attributes, "pre_class", render_unsafe),
            italic: false,
            include_highlights: attributes.contains_key("include_highlights"),
            rainbow_brackets,
            highlight_lines: inline_highlight_lines(attributes, None, render_unsafe),
            header: None,
        },
    }
}

/// `header` is always dropped: Comrak owns the closing tags, so an outer header
/// could never be closed by its syntax-highlighter adapter contract.
#[allow(clippy::too_many_arguments)]
fn html_inline_with_attributes(
    theme: Option<ThemeOrString>,
    pre_class: Option<String>,
    italic: bool,
    include_highlights: bool,
    rainbow_brackets: bool,
    highlight_lines: Option<ExHtmlInlineHighlightLines>,
    attributes: &HashMap<String, String>,
    render_unsafe: bool,
) -> ExFormatterOption {
    let theme = attributes
        .get("theme")
        .map(|name| ThemeOrString::String(name.clone()))
        .or(theme)
        .or_else(|| Some(ThemeOrString::String("onedark".to_string())));

    ExFormatterOption::HtmlInline {
        pre_class: mdex_attribute(attributes, "pre_class", render_unsafe).or(pre_class),
        italic,
        include_highlights: include_highlights || attributes.contains_key("include_highlights"),
        rainbow_brackets,
        highlight_lines: inline_highlight_lines(
            attributes,
            Some(line_background(&theme)),
            render_unsafe,
        )
        .or(highlight_lines),
        theme,
        header: None,
    }
}

/// A multi-theme block resolves `style: :theme` against the decorator's theme,
/// which the formatter itself cannot see.
fn multi_theme_highlight_lines(
    highlight_lines: Option<ExHtmlInlineHighlightLines>,
    attributes: &HashMap<String, String>,
    render_unsafe: bool,
) -> Option<ExHtmlInlineHighlightLines> {
    if let Some(lines) = inline_highlight_lines(
        attributes,
        Some(line_background_from_name(attributes.get("theme"))),
        render_unsafe,
    ) {
        return Some(lines);
    }

    let mut highlight_lines = highlight_lines?;
    if matches!(
        highlight_lines.style,
        Some(ExHtmlInlineHighlightLinesStyle::Theme)
    ) {
        highlight_lines.style = Some(ExHtmlInlineHighlightLinesStyle::Style {
            style: line_background_from_name(attributes.get("theme")),
        });
    }

    Some(highlight_lines)
}

fn inline_highlight_lines(
    attributes: &HashMap<String, String>,
    default_style: Option<String>,
    render_unsafe: bool,
) -> Option<ExHtmlInlineHighlightLines> {
    let lines = parse_highlight_lines(attributes.get("highlight_lines")?)?;
    let style = attributes
        .get("highlight_lines_style")
        .map(|style| match style.as_str() {
            "theme" => ExHtmlInlineHighlightLinesStyle::Theme,
            style => ExHtmlInlineHighlightLinesStyle::Style {
                style: escape_mdex_attribute(style, render_unsafe),
            },
        })
        .or_else(|| default_style.map(|style| ExHtmlInlineHighlightLinesStyle::Style { style }));

    Some(ExHtmlInlineHighlightLines {
        lines,
        style,
        class: mdex_attribute(attributes, "highlight_lines_class", render_unsafe),
    })
}

fn mdex_attribute(
    attributes: &HashMap<String, String>,
    name: &str,
    render_unsafe: bool,
) -> Option<String> {
    attributes
        .get(name)
        .map(|value| escape_mdex_attribute(value, render_unsafe))
}

fn escape_mdex_attribute(value: &str, render_unsafe: bool) -> String {
    if render_unsafe {
        value.to_string()
    } else {
        html::escape(value)
    }
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

/// Collapses nested scopes so each span carries exactly one, and leaves
/// whitespace-only spans unscoped.
///
/// Lumis's formatters nest: an inner scope renders inside its outer one. MDEx
/// has always published the flat form, and `mdex_test.exs` pins it, so a fence
/// deliberately renders differently from `Lumis.highlight/2` for languages with
/// injected or nested scopes. Per-character colour is the same either way; only
/// the element structure differs, and CSS written against the flat form would
/// break without this.
fn flatten_events(source: &str, events: Vec<HighlightEvent>) -> Vec<HighlightEvent> {
    let mut flattened = Vec::with_capacity(events.len());
    let mut scopes = Vec::new();

    for event in events {
        match event {
            HighlightEvent::Start {
                scope_index,
                language,
            } => scopes.push((scope_index, language)),
            HighlightEvent::End => {
                scopes.pop();
            }
            HighlightEvent::Source { start, end } => {
                let text = source.get(start..end).unwrap_or_default();
                if text.trim().is_empty() || scopes.is_empty() {
                    flattened.push(HighlightEvent::Source { start, end });
                } else if let Some((scope_index, language)) = scopes.last() {
                    flattened.push(HighlightEvent::Start {
                        scope_index: *scope_index,
                        language: language.clone(),
                    });
                    flattened.push(HighlightEvent::Source { start, end });
                    flattened.push(HighlightEvent::End);
                }
            }
        }
    }

    flattened
}

fn linked_highlight_lines(
    attributes: &HashMap<String, String>,
    render_unsafe: bool,
) -> Option<ExHtmlLinkedHighlightLines> {
    Some(ExHtmlLinkedHighlightLines {
        lines: parse_highlight_lines(attributes.get("highlight_lines")?)?,
        class: mdex_attribute(attributes, "highlight_lines_class", render_unsafe)
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
            lines.push(ExLineSpec::Range { start, end });
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
        render_unsafe: bool,
    ) -> String {
        let attributes = attributes
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        render_code_fence(source, language, formatter, &attributes, render_unsafe).unwrap()
    }

    #[test]
    fn parses_single_lines_and_ranges() {
        let lines = parse_highlight_lines("1, 3-5, 0, 7-6").unwrap();
        assert_eq!(lines.len(), 2);
        assert!(matches!(lines[0], ExLineSpec::Single(1)));
        assert!(matches!(lines[1], ExLineSpec::Range { start: 3, end: 5 }));
    }

    #[test]
    fn flattens_nested_scopes_to_the_innermost_non_whitespace_scope() {
        let events = vec![
            HighlightEvent::Start {
                scope_index: 1,
                language: "elixir".to_string(),
            },
            HighlightEvent::Source { start: 0, end: 1 },
            HighlightEvent::Start {
                scope_index: 2,
                language: "elixir".to_string(),
            },
            HighlightEvent::Source { start: 1, end: 2 },
            HighlightEvent::End,
            HighlightEvent::Source { start: 2, end: 3 },
            HighlightEvent::End,
        ];

        assert_eq!(
            flatten_events("a b", events),
            vec![
                HighlightEvent::Start {
                    scope_index: 1,
                    language: "elixir".to_string(),
                },
                HighlightEvent::Source { start: 0, end: 1 },
                HighlightEvent::End,
                HighlightEvent::Source { start: 1, end: 2 },
                HighlightEvent::Start {
                    scope_index: 1,
                    language: "elixir".to_string(),
                },
                HighlightEvent::Source { start: 2, end: 3 },
                HighlightEvent::End,
            ]
        );
    }

    #[test]
    fn keeps_parsable_parts_when_one_part_is_malformed() {
        let lines = parse_highlight_lines("1, oops, 3-5, 7-x").unwrap();
        assert_eq!(lines.len(), 2);
        assert!(matches!(lines[0], ExLineSpec::Single(1)));
        assert!(matches!(lines[1], ExLineSpec::Range { start: 3, end: 5 }));
    }

    #[test]
    fn omits_the_closing_tags_comrak_writes_itself() {
        let html = render("hello\n", Some("plaintext"), None, &[], false);
        assert!(html.starts_with("<pre"), "{html}");
        assert!(!html.contains("</code></pre>"), "{html}");
    }

    #[test]
    fn does_not_render_the_code_fence_terminator_as_a_line() {
        let one_line = render("hello\n", Some("plaintext"), None, &[], false);
        assert_eq!(one_line.matches("data-line=").count(), 1, "{one_line}");
    }

    #[test]
    fn a_fence_without_an_info_string_stays_plain() {
        let html = render("#!/usr/bin/env python\nx = 1\n", None, None, &[], false);
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
            false,
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
            false,
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
            false,
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
            true,
        );

        assert!(html.contains("color: red;"), "{html}");
    }

    #[test]
    fn a_linked_formatter_defaults_the_highlighted_line_class() {
        let formatter = ExFormatterOption::HtmlLinked {
            pre_class: None,
            rainbow_brackets: false,
            highlight_lines: None,
            header: None,
        };
        let html = render(
            "hello\n",
            Some("plaintext"),
            Some(formatter),
            &[("highlight_lines", "1")],
            false,
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
            false,
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
            false,
        );

        assert!(html.starts_with("<pre"), "{html}");
    }

    #[test]
    fn escapes_decorator_attributes_in_safe_rendering() {
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

        let inline = inline_highlight_lines(&attributes, None, false).unwrap();
        assert!(matches!(
            inline.style,
            Some(ExHtmlInlineHighlightLinesStyle::Style { style })
                if style == html::escape(&attributes["highlight_lines_style"])
        ));
        assert_eq!(
            inline.class,
            Some(html::escape(&attributes["highlight_lines_class"]))
        );

        let linked = linked_highlight_lines(&attributes, false).unwrap();
        assert_eq!(
            linked.class,
            html::escape(&attributes["highlight_lines_class"])
        );
    }

    #[test]
    fn preserves_decorator_attributes_in_unsafe_rendering() {
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

        let inline = inline_highlight_lines(&attributes, None, true).unwrap();
        assert!(matches!(
            inline.style,
            Some(ExHtmlInlineHighlightLinesStyle::Style { style })
                if style == attributes["highlight_lines_style"]
        ));
        assert_eq!(
            inline.class.as_deref(),
            Some(attributes["highlight_lines_class"].as_str())
        );

        let linked = linked_highlight_lines(&attributes, true).unwrap();
        assert_eq!(linked.class, attributes["highlight_lines_class"]);
    }
}
