use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};
use std::sync::Mutex;

use crate::types::options::ExBudget;
use comrak::adapters::{SyntaxHighlightMeta, SyntaxHighlighterAdapter};
use lumis_core::elixir::ExFormatterOption;

use crate::html_formatter::InlineCodeHighlighter;
use crate::lumis_render::{render_code_fence, render_inline_code};

const POISONED: &str = "a Lumis adapter lock is poisoned";

/// What the `<pre>` and `<code>` tags carried for the fence Comrak is about to
/// hand to `write_highlighted`: the decorators, which Comrak puts in
/// `data-meta` only with `render: [full_info_string: true]`, and the fence's
/// own attributes, which it puts on `<pre>` only with `render: [unsafe: true,
/// attributes: true]`.
#[derive(Default)]
struct FenceState {
    attributes: HashMap<String, String>,
    pre_attrs: Vec<(String, String)>,
}

/// Attributes Comrak itself writes on a fence's `<pre>`. MDExNative never
/// wrote them on a Lumis `<pre>`, and the language and decorators reach Lumis
/// another way.
const GENERATED_PRE_ATTRS: [&str; 4] = ["lang", "data-meta", "data-sourcepos", "data-math-style"];

impl FenceState {
    fn reset(&mut self) {
        self.attributes.clear();
        self.pre_attrs.clear();
    }

    fn record_pre_attrs(&mut self, attributes: &BTreeMap<&str, Cow<'_, str>>) {
        self.pre_attrs = attributes
            .iter()
            .filter(|(name, _)| !GENERATED_PRE_ATTRS.contains(name))
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
    }

    fn update(&mut self, attributes: &BTreeMap<&str, Cow<'_, str>>) {
        if let Some(custom_attributes) = attributes
            .get("data-meta")
            .and_then(|metadata| parse_custom_attributes(metadata.as_ref()))
        {
            self.attributes = custom_attributes;
        }
    }
}

fn parse_custom_attributes(info_string: &str) -> Option<HashMap<String, String>> {
    let tokens = shlex::split(info_string)?;
    let attributes = tokens
        .into_iter()
        .map(|token| match token.split_once('=') {
            Some((key, value)) => (key.trim().to_string(), value.to_string()),
            None => (token.trim().to_string(), "true".to_string()),
        })
        .collect();

    Some(attributes)
}

pub struct LumisAdapter {
    formatter: ExFormatterOption,
    rainbow_brackets: bool,
    word_diff: bool,
    budget: ExBudget,
    fence: Mutex<FenceState>,
    /// Comrak's adapter trait can only answer `fmt::Error`, which says nothing
    /// about why Lumis refused a fence. Park the reason here for the NIF to
    /// read once the render has unwound.
    failure: Mutex<Option<String>>,
}

impl LumisAdapter {
    pub fn new(
        formatter: ExFormatterOption,
        rainbow_brackets: bool,
        word_diff: bool,
        budget: ExBudget,
    ) -> Self {
        Self {
            formatter,
            rainbow_brackets,
            word_diff,
            budget,
            fence: Mutex::new(FenceState::default()),
            failure: Mutex::new(None),
        }
    }

    pub fn take_failure(&self) -> Option<String> {
        self.failure.lock().ok()?.take()
    }

    fn fail(&self, reason: String) -> fmt::Error {
        if let Ok(mut failure) = self.failure.lock() {
            failure.get_or_insert(reason);
        }

        fmt::Error
    }
}

impl SyntaxHighlighterAdapter for LumisAdapter {
    /// Lumis writes the opening tags itself while rendering the fence, so these
    /// two only record what Comrak reported about it.
    fn write_pre_tag(
        &self,
        _output: &mut dyn Write,
        _highlight: &SyntaxHighlightMeta<'_>,
        attributes: BTreeMap<&str, Cow<'_, str>>,
    ) -> fmt::Result {
        let mut fence = self.fence.lock().map_err(|_| self.fail(POISONED.into()))?;
        fence.reset();
        fence.update(&attributes);
        fence.record_pre_attrs(&attributes);

        Ok(())
    }

    fn write_code_tag(
        &self,
        _output: &mut dyn Write,
        _highlight: &SyntaxHighlightMeta<'_>,
        attributes: BTreeMap<&str, Cow<'_, str>>,
    ) -> fmt::Result {
        self.fence
            .lock()
            .map_err(|_| self.fail(POISONED.into()))?
            .update(&attributes);

        Ok(())
    }

    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        highlight: &SyntaxHighlightMeta<'_>,
    ) -> fmt::Result {
        let fence = self.fence.lock().map_err(|_| self.fail(POISONED.into()))?;
        let highlighted = render_code_fence(
            highlight.code,
            Some(highlight.lang),
            Some(self.formatter.clone()),
            self.rainbow_brackets,
            self.word_diff,
            self.budget,
            &fence.attributes,
            &fence.pre_attrs,
        )
        .map_err(|reason| self.fail(reason))?;

        output.write_str(&highlighted)
    }
}

impl InlineCodeHighlighter for LumisAdapter {
    fn write_inline_code(
        &self,
        output: &mut dyn Write,
        source: &str,
        language: &str,
        attrs: &[(String, String)],
    ) -> fmt::Result {
        let highlighted = render_inline_code(
            source,
            language,
            Some(self.formatter.clone()),
            self.rainbow_brackets,
            self.word_diff,
            self.budget,
            attrs,
        )
        .map_err(|reason| self.fail(reason))?;

        output.write_str(&highlighted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(attributes: &[(&'static str, &str)]) -> BTreeMap<&'static str, Cow<'static, str>> {
        attributes
            .iter()
            .map(|(key, value)| (*key, Cow::Owned(value.to_string())))
            .collect()
    }

    #[test]
    fn decorator_attributes_do_not_leak_into_the_next_fence() {
        let mut fence = FenceState::default();
        fence.update(&tag(&[
            ("class", "language-rust"),
            ("data-meta", "highlight_lines=1 theme=github_light"),
        ]));
        assert_eq!(fence.attributes.len(), 2);

        fence.reset();
        fence.update(&tag(&[("class", "language-rust")]));

        assert!(fence.attributes.is_empty());
    }

    #[test]
    fn keeps_only_authored_pre_attributes() {
        let mut fence = FenceState::default();
        fence.record_pre_attrs(&tag(&[
            ("lang", "rust"),
            ("data-sourcepos", "1:1-3:3"),
            ("data-meta", "theme=x"),
            ("id", "main"),
            ("class", "n"),
        ]));
        assert_eq!(
            fence.pre_attrs,
            [
                ("class".to_string(), "n".to_string()),
                ("id".to_string(), "main".to_string())
            ]
        );

        fence.reset();
        assert!(fence.pre_attrs.is_empty());
    }

    #[test]
    fn a_bare_flag_in_the_info_string_reads_as_true() {
        let attributes = parse_custom_attributes("include_highlights theme=onedark").unwrap();
        assert_eq!(attributes["include_highlights"], "true");
        assert_eq!(attributes["theme"], "onedark");
    }

    #[test]
    fn a_quoted_decorator_value_keeps_its_spaces() {
        let attributes = parse_custom_attributes(r#"highlight_lines_style="color: red;""#).unwrap();
        assert_eq!(attributes["highlight_lines_style"], "color: red;");
    }
}
