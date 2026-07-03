use crate::types::elixir_types::{ExFormatterOption, ThemeOrString};
use comrak::adapters::SyntaxHighlighterAdapter;
use lumis::highlight::highlight_iter;
use lumis::html;
use lumis::languages::Language;
use lumis::themes::{self, Appearance};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::io;
use std::sync::Arc;

struct FmtToIoAdapter<'a, W: Write + ?Sized>(&'a mut W);

impl<'a, W: Write + ?Sized> io::Write for FmtToIoAdapter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.0
            .write_str(s)
            .map_err(|_| io::Error::other("write error"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct MultiThemesConfig {
    themes: HashMap<String, themes::Theme>,
    default_theme: Option<String>,
    css_variable_prefix: String,
    italic: bool,
    include_highlights: bool,
}

pub struct LumisAdapter {
    formatter_config: ExFormatterOption,
    render_unsafe: bool,
    stored_attrs: Mutex<Option<Arc<HashMap<String, String>>>>,
    stored_lang: Mutex<Option<Language>>,
}

impl Default for LumisAdapter {
    fn default() -> Self {
        Self {
            formatter_config: ExFormatterOption::default(),
            render_unsafe: false,
            stored_attrs: Mutex::new(None),
            stored_lang: Mutex::new(None),
        }
    }
}

impl LumisAdapter {
    pub fn new(formatter_config: ExFormatterOption, render_unsafe: bool) -> Self {
        Self {
            formatter_config,
            render_unsafe,
            stored_attrs: Mutex::new(None),
            stored_lang: Mutex::new(None),
        }
    }

    fn parse_custom_attributes(info_string: &str) -> Option<HashMap<String, String>> {
        let tokens = shlex::split(info_string)?;
        if tokens.is_empty() {
            return None;
        }

        let mut attributes = HashMap::with_capacity(tokens.len());
        for token in tokens {
            if let Some((key, value)) = token.split_once('=') {
                attributes.insert(key.trim().to_string(), value.to_string());
            } else {
                // Handle standalone flags (e.g., "include_highlights" without "=true")
                attributes.insert(token.trim().to_string(), "true".to_string());
            }
        }

        if attributes.is_empty() {
            None
        } else {
            Some(attributes)
        }
    }

    fn theme_from_formatter(&self) -> Option<themes::Theme> {
        match &self.formatter_config {
            ExFormatterOption::HtmlInline { theme, .. } => theme.as_ref().and_then(|t| match t {
                ThemeOrString::Theme(ex_theme) => Some(ex_theme.clone().into()),
                ThemeOrString::String(name) => themes::get(name).ok(),
            }),
            ExFormatterOption::Terminal { theme } => theme.as_ref().and_then(|t| match t {
                ThemeOrString::Theme(ex_theme) => Some(ex_theme.clone().into()),
                ThemeOrString::String(name) => themes::get(name).ok(),
            }),
            _ => None,
        }
    }

    fn multi_themes_config(&self) -> Option<MultiThemesConfig> {
        match &self.formatter_config {
            ExFormatterOption::HtmlMultiThemes {
                themes,
                default_theme,
                css_variable_prefix,
                italic,
                include_highlights,
                ..
            } => {
                let themes_map: HashMap<String, themes::Theme> = themes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect();
                Some(MultiThemesConfig {
                    themes: themes_map,
                    default_theme: default_theme.clone(),
                    css_variable_prefix: css_variable_prefix
                        .clone()
                        .unwrap_or_else(|| "--lumis".to_string()),
                    italic: *italic,
                    include_highlights: *include_highlights,
                })
            }
            _ => None,
        }
    }

    fn pre_class(&self) -> Option<&str> {
        match &self.formatter_config {
            ExFormatterOption::HtmlInline { pre_class, .. } => pre_class.as_deref(),
            ExFormatterOption::HtmlLinked { pre_class, .. } => pre_class.as_deref(),
            ExFormatterOption::HtmlMultiThemes { pre_class, .. } => pre_class.as_deref(),
            _ => None,
        }
    }

    fn italic_enabled(&self) -> bool {
        match &self.formatter_config {
            ExFormatterOption::HtmlInline { italic, .. } => *italic,
            ExFormatterOption::HtmlMultiThemes { italic, .. } => *italic,
            _ => false,
        }
    }

    fn should_include_highlights(&self) -> bool {
        let stored_attrs = self.stored_attrs.lock();
        if let Some(attrs) = stored_attrs.as_ref() {
            if attrs.contains_key("include_highlights") {
                return true;
            }
        }

        match &self.formatter_config {
            ExFormatterOption::HtmlInline {
                include_highlights, ..
            } => *include_highlights,
            ExFormatterOption::HtmlMultiThemes {
                include_highlights, ..
            } => *include_highlights,
            _ => false,
        }
    }

    fn decorator_theme(&self) -> Option<themes::Theme> {
        let stored_attrs = self.stored_attrs.lock();
        if let Some(attrs) = stored_attrs.as_ref() {
            if let Some(theme_name) = attrs.get("theme") {
                return themes::get(theme_name).ok();
            }
        }
        None
    }

    fn decorator_pre_class(&self) -> Option<String> {
        let stored_attrs = self.stored_attrs.lock();
        if let Some(attrs) = stored_attrs.as_ref() {
            if let Some(pre_class) = attrs.get("pre_class") {
                return Some(pre_class.clone());
            }
        }
        None
    }

    fn parse_highlight_lines(line_spec: &str, max_line: usize) -> HashSet<usize> {
        let mut lines = HashSet::new();
        for part in line_spec.split(',') {
            let part = part.trim();
            if let Some((start, end)) = part.split_once('-') {
                if let (Ok(start_line), Ok(end_line)) =
                    (start.trim().parse::<usize>(), end.trim().parse::<usize>())
                {
                    if start_line <= end_line && (1..=max_line).contains(&start_line) {
                        for line in start_line.max(1)..=end_line.min(max_line) {
                            lines.insert(line);
                        }
                    }
                }
            } else if let Ok(line) = part.parse::<usize>() {
                if (1..=max_line).contains(&line) {
                    lines.insert(line);
                }
            }
        }
        lines
    }

    fn highlight_lines_config(
        &self,
        theme: &Option<themes::Theme>,
        max_line: usize,
    ) -> Option<(HashSet<usize>, Option<String>, Option<String>)> {
        let stored_attrs = self.stored_attrs.lock();
        if let Some(attrs) = stored_attrs.as_ref() {
            if let Some(lines_spec) = attrs.get("highlight_lines") {
                let lines = Self::parse_highlight_lines(lines_spec, max_line);
                let is_linked =
                    matches!(self.formatter_config, ExFormatterOption::HtmlLinked { .. });
                let class = attrs.get("highlight_lines_class").cloned().or_else(|| {
                    if is_linked {
                        Some("highlighted".to_string())
                    } else {
                        None
                    }
                });
                let style = attrs
                    .get("highlight_lines_style")
                    .and_then(|s| {
                        if s == "theme" {
                            theme.as_ref().and_then(|t| {
                                t.get_style("highlighted").map(|style| style.css(true, " "))
                            })
                        } else {
                            Some(s.clone())
                        }
                    })
                    .or_else(|| {
                        if is_linked {
                            None
                        } else {
                            theme.as_ref().map(|t| {
                                let is_light = matches!(t.appearance, Appearance::Light)
                                    || t.name.to_lowercase().contains("light");
                                let highlight_bg = if is_light { "#e7eaf0" } else { "#3b4252" };
                                format!("background-color: {};", highlight_bg)
                            })
                        }
                    });
                return Some((lines, style, class));
            }
        }
        drop(stored_attrs);

        self.formatter_highlight_lines_config(theme, max_line)
    }

    fn formatter_highlight_lines_config(
        &self,
        theme: &Option<themes::Theme>,
        max_line: usize,
    ) -> Option<(HashSet<usize>, Option<String>, Option<String>)> {
        match &self.formatter_config {
            ExFormatterOption::HtmlInline {
                highlight_lines: Some(hl),
                ..
            }
            | ExFormatterOption::HtmlMultiThemes {
                highlight_lines: Some(hl),
                ..
            } => {
                let lines = Self::convert_line_specs(&hl.lines, max_line);
                let style = hl.style.as_ref().map(|s| match s {
                    crate::types::elixir_types::ExHtmlInlineHighlightLinesStyle::Theme => theme
                        .as_ref()
                        .and_then(|t| t.get_style("highlighted").map(|style| style.css(true, " ")))
                        .unwrap_or_else(|| {
                            let is_light = theme
                                .as_ref()
                                .map(|t| {
                                    matches!(t.appearance, Appearance::Light)
                                        || t.name.to_lowercase().contains("light")
                                })
                                .unwrap_or(false);
                            let highlight_bg = if is_light { "#e7eaf0" } else { "#3b4252" };
                            format!("background-color: {};", highlight_bg)
                        }),
                    crate::types::elixir_types::ExHtmlInlineHighlightLinesStyle::Style {
                        style,
                    } => style.clone(),
                });
                let class = hl.class.clone();
                Some((lines, style, class))
            }
            ExFormatterOption::HtmlLinked {
                highlight_lines: Some(hl),
                ..
            } => {
                let lines = Self::convert_line_specs(&hl.lines, max_line);
                let class = Some(hl.class.clone());
                Some((lines, None, class))
            }
            _ => None,
        }
    }

    fn convert_line_specs(
        lines: &[crate::types::elixir_types::ExLineSpec],
        max_line: usize,
    ) -> HashSet<usize> {
        let mut result = HashSet::new();
        for spec in lines {
            match spec {
                crate::types::elixir_types::ExLineSpec::Single(n) => {
                    if (1..=max_line).contains(n) {
                        result.insert(*n);
                    }
                }
                crate::types::elixir_types::ExLineSpec::Range { start, end } => {
                    if start <= end && (1..=max_line).contains(start) {
                        for n in (*start).max(1)..=(*end).min(max_line) {
                            result.insert(n);
                        }
                    }
                }
            }
        }
        result
    }

    fn header_config(&self) -> Option<(&str, &str)> {
        match &self.formatter_config {
            ExFormatterOption::HtmlInline {
                header: Some(header),
                ..
            }
            | ExFormatterOption::HtmlLinked {
                header: Some(header),
                ..
            }
            | ExFormatterOption::HtmlMultiThemes {
                header: Some(header),
                ..
            } => Some((&header.open_tag, &header.close_tag)),
            _ => None,
        }
    }

    fn custom_attrs(
        attributes: &HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> Option<HashMap<String, String>> {
        attributes
            .get("data-meta")
            .and_then(|info| Self::parse_custom_attributes(info.as_ref()))
    }

    fn language_from_attrs(
        attributes: &HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> Language {
        if let Some(lang) = attributes.get("lang") {
            Language::guess(Some(lang.as_ref()), "")
        } else if let Some(class) = attributes.get("class") {
            let language = class.strip_prefix("language-").unwrap_or("plaintext");
            Language::guess(Some(language), "")
        } else {
            Language::guess(Some("plaintext"), "")
        }
    }
}

impl SyntaxHighlighterAdapter for LumisAdapter {
    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'s, str>>,
    ) -> std::fmt::Result {
        let custom_attrs = Self::custom_attrs(&attributes);
        let lang = attributes.get("lang").map(|l| Language::guess(Some(l), ""));

        if let Some(attrs) = custom_attrs {
            *self.stored_attrs.lock() = Some(Arc::new(attrs));
        }
        if let Some(language) = lang {
            *self.stored_lang.lock() = Some(language);
        }

        let theme = self
            .decorator_theme()
            .or_else(|| self.theme_from_formatter());
        let _header = self.header_config();
        let pre_class = self
            .decorator_pre_class()
            .or_else(|| self.pre_class().map(|s| s.to_string()));
        let escaped_pre_class = pre_class.as_deref().map(|class| {
            if self.render_unsafe {
                class.to_string()
            } else {
                v_htmlescape::escape_fmt(class).to_string()
            }
        });

        let mut adapter = FmtToIoAdapter(output);
        html::open_pre_tag(&mut adapter, escaped_pre_class.as_deref(), theme.as_ref())
            .map_err(|_| std::fmt::Error)
    }

    fn write_code_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'s, str>>,
    ) -> std::fmt::Result {
        let custom_attrs = Self::custom_attrs(&attributes);
        let lang = Self::language_from_attrs(&attributes);

        if let Some(attrs) = custom_attrs {
            *self.stored_attrs.lock() = Some(Arc::new(attrs));
        }
        if !attributes.is_empty() {
            *self.stored_lang.lock() = Some(lang);
        }

        let stored_lang = *self.stored_lang.lock();
        let effective_lang = stored_lang.unwrap_or(lang);

        let mut adapter = FmtToIoAdapter(output);
        html::open_code_tag(&mut adapter, &effective_lang).map_err(|_| std::fmt::Error)
    }

    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        source: &str,
    ) -> std::fmt::Result {
        let stored_lang = *self.stored_lang.lock();

        let language = if let Some(stored_lang) = stored_lang {
            stored_lang
        } else if let Some(lang_str) = lang {
            Language::guess(Some(lang_str), source)
        } else {
            Language::guess(Some("plaintext"), source)
        };
        let is_plaintext = language == Language::PlainText;

        let theme = self
            .decorator_theme()
            .or_else(|| self.theme_from_formatter());
        let is_linked = matches!(self.formatter_config, ExFormatterOption::HtmlLinked { .. });
        let is_multi_themes = matches!(
            self.formatter_config,
            ExFormatterOption::HtmlMultiThemes { .. }
        );
        let include_highlights = self.should_include_highlights();
        let italic = self.italic_enabled();

        let multi_themes_config = self.multi_themes_config();

        let mut html_output = String::new();
        let mut last_end = 0usize;

        highlight_iter(
            source,
            language,
            theme.clone(),
            |text, language, range, scope, _style| {
                let language = Some(language);
                if range.start > last_end {
                    let gap = &source[last_end..range.start];
                    html_output.push_str(gap);
                }

                if text.trim().is_empty() {
                    html_output.push_str(text);
                } else {
                    let span = if is_linked && !is_plaintext {
                        html::span_linked(text, scope)
                    } else if is_multi_themes {
                        if let Some(ref config) = multi_themes_config {
                            html::span_multi_themes(
                                text,
                                scope,
                                language,
                                &config.themes,
                                config.default_theme.as_deref(),
                                &config.css_variable_prefix,
                                config.italic,
                                config.include_highlights,
                            )
                        } else {
                            html::span_inline(
                                text,
                                language,
                                scope,
                                theme.as_ref(),
                                italic,
                                include_highlights,
                            )
                        }
                    } else {
                        html::span_inline(
                            text,
                            language,
                            scope,
                            theme.as_ref(),
                            italic,
                            include_highlights,
                        )
                    };
                    html_output.push_str(&span);
                }
                last_end = range.end;
                Ok::<(), std::fmt::Error>(())
            },
        )
        .map_err(|_| std::fmt::Error)?;

        if last_end < source.len() {
            let remaining = &source[last_end..];
            html_output.push_str(remaining);
        }

        let line_count = html_output.lines().count();
        let highlight_config = self.highlight_lines_config(&theme, line_count);

        for (i, line) in html_output.lines().enumerate() {
            let line_number = i + 1;
            let mut class_name = String::from("l-line");
            let mut style_attr = String::new();

            if let Some((ref lines, ref style, ref class)) = highlight_config {
                if lines.contains(&line_number) {
                    if let Some(ref c) = class {
                        class_name.push(' ');
                        if self.render_unsafe {
                            class_name.push_str(c);
                        } else {
                            class_name.push_str(&v_htmlescape::escape_fmt(c).to_string());
                        }
                    }
                    if let Some(ref s) = style {
                        if self.render_unsafe {
                            style_attr = format!(" style=\"{}\"", s);
                        } else {
                            style_attr = format!(" style=\"{}\"", v_htmlescape::escape_fmt(s));
                        }
                    }
                }
            }

            write!(
                output,
                "<div class=\"{}\"{} data-line=\"{}\">{}\n</div>",
                class_name, style_attr, line_number, line
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use pretty_assertions::assert_str_eq;

    use super::*;
    use comrak::options::Plugins;
    use comrak::{format_html_with_plugins, parse_document, Arena, Options};

    fn run_test(markdown: &str, formatter: ExFormatterOption, options: Options) -> String {
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &options);
        let adapter = LumisAdapter::new(formatter, options.render.r#unsafe);

        let plugins = Plugins {
            render: comrak::options::RenderPlugins {
                codefence_syntax_highlighter: Some(&adapter),
                ..Default::default()
            },
        };

        let mut html = String::new();
        format_html_with_plugins(root, &options, &mut html, &plugins)
            .expect("Failed to format HTML with plugins");

        html
    }

    #[test]
    fn test_default_formatter_option() {
        let markdown = r#"
```rust
fn main() {
    let message = "Hello, world!";
}
```
"#;

        let output = run_test(markdown, ExFormatterOption::default(), Options::default());

        let expected = r#"<pre class="lumis"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1">fn main() {
</div><div class="l-line" data-line="2">    let message = &quot;Hello, world!&quot;;
</div><div class="l-line" data-line="3">}
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_plaintext() {
        let markdown = r#"
```
plain
text
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let output = run_test(markdown, formatter, Options::default());

        let expected = r#"<pre class="lumis"><code class="language-plaintext" translate="no" tabindex="0"><div class="l-line" data-line="1">plain
</div><div class="l-line" data-line="2">text
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_no_attrs() {
        let markdown = r#"
```rust
fn main() {
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let output = run_test(markdown, formatter, Options::default());

        let expected = r#"<pre class="lumis"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1">fn main() {
</div><div class="l-line" data-line="2">    let message = &quot;Hello, world!&quot;;
</div><div class="l-line" data-line="3">}
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_all_attrs() {
        let markdown = r#"
```rust
fn main() {
    let a = 1;
    let b = 2;
    let sum = a + b;
}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: Some(ThemeOrString::String("nord".to_string())),
            pre_class: Some("custom-pre-class".to_string()),
            italic: true,
            include_highlights: true,
            highlight_lines: None,
            header: None,
        };

        let output = run_test(markdown, formatter, Options::default());

        let expected = r#"<pre class="lumis custom-pre-class" style="color: #d8dee9; background-color: #2e3440;"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1"><span data-highlight="keyword.function" style="color: #88c0d0; font-style: italic;">fn</span> <span data-highlight="function" style="color: #88c0d0; font-style: italic;">main</span><span data-highlight="punctuation.bracket" style="color: #88c0d0;">(</span><span data-highlight="punctuation.bracket" style="color: #88c0d0;">)</span> <span data-highlight="punctuation.bracket" style="color: #88c0d0;">{</span>
</div><div class="l-line" data-line="2">    <span data-highlight="keyword" style="color: #81a1c1; font-style: italic;">let</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">a</span> <span data-highlight="operator" style="color: #81a1c1;">=</span> <span data-highlight="number" style="color: #b48ead;">1</span><span data-highlight="punctuation.delimiter" style="color: #88c0d0;">;</span>
</div><div class="l-line" data-line="3">    <span data-highlight="keyword" style="color: #81a1c1; font-style: italic;">let</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">b</span> <span data-highlight="operator" style="color: #81a1c1;">=</span> <span data-highlight="number" style="color: #b48ead;">2</span><span data-highlight="punctuation.delimiter" style="color: #88c0d0;">;</span>
</div><div class="l-line" data-line="4">    <span data-highlight="keyword" style="color: #81a1c1; font-style: italic;">let</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">sum</span> <span data-highlight="operator" style="color: #81a1c1;">=</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">a</span> <span data-highlight="operator" style="color: #81a1c1;">+</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">b</span><span data-highlight="punctuation.delimiter" style="color: #88c0d0;">;</span>
</div><div class="l-line" data-line="5"><span data-highlight="punctuation.bracket" style="color: #88c0d0;">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_decorators() {
        let markdown = r#"
```rust pre_class="my-custom-pre extra-class" theme=github_light include_highlights highlight_lines="1,3-5" highlight_lines_style="background-color: #ffffcc; border-left: 3px solid #ff0000" highlight_lines_class="custom-highlight-class"
fn main() {
    let x = 1;
    let y = 2;
    let z = 3;
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: Some(ThemeOrString::String("nord".to_string())),
            pre_class: Some("default-pre-class".to_string()),
            italic: true,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(markdown, formatter, options);

        let expected = r#"<pre class="lumis my-custom-pre extra-class" style="color: #1f2328; background-color: #ffffff;"><code class="language-rust" translate="no" tabindex="0"><div class="l-line custom-highlight-class" style="background-color: #ffffcc; border-left: 3px solid #ff0000" data-line="1"><span data-highlight="keyword.function" style="color: #cf222e;">fn</span> <span data-highlight="function" style="color: #6639ba;">main</span><span data-highlight="punctuation.bracket" style="color: #1f2328;">(</span><span data-highlight="punctuation.bracket" style="color: #1f2328;">)</span> <span data-highlight="punctuation.bracket" style="color: #1f2328;">{</span>
</div><div class="l-line" data-line="2">    <span data-highlight="keyword" style="color: #cf222e;">let</span> <span data-highlight="variable" style="color: #1f2328;">x</span> <span data-highlight="operator" style="color: #0550ae;">=</span> <span data-highlight="number" style="color: #0550ae;">1</span><span data-highlight="punctuation.delimiter" style="color: #1f2328;">;</span>
</div><div class="l-line custom-highlight-class" style="background-color: #ffffcc; border-left: 3px solid #ff0000" data-line="3">    <span data-highlight="keyword" style="color: #cf222e;">let</span> <span data-highlight="variable" style="color: #1f2328;">y</span> <span data-highlight="operator" style="color: #0550ae;">=</span> <span data-highlight="number" style="color: #0550ae;">2</span><span data-highlight="punctuation.delimiter" style="color: #1f2328;">;</span>
</div><div class="l-line custom-highlight-class" style="background-color: #ffffcc; border-left: 3px solid #ff0000" data-line="4">    <span data-highlight="keyword" style="color: #cf222e;">let</span> <span data-highlight="variable" style="color: #1f2328;">z</span> <span data-highlight="operator" style="color: #0550ae;">=</span> <span data-highlight="number" style="color: #0550ae;">3</span><span data-highlight="punctuation.delimiter" style="color: #1f2328;">;</span>
</div><div class="l-line custom-highlight-class" style="background-color: #ffffcc; border-left: 3px solid #ff0000" data-line="5">    <span data-highlight="keyword" style="color: #cf222e;">let</span> <span data-highlight="variable" style="color: #1f2328;">message</span> <span data-highlight="operator" style="color: #0550ae;">=</span> <span data-highlight="string" style="color: #0a3069;">&quot;Hello, world!&quot;</span><span data-highlight="punctuation.delimiter" style="color: #1f2328;">;</span>
</div><div class="l-line" data-line="6"><span data-highlight="punctuation.bracket" style="color: #1f2328;">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_decorator_attributes_are_escaped() {
        let pre_class = "x\"><script>alert(1)</script>";

        let markdown = format!("```text pre_class='{}'\nhello\n```", pre_class);

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(&markdown, formatter, options);

        let expected = r#"<pre class="lumis x&quot;&gt;&lt;script&gt;alert(1)&lt;&#x2f;script&gt;"><code class="language-plaintext" translate="no" tabindex="0"><div class="l-line" data-line="1">hello
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_highlight_lines_class_decorator_attribute_is_escaped() {
        let highlight_lines_class = "x\"><script>alert(1)</script>";

        let markdown = format!(
            "```text highlight_lines=1 highlight_lines_class='{}'\nhello\n```",
            highlight_lines_class
        );

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(&markdown, formatter, options);

        let expected = r#"<pre class="lumis"><code class="language-plaintext" translate="no" tabindex="0"><div class="l-line x&quot;&gt;&lt;script&gt;alert(1)&lt;&#x2f;script&gt;" data-line="1">hello
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_highlight_lines_style_decorator_attribute_is_escaped() {
        let highlight_lines_style = "x\"><script>alert(1)</script>";

        let markdown = format!(
            "```text highlight_lines=1 highlight_lines_style='{}'\nhello\n```",
            highlight_lines_style
        );

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(&markdown, formatter, options);

        let expected = r#"<pre class="lumis"><code class="language-plaintext" translate="no" tabindex="0"><div class="l-line" style="x&quot;&gt;&lt;script&gt;alert(1)&lt;&#x2f;script&gt;" data-line="1">hello
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_decorator_attributes_respect_unsafe_render() {
        let pre_class = "x\"><script>alert(1)</script>";

        let markdown = format!("```text pre_class='{}'\nhello\n```", pre_class);

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;
        options.render.r#unsafe = true;

        let output = run_test(&markdown, formatter, options);

        let expected = r#"<pre class="lumis x"><script>alert(1)</script>"><code class="language-plaintext" translate="no" tabindex="0"><div class="l-line" data-line="1">hello
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_decorator_attributes_respect_unsafe_render_with_sanitize_clean() {
        let pre_class = "x\"><script>alert(1)</script>";

        let markdown = format!("```text pre_class='{}'\nhello\n```", pre_class);

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;
        options.render.r#unsafe = true;

        let unsafe_output = run_test(&markdown, formatter, options);
        let output = ammonia::clean(&unsafe_output);

        let expected = r#"<pre>"&gt;<code><div>hello
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_safe_decorator_attribute_with_unsafe_render_and_sanitize_clean() {
        let markdown = "```text pre_class='safe-class'\nhello\n```";

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;
        options.render.r#unsafe = true;

        let unsafe_output = run_test(markdown, formatter, options);
        let output = ammonia::clean(&unsafe_output);

        let expected = r#"<pre><code><div>hello
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_inline_decorator_highlight_lines_clamped_to_rendered_lines() {
        let markdown = r#"
```rust highlight_lines="1-99999999999" highlight_lines_style="background-color: yellow;"
fn main() {}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: None,
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(markdown, formatter, options);

        assert!(output
            .contains(r#"<div class="l-line" style="background-color: yellow;" data-line="1">"#));
        assert!(!output.contains(r#"data-line="2""#));
    }

    #[test]
    fn test_parse_highlight_lines_ignores_malformed_and_invalid_specs() {
        let lines = LumisAdapter::parse_highlight_lines("abc,1-x,,0,-1,5-3,999-1000,1", 3);

        assert_eq!(lines.len(), 1);
        assert!(lines.contains(&1));
    }

    #[test]
    fn test_parse_highlight_lines_clamps_mixed_ranges() {
        let lines = LumisAdapter::parse_highlight_lines("1-2,2-99999999999,3", 3);

        assert_eq!(lines.len(), 3);
        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        assert!(lines.contains(&3));
    }

    #[test]
    fn test_parse_highlight_lines_handles_whitespace_and_duplicates() {
        let lines = LumisAdapter::parse_highlight_lines(" 1 , 2 - 3 , 3 , 2 ", 3);

        assert_eq!(lines.len(), 3);
        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        assert!(lines.contains(&3));
    }

    #[test]
    fn test_parse_highlight_lines_returns_empty_when_no_rendered_lines() {
        let lines = LumisAdapter::parse_highlight_lines("1,1-99999999999", 0);

        assert!(lines.is_empty());
    }

    #[test]
    fn test_parse_highlight_lines_ignores_ranges_starting_after_rendered_lines() {
        let lines = LumisAdapter::parse_highlight_lines("4-99999999999,1-99999999999", 3);

        assert_eq!(lines.len(), 3);
        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        assert!(lines.contains(&3));
    }

    #[test]
    fn test_formatter_highlight_lines_clamped_to_rendered_lines() {
        let lines = vec![crate::types::elixir_types::ExLineSpec::Range {
            start: 1,
            end: 99_999_999_999,
        }];

        let converted = LumisAdapter::convert_line_specs(&lines, 1);

        assert_eq!(converted.len(), 1);
        assert!(converted.contains(&1));
    }

    #[test]
    fn test_formatter_highlight_lines_handles_empty_rendered_output() {
        let lines = vec![
            crate::types::elixir_types::ExLineSpec::Single(1),
            crate::types::elixir_types::ExLineSpec::Range {
                start: 1,
                end: 99_999_999_999,
            },
        ];

        let converted = LumisAdapter::convert_line_specs(&lines, 0);

        assert!(converted.is_empty());
    }

    #[test]
    fn test_formatter_highlight_lines_deduplicates_overlapping_specs() {
        let lines = vec![
            crate::types::elixir_types::ExLineSpec::Range { start: 1, end: 3 },
            crate::types::elixir_types::ExLineSpec::Range { start: 2, end: 4 },
            crate::types::elixir_types::ExLineSpec::Single(3),
        ];

        let converted = LumisAdapter::convert_line_specs(&lines, 3);

        assert_eq!(converted.len(), 3);
        assert!(converted.contains(&1));
        assert!(converted.contains(&2));
        assert!(converted.contains(&3));
    }

    #[test]
    fn test_formatter_highlight_lines_ignores_invalid_specs() {
        let lines = vec![
            crate::types::elixir_types::ExLineSpec::Single(0),
            crate::types::elixir_types::ExLineSpec::Single(999),
            crate::types::elixir_types::ExLineSpec::Range { start: 5, end: 3 },
            crate::types::elixir_types::ExLineSpec::Range {
                start: 999,
                end: 1000,
            },
            crate::types::elixir_types::ExLineSpec::Single(1),
        ];

        let converted = LumisAdapter::convert_line_specs(&lines, 3);

        assert_eq!(converted.len(), 1);
        assert!(converted.contains(&1));
    }

    #[test]
    fn test_html_inline_decorators_without_github_pre_lang() {
        let markdown = r#"
```rust pre_class="custom-class" highlight_lines="1,3" theme=github_light
fn main() {
    let x = 1;
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: Some(ThemeOrString::String("nord".to_string())),
            pre_class: Some("default-class".to_string()),
            italic: false,
            include_highlights: true,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = false;
        options.render.full_info_string = true;

        let output = run_test(markdown, formatter, options);

        let expected = r#"<pre class="lumis default-class" style="color: #d8dee9; background-color: #2e3440;"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" style="background-color: #e7eaf0;" data-line="1"><span data-highlight="keyword.function" style="color: #cf222e;">fn</span> <span data-highlight="function" style="color: #6639ba;">main</span><span data-highlight="punctuation.bracket" style="color: #1f2328;">(</span><span data-highlight="punctuation.bracket" style="color: #1f2328;">)</span> <span data-highlight="punctuation.bracket" style="color: #1f2328;">{</span>
</div><div class="l-line" data-line="2">    <span data-highlight="keyword" style="color: #cf222e;">let</span> <span data-highlight="variable" style="color: #1f2328;">x</span> <span data-highlight="operator" style="color: #0550ae;">=</span> <span data-highlight="number" style="color: #0550ae;">1</span><span data-highlight="punctuation.delimiter" style="color: #1f2328;">;</span>
</div><div class="l-line" style="background-color: #e7eaf0;" data-line="3">    <span data-highlight="keyword" style="color: #cf222e;">let</span> <span data-highlight="variable" style="color: #1f2328;">message</span> <span data-highlight="operator" style="color: #0550ae;">=</span> <span data-highlight="string" style="color: #0a3069;">&quot;Hello, world!&quot;</span><span data-highlight="punctuation.delimiter" style="color: #1f2328;">;</span>
</div><div class="l-line" data-line="4"><span data-highlight="punctuation.bracket" style="color: #1f2328;">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_linked_no_attrs() {
        let markdown = r#"
```rust
fn main() {
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlLinked {
            pre_class: None,
            highlight_lines: None,
            header: None,
        };

        let output = run_test(markdown, formatter, Options::default());

        let expected = r#"<pre class="lumis"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1"><span class="l-keyword-function">fn</span> <span class="l-function">main</span><span class="l-punctuation-bracket">(</span><span class="l-punctuation-bracket">)</span> <span class="l-punctuation-bracket">{</span>
</div><div class="l-line" data-line="2">    <span class="l-keyword">let</span> <span class="l-variable">message</span> <span class="l-operator">=</span> <span class="l-string">&quot;Hello, world!&quot;</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line" data-line="3"><span class="l-punctuation-bracket">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_linked_plaintext() {
        let markdown = r#"
```
plain
text
```
"#;

        let formatter = ExFormatterOption::HtmlLinked {
            pre_class: None,
            highlight_lines: None,
            header: None,
        };

        let output = run_test(markdown, formatter, Options::default());

        let expected = r#"<pre class="lumis"><code class="language-plaintext" translate="no" tabindex="0"><div class="l-line" data-line="1">plain
</div><div class="l-line" data-line="2">text
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_linked_all_attrs() {
        let markdown = r#"
```rust
fn main() {
    let a = 1;
    let b = 2;
    let sum = a + b;
}
```
"#;

        let formatter = ExFormatterOption::HtmlLinked {
            pre_class: Some("custom-linked-class".to_string()),
            highlight_lines: None,
            header: None,
        };

        let output = run_test(markdown, formatter, Options::default());

        let expected = r#"<pre class="lumis custom-linked-class"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1"><span class="l-keyword-function">fn</span> <span class="l-function">main</span><span class="l-punctuation-bracket">(</span><span class="l-punctuation-bracket">)</span> <span class="l-punctuation-bracket">{</span>
</div><div class="l-line" data-line="2">    <span class="l-keyword">let</span> <span class="l-variable">a</span> <span class="l-operator">=</span> <span class="l-number">1</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line" data-line="3">    <span class="l-keyword">let</span> <span class="l-variable">b</span> <span class="l-operator">=</span> <span class="l-number">2</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line" data-line="4">    <span class="l-keyword">let</span> <span class="l-variable">sum</span> <span class="l-operator">=</span> <span class="l-variable">a</span> <span class="l-operator">+</span> <span class="l-variable">b</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line" data-line="5"><span class="l-punctuation-bracket">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_html_linked_decorators() {
        let markdown = r#"
```rust pre_class="custom-linked-pre extra-linked" highlight_lines="1-2,4-5" highlight_lines_class="my-custom-highlight-line"
fn main() {
    let x = 1;
    let y = 2;
    let z = 3;
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlLinked {
            pre_class: Some("default-linked-pre".to_string()),
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(markdown, formatter, options);

        let expected = r#"<pre class="lumis custom-linked-pre extra-linked"><code class="language-rust" translate="no" tabindex="0"><div class="l-line my-custom-highlight-line" data-line="1"><span class="l-keyword-function">fn</span> <span class="l-function">main</span><span class="l-punctuation-bracket">(</span><span class="l-punctuation-bracket">)</span> <span class="l-punctuation-bracket">{</span>
</div><div class="l-line my-custom-highlight-line" data-line="2">    <span class="l-keyword">let</span> <span class="l-variable">x</span> <span class="l-operator">=</span> <span class="l-number">1</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line" data-line="3">    <span class="l-keyword">let</span> <span class="l-variable">y</span> <span class="l-operator">=</span> <span class="l-number">2</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line my-custom-highlight-line" data-line="4">    <span class="l-keyword">let</span> <span class="l-variable">z</span> <span class="l-operator">=</span> <span class="l-number">3</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line my-custom-highlight-line" data-line="5">    <span class="l-keyword">let</span> <span class="l-variable">message</span> <span class="l-operator">=</span> <span class="l-string">&quot;Hello, world!&quot;</span><span class="l-punctuation-delimiter">;</span>
</div><div class="l-line" data-line="6"><span class="l-punctuation-bracket">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_decorator_include_highlights_with_value() {
        let markdown = r#"
```rust include_highlights=true
fn main() {
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: Some(ThemeOrString::String("nord".to_string())),
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(markdown, formatter, options);

        let expected = r#"<pre class="lumis" style="color: #d8dee9; background-color: #2e3440;"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1"><span data-highlight="keyword.function" style="color: #88c0d0;">fn</span> <span data-highlight="function" style="color: #88c0d0;">main</span><span data-highlight="punctuation.bracket" style="color: #88c0d0;">(</span><span data-highlight="punctuation.bracket" style="color: #88c0d0;">)</span> <span data-highlight="punctuation.bracket" style="color: #88c0d0;">{</span>
</div><div class="l-line" data-line="2">    <span data-highlight="keyword" style="color: #81a1c1;">let</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">message</span> <span data-highlight="operator" style="color: #81a1c1;">=</span> <span data-highlight="string" style="color: #a3be8c;">&quot;Hello, world!&quot;</span><span data-highlight="punctuation.delimiter" style="color: #88c0d0;">;</span>
</div><div class="l-line" data-line="3"><span data-highlight="punctuation.bracket" style="color: #88c0d0;">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }

    #[test]
    fn test_decorator_include_highlights_standalone() {
        let markdown = r#"
```rust include_highlights
fn main() {
    let message = "Hello, world!";
}
```
"#;

        let formatter = ExFormatterOption::HtmlInline {
            theme: Some(ThemeOrString::String("nord".to_string())),
            pre_class: None,
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        };

        let mut options = Options::default();
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;

        let output = run_test(markdown, formatter, options);

        let expected = r#"<pre class="lumis" style="color: #d8dee9; background-color: #2e3440;"><code class="language-rust" translate="no" tabindex="0"><div class="l-line" data-line="1"><span data-highlight="keyword.function" style="color: #88c0d0;">fn</span> <span data-highlight="function" style="color: #88c0d0;">main</span><span data-highlight="punctuation.bracket" style="color: #88c0d0;">(</span><span data-highlight="punctuation.bracket" style="color: #88c0d0;">)</span> <span data-highlight="punctuation.bracket" style="color: #88c0d0;">{</span>
</div><div class="l-line" data-line="2">    <span data-highlight="keyword" style="color: #81a1c1;">let</span> <span data-highlight="variable" style="color: #d8dee9; font-weight: bold;">message</span> <span data-highlight="operator" style="color: #81a1c1;">=</span> <span data-highlight="string" style="color: #a3be8c;">&quot;Hello, world!&quot;</span><span data-highlight="punctuation.delimiter" style="color: #88c0d0;">;</span>
</div><div class="l-line" data-line="3"><span data-highlight="punctuation.bracket" style="color: #88c0d0;">}</span>
</div></code></pre>"#;

        assert_str_eq!(output.trim(), expected.trim());
    }
}
