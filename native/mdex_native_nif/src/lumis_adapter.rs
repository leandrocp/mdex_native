use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::Mutex;

use crate::types::options::ExBudget;
use comrak::adapters::SyntaxHighlighterAdapter;
use lumis_core::code_fence::{parse_meta, CodeFenceMeta};
use lumis_core::elixir::ExFormatterOption;

use crate::html_formatter::InlineCodeHighlighter;
use crate::lumis_render::{render_code_fence, render_inline_code};

const POISONED: &str = "a Lumis adapter lock is poisoned";

/// What the `<pre>` and `<code>` tags said about the fence Comrak is about to
/// hand to `write_highlighted`. Comrak reports the info string and the language
/// on those tags and neither on the fence body.
#[derive(Default)]
struct FenceState {
    meta: CodeFenceMeta,
    language: Option<String>,
}

impl FenceState {
    fn reset(&mut self) {
        self.meta = CodeFenceMeta::default();
        self.language = None;
    }

    fn update(&mut self, attributes: &HashMap<&'static str, std::borrow::Cow<'_, str>>) {
        if let Some(meta) = attributes.get("data-meta") {
            self.meta = parse_meta(meta);
        }

        let language = attributes
            .get("lang")
            .map(|language| language.to_string())
            .or_else(|| {
                attributes
                    .get("class")
                    .and_then(|class| class.strip_prefix("language-"))
                    .map(str::to_string)
            });

        if language.is_some() {
            self.language = language;
        }
    }
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
        attributes: HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> fmt::Result {
        let mut fence = self.fence.lock().map_err(|_| self.fail(POISONED.into()))?;
        fence.reset();
        fence.update(&attributes);

        Ok(())
    }

    fn write_code_tag(
        &self,
        _output: &mut dyn Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'_, str>>,
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
        language: Option<&str>,
        source: &str,
    ) -> fmt::Result {
        let fence = self.fence.lock().map_err(|_| self.fail(POISONED.into()))?;
        let language = fence.language.as_deref().or(language);
        let highlighted = render_code_fence(
            source,
            language,
            Some(self.formatter.clone()),
            self.rainbow_brackets,
            self.word_diff,
            self.budget,
            &fence.meta,
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
    use std::borrow::Cow;

    fn tag(attributes: &[(&'static str, &str)]) -> HashMap<&'static str, Cow<'static, str>> {
        attributes
            .iter()
            .map(|(key, value)| (*key, Cow::Owned(value.to_string())))
            .collect()
    }

    #[test]
    fn reads_the_language_from_either_tag() {
        let mut fence = FenceState::default();
        fence.update(&tag(&[("class", "language-rust")]));
        assert_eq!(fence.language.as_deref(), Some("rust"));

        let mut fence = FenceState::default();
        fence.update(&tag(&[("lang", "elixir")]));
        assert_eq!(fence.language.as_deref(), Some("elixir"));
    }

    #[test]
    fn a_language_does_not_leak_into_the_next_fence() {
        let mut fence = FenceState::default();
        fence.update(&tag(&[("class", "language-rust")]));

        fence.reset();
        fence.update(&tag(&[("class", "plain")]));

        assert_eq!(fence.language, None);
    }

    #[test]
    fn decorator_attributes_do_not_leak_into_the_next_fence() {
        let mut fence = FenceState::default();
        fence.update(&tag(&[
            ("class", "language-rust"),
            ("data-meta", "highlight_lines=1 theme=github_light"),
        ]));
        assert_eq!(fence.meta.theme.as_deref(), Some("github_light"));

        fence.reset();
        fence.update(&tag(&[("class", "language-rust")]));

        assert_eq!(fence.meta, CodeFenceMeta::default());
    }
}
