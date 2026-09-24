//! The Lumis option types as Elixir spells them.
//!
//! `lumis-core` carries the formatters but not their BEAM wire format: it has
//! no `rustler` dependency, so every runtime that speaks to the BEAM declares
//! the terms itself. These mirror `lumis_nif`'s definitions field for field —
//! the `:lumis` application encodes the terms this decodes, so the two have to
//! agree.

#[cfg(feature = "lumis")]
mod lumis_types {
    use lumis_core::formatter::{
        bbcode, html::AttrValue, html::SteppedLineRange, html_inline, html_linked, terminal,
        BBCodeScopedBuilder, Formatter, HtmlElement, HtmlInlineBuilder, HtmlLinkedBuilder,
        HtmlMultiThemesBuilder, TerminalBackground, TerminalBuilder,
    };
    use lumis_core::{languages::Language, themes};
    use rustler::{NifStruct, NifTaggedEnum, NifUnitEnum, NifUntaggedEnum};
    use std::collections::HashMap;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, NifUnitEnum)]
    pub enum ExAppearance {
        Light,
        #[default]
        Dark,
    }

    /// An attribute value as Elixir spells it: a string, or `true`/`false` for
    /// the boolean form and for removing one of Lumis's own attributes.
    ///
    /// `bool` comes first because Rustler tries the variants in order and
    /// `true` is an atom, not a binary.
    #[derive(Clone, Debug, NifUntaggedEnum)]
    pub enum ExAttrValue {
        Flag(bool),
        Value(String),
    }

    impl From<ExAttrValue> for AttrValue {
        fn from(value: ExAttrValue) -> Self {
            match value {
                ExAttrValue::Flag(flag) => Self::from(flag),
                ExAttrValue::Value(value) => Self::Value(value),
            }
        }
    }

    /// Decode the attribute pairs an Elixir keyword list arrives as.
    fn attr_values(attrs: Vec<(String, ExAttrValue)>) -> lumis_core::formatter::html::HtmlAttrs {
        attrs
            .into_iter()
            .map(|(name, value)| (name, value.into()))
            .collect()
    }

    #[derive(Clone, Debug, NifTaggedEnum)]
    pub enum ExFormatterOption {
        HtmlInline {
            theme: Option<ThemeOrString>,
            pre_class: Option<String>,
            pre_attrs: Vec<(String, ExAttrValue)>,
            code_attrs: Vec<(String, ExAttrValue)>,
            italic: bool,
            include_highlights: bool,
            highlight_lines: Option<ExHtmlInlineHighlightLines>,
            line_numbers: bool,
            header: Option<ExHtmlElement>,
        },
        HtmlLinked {
            pre_class: Option<String>,
            pre_attrs: Vec<(String, ExAttrValue)>,
            code_attrs: Vec<(String, ExAttrValue)>,
            highlight_lines: Option<ExHtmlLinkedHighlightLines>,
            line_numbers: bool,
            header: Option<ExHtmlElement>,
        },
        HtmlMultiThemes {
            themes: HashMap<String, ExTheme>,
            default_theme: Option<String>,
            css_variable_prefix: Option<String>,
            pre_class: Option<String>,
            pre_attrs: Vec<(String, ExAttrValue)>,
            code_attrs: Vec<(String, ExAttrValue)>,
            italic: bool,
            include_highlights: bool,
            highlight_lines: Option<ExHtmlInlineHighlightLines>,
            line_numbers: bool,
            header: Option<ExHtmlElement>,
        },
        Terminal {
            theme: Option<ThemeOrString>,
            background: Option<ExTerminalBackground>,
            width: Option<usize>,
            highlight_lines: Option<ExTerminalHighlightLines>,
            line_numbers: bool,
        },
        BbcodeScoped {
            highlight_lines: Option<ExBBCodeHighlightLines>,
        },
    }

    #[derive(Clone, Debug, NifTaggedEnum)]
    pub enum ExTerminalBackground {
        Theme,
        String(String),
    }

    impl Default for ExFormatterOption {
        fn default() -> Self {
            Self::HtmlInline {
                theme: None,
                pre_class: None,
                pre_attrs: Vec::new(),
                code_attrs: Vec::new(),
                italic: false,
                include_highlights: false,
                highlight_lines: None,
                line_numbers: false,
                header: None,
            }
        }
    }

    #[derive(Clone, Debug, NifTaggedEnum)]
    pub enum ThemeOrString {
        Theme(ExTheme),
        String(String),
    }

    impl Default for ThemeOrString {
        fn default() -> Self {
            Self::String("onedark".to_string())
        }
    }

    fn resolve_theme(theme_or_string: ThemeOrString) -> Option<themes::Theme> {
        match theme_or_string {
            ThemeOrString::Theme(theme) => Some(theme.into()),
            ThemeOrString::String(name) => themes::get(&name).ok(),
        }
    }

    #[inline]
    fn convert_line_specs(
        lines: Vec<ExLineSpec>,
    ) -> (Vec<std::ops::RangeInclusive<usize>>, Vec<SteppedLineRange>) {
        let mut contiguous = Vec::new();
        let mut stepped = Vec::new();

        for line in lines {
            match line {
                ExLineSpec::Single(line) => contiguous.push(line..=line),
                ExLineSpec::Range {
                    start,
                    end,
                    step: 1,
                } if start <= end => contiguous.push(start..=end),
                ExLineSpec::Range {
                    start,
                    end,
                    step: -1,
                } if start >= end => contiguous.push(end..=start),
                line @ ExLineSpec::Range { .. } => {
                    if let Some(range) = line.to_stepped_line_range() {
                        stepped.push(range);
                    }
                }
            }
        }

        (contiguous, stepped)
    }

    #[inline]
    fn convert_inline_style(
        style: ExHtmlInlineHighlightLinesStyle,
    ) -> html_inline::HighlightLinesStyle {
        match style {
            ExHtmlInlineHighlightLinesStyle::Theme => html_inline::HighlightLinesStyle::Theme,
            ExHtmlInlineHighlightLinesStyle::Style { style } => {
                html_inline::HighlightLinesStyle::Style(style)
            }
        }
    }

    fn convert_inline_highlight_lines(
        highlight: ExHtmlInlineHighlightLines,
    ) -> (html_inline::HighlightLines, Vec<SteppedLineRange>) {
        let (lines, stepped) = convert_line_specs(highlight.lines);
        let highlight = html_inline::HighlightLines {
            lines,
            style: highlight.style.map(convert_inline_style),
            class: highlight.class,
        };
        (highlight, stepped)
    }

    fn convert_terminal_highlight_lines(
        highlight: ExTerminalHighlightLines,
    ) -> (terminal::HighlightLines, Vec<SteppedLineRange>) {
        let (lines, stepped) = convert_line_specs(highlight.lines);
        let highlight = terminal::HighlightLines {
            lines,
            background: highlight.background,
        };
        (highlight, stepped)
    }

    fn convert_bbcode_highlight_lines(
        highlight: ExBBCodeHighlightLines,
    ) -> (bbcode::HighlightLines, Vec<SteppedLineRange>) {
        let (lines, stepped) = convert_line_specs(highlight.lines);
        (bbcode::HighlightLines { lines }, stepped)
    }

    fn convert_linked_highlight_lines(
        highlight: ExHtmlLinkedHighlightLines,
    ) -> (html_linked::HighlightLines, Vec<SteppedLineRange>) {
        let (lines, stepped) = convert_line_specs(highlight.lines);
        let highlight = html_linked::HighlightLines {
            lines,
            class: highlight.class,
        };
        (highlight, stepped)
    }

    fn split_highlight_lines<T>(
        converted: Option<(T, Vec<SteppedLineRange>)>,
    ) -> (Option<T>, Vec<SteppedLineRange>) {
        converted.map_or((None, Vec::new()), |(highlight, stepped)| {
            (Some(highlight), stepped)
        })
    }

    impl ExFormatterOption {
        pub fn into_formatter<T>(
            self,
            language: Language,
        ) -> Result<Box<dyn Formatter<T>>, String> {
            match self {
                ExFormatterOption::HtmlInline {
                    theme,
                    pre_class,
                    pre_attrs,
                    code_attrs,
                    italic,
                    include_highlights,
                    highlight_lines,
                    line_numbers,
                    header,
                } => {
                    let theme = theme.and_then(resolve_theme);

                    let (highlight_lines, stepped_highlight_lines) =
                        split_highlight_lines(highlight_lines.map(convert_inline_highlight_lines));

                    let mut formatter = HtmlInlineBuilder::new()
                        .language(language)
                        .theme(theme)
                        .pre_class(pre_class)
                        .pre_attrs(attr_values(pre_attrs))
                        .code_attrs(attr_values(code_attrs))
                        .italic(italic)
                        .include_highlights(include_highlights)
                        .highlight_lines(highlight_lines)
                        .line_numbers(line_numbers)
                        .header(header.map(HtmlElement::from))
                        .build()
                        .map_err(|error| format!("HtmlInline builder error: {error:?}"))?;
                    formatter.set_stepped_highlight_lines(stepped_highlight_lines);

                    Ok(Box::new(formatter))
                }
                ExFormatterOption::HtmlLinked {
                    pre_class,
                    pre_attrs,
                    code_attrs,
                    highlight_lines,
                    line_numbers,
                    header,
                } => {
                    let (highlight_lines, stepped_highlight_lines) =
                        split_highlight_lines(highlight_lines.map(convert_linked_highlight_lines));

                    let mut formatter = HtmlLinkedBuilder::new()
                        .language(language)
                        .pre_class(pre_class)
                        .pre_attrs(attr_values(pre_attrs))
                        .code_attrs(attr_values(code_attrs))
                        .highlight_lines(highlight_lines)
                        .line_numbers(line_numbers)
                        .header(header.map(HtmlElement::from))
                        .build()
                        .map_err(|error| format!("HtmlLinked builder error: {error:?}"))?;
                    formatter.set_stepped_highlight_lines(stepped_highlight_lines);

                    Ok(Box::new(formatter))
                }
                ExFormatterOption::HtmlMultiThemes {
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
                    header,
                } => {
                    let themes_map: HashMap<String, themes::Theme> = themes
                        .into_iter()
                        .map(|(name, theme)| (name, theme.into()))
                        .collect();

                    let (highlight_lines, stepped_highlight_lines) =
                        split_highlight_lines(highlight_lines.map(convert_inline_highlight_lines));

                    let mut builder = HtmlMultiThemesBuilder::new();
                    builder
                        .language(language)
                        .themes(themes_map)
                        .css_variable_prefix(css_variable_prefix.as_deref().unwrap_or("--lumis"))
                        .pre_class(pre_class)
                        .pre_attrs(attr_values(pre_attrs))
                        .code_attrs(attr_values(code_attrs))
                        .italic(italic)
                        .include_highlights(include_highlights)
                        .highlight_lines(highlight_lines)
                        .line_numbers(line_numbers)
                        .header(header.map(HtmlElement::from));

                    if let Some(default_theme) = default_theme {
                        builder.default_theme(default_theme);
                    }

                    let mut formatter = builder
                        .build()
                        .map_err(|error| format!("HtmlMultiThemes builder error: {error:?}"))?;
                    formatter.set_stepped_highlight_lines(stepped_highlight_lines);

                    Ok(Box::new(formatter))
                }
                ExFormatterOption::Terminal {
                    theme,
                    background,
                    width,
                    highlight_lines,
                    line_numbers,
                } => {
                    let theme = theme.and_then(resolve_theme);
                    let background = match background {
                        Some(ExTerminalBackground::Theme) => TerminalBackground::Theme,
                        Some(ExTerminalBackground::String(color)) => {
                            TerminalBackground::Color(color)
                        }
                        None => TerminalBackground::Inherit,
                    };
                    let (highlight_lines, stepped_highlight_lines) = split_highlight_lines(
                        highlight_lines.map(convert_terminal_highlight_lines),
                    );

                    let mut formatter = TerminalBuilder::new()
                        .language(language)
                        .theme(theme)
                        .background(background)
                        .width(width)
                        .highlight_lines(highlight_lines)
                        .line_numbers(line_numbers)
                        .build()
                        .map_err(|error| format!("Terminal builder error: {error:?}"))?;
                    formatter.set_stepped_highlight_lines(stepped_highlight_lines);

                    Ok(Box::new(formatter))
                }
                ExFormatterOption::BbcodeScoped { highlight_lines } => {
                    let (highlight_lines, stepped_highlight_lines) =
                        split_highlight_lines(highlight_lines.map(convert_bbcode_highlight_lines));

                    let mut formatter = BBCodeScopedBuilder::new()
                        .language(language)
                        .highlight_lines(highlight_lines)
                        .build()
                        .map_err(|error| format!("BBCode scoped builder error: {error:?}"))?;
                    formatter.set_stepped_highlight_lines(stepped_highlight_lines);

                    Ok(Box::new(formatter))
                }
            }
        }
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.Theme"]
    pub struct ExTheme {
        pub name: String,
        pub appearance: ExAppearance,
        pub revision: String,
        pub highlights: HashMap<String, ExStyle>,
    }

    impl From<ExTheme> for themes::Theme {
        fn from(theme: ExTheme) -> Self {
            let appearance = match theme.appearance {
                ExAppearance::Light => themes::Appearance::Light,
                ExAppearance::Dark => themes::Appearance::Dark,
            };
            themes::Theme {
                name: theme.name,
                appearance,
                revision: theme.revision,
                highlights: theme
                    .highlights
                    .into_iter()
                    .map(|(name, style)| (name, style.into()))
                    .collect(),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, NifUnitEnum)]
    pub enum ExUnderlineStyle {
        Solid,
        Wavy,
        Double,
        Dotted,
        Dashed,
    }

    impl ExUnderlineStyle {
        fn to_theme(style: Option<Self>) -> themes::UnderlineStyle {
            match style {
                None => themes::UnderlineStyle::None,
                Some(ExUnderlineStyle::Solid) => themes::UnderlineStyle::Solid,
                Some(ExUnderlineStyle::Wavy) => themes::UnderlineStyle::Wavy,
                Some(ExUnderlineStyle::Double) => themes::UnderlineStyle::Double,
                Some(ExUnderlineStyle::Dotted) => themes::UnderlineStyle::Dotted,
                Some(ExUnderlineStyle::Dashed) => themes::UnderlineStyle::Dashed,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, NifStruct)]
    #[module = "Lumis.Theme.TextDecoration"]
    pub struct ExTextDecoration {
        pub underline: Option<ExUnderlineStyle>,
        pub strikethrough: bool,
    }

    impl From<ExTextDecoration> for themes::TextDecoration {
        fn from(decoration: ExTextDecoration) -> Self {
            themes::TextDecoration {
                underline: ExUnderlineStyle::to_theme(decoration.underline),
                strikethrough: decoration.strikethrough,
            }
        }
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.Theme.Style"]
    pub struct ExStyle {
        pub fg: Option<String>,
        pub bg: Option<String>,
        pub bold: bool,
        pub italic: bool,
        pub text_decoration: ExTextDecoration,
    }

    impl From<ExStyle> for themes::Style {
        fn from(style: ExStyle) -> Self {
            themes::Style {
                fg: style.fg,
                bg: style.bg,
                bold: style.bold,
                italic: style.italic,
                text_decoration: style.text_decoration.into(),
            }
        }
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.HTMLElement"]
    pub struct ExHtmlElement {
        pub open_tag: String,
        pub close_tag: String,
    }

    impl From<ExHtmlElement> for HtmlElement {
        fn from(element: ExHtmlElement) -> Self {
            HtmlElement {
                open_tag: element.open_tag,
                close_tag: element.close_tag,
            }
        }
    }

    #[derive(Clone, Debug, NifTaggedEnum)]
    pub enum ExLineSpec {
        Single(usize),
        Range {
            start: usize,
            end: usize,
            step: isize,
        },
    }

    impl ExLineSpec {
        fn to_stepped_line_range(&self) -> Option<SteppedLineRange> {
            match self {
                ExLineSpec::Single(line) => SteppedLineRange::new(*line, *line, 1),
                ExLineSpec::Range { start, end, step } => {
                    SteppedLineRange::new(*start, *end, *step)
                }
            }
        }
    }

    #[derive(Clone, Debug, Default, NifTaggedEnum)]
    pub enum ExHtmlInlineHighlightLinesStyle {
        #[default]
        Theme,
        Style {
            style: String,
        },
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.HTMLInlineHighlightLines"]
    pub struct ExHtmlInlineHighlightLines {
        pub lines: Vec<ExLineSpec>,
        pub style: Option<ExHtmlInlineHighlightLinesStyle>,
        pub class: Option<String>,
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.HTMLLinkedHighlightLines"]
    pub struct ExHtmlLinkedHighlightLines {
        pub lines: Vec<ExLineSpec>,
        pub class: String,
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.TerminalHighlightLines"]
    pub struct ExTerminalHighlightLines {
        pub lines: Vec<ExLineSpec>,
        pub background: Option<String>,
    }

    #[derive(Clone, Debug, Default, NifStruct)]
    #[module = "Lumis.BBCodeHighlightLines"]
    pub struct ExBBCodeHighlightLines {
        pub lines: Vec<ExLineSpec>,
    }

    #[cfg(test)]
    mod tests {
        use super::{convert_line_specs, ExLineSpec};

        #[test]
        fn partitions_contiguous_and_genuinely_stepped_line_specs() {
            let (contiguous, stepped) = convert_line_specs(vec![
                ExLineSpec::Single(2),
                ExLineSpec::Range {
                    start: 3,
                    end: 5,
                    step: 1,
                },
                ExLineSpec::Range {
                    start: 9,
                    end: 7,
                    step: -1,
                },
                ExLineSpec::Range {
                    start: 1,
                    end: 9,
                    step: 2,
                },
            ]);

            assert_eq!(contiguous, [2..=2, 3..=5, 7..=9]);
            assert_eq!(stepped.len(), 1);
            assert!(stepped[0].contains(7));
            assert!(!stepped[0].contains(8));
        }
    }
}

#[cfg(feature = "lumis")]
pub use lumis_types::*;
