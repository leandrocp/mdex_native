use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::Mutex;

use comrak::adapters::SyntaxHighlighterAdapter;
use rustler::env::SavedTerm;
use rustler::{Atom, Encoder, OwnedEnv, Term};

rustler::atoms! {
    provider_module = "Elixir.Lumis.Native",
    bridge_name = "lumis_mdex_bridge_v1",
    ok,
    nil,
}

const POISONED: &str = "a Lumis bridge lock is poisoned";

pub struct LumisBridgeConfig {
    state: Mutex<BridgeState>,
}

struct BridgeState {
    config_env: OwnedEnv,
    config: SavedTerm,
    call_env: OwnedEnv,
}

impl fmt::Debug for LumisBridgeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LumisBridgeConfig").finish()
    }
}

impl LumisBridgeConfig {
    pub fn new<'a>(bridge: Term<'a>, formatter: Option<Term<'a>>) -> Self {
        let config_env = OwnedEnv::new();
        let formatter = formatter.unwrap_or_else(|| nil().to_term(bridge.get_env()));
        let config = (bridge, formatter).encode(bridge.get_env());
        let config = config_env.save(config);

        Self {
            state: Mutex::new(BridgeState {
                config_env,
                config,
                call_env: OwnedEnv::new(),
            }),
        }
    }

    fn render(
        &self,
        source: &str,
        language: Option<&str>,
        attributes: &HashMap<String, String>,
        render_unsafe: bool,
    ) -> Result<String, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "the Lumis bridge lock is poisoned".to_string())?;
        let BridgeState {
            config_env,
            config,
            call_env,
        } = &mut *state;

        let rendered = config_env.run(|config_env| {
            let (bridge, formatter): (Term<'_>, Term<'_>) = config
                .load(config_env)
                .decode()
                .map_err(|_| "the Lumis bridge configuration is not a resource".to_string())?;

            call_env.run(|env| {
                let bridge = bridge.in_env(env);
                let formatter = formatter.in_env(env);
                let request = (source, language, formatter, attributes, render_unsafe).encode(env);
                let mut call_term = request.as_c_arg();

                unsafe {
                    env.dynamic_resource_call(
                        provider_module(),
                        bridge_name(),
                        bridge,
                        (&mut call_term as *mut usize).cast(),
                    )
                }
                .map_err(|_| {
                    "the loaded Lumis NIF does not export lumis_mdex_bridge_v1".to_string()
                })?;

                let response = unsafe { Term::new(env, call_term) };
                let (status, value): (Atom, String) = response
                    .decode()
                    .map_err(|_| "the Lumis bridge returned an unexpected response".to_string())?;

                if status == ok() {
                    Ok(value)
                } else {
                    Err(value)
                }
            })
        });

        // A NIF environment never collects on its own, so the request and the
        // rendered fragment for every fence would otherwise pile up until the
        // whole document finished rendering.
        call_env.clear();

        rendered
    }
}

/// What the `<pre>` and `<code>` tags said about the fence Comrak is about to
/// hand to `write_highlighted`. Comrak reports the info string and the language
/// on those tags and neither on the fence body.
#[derive(Default)]
struct FenceState {
    attributes: HashMap<String, String>,
    language: Option<String>,
}

impl FenceState {
    fn reset(&mut self) {
        self.attributes.clear();
        self.language = None;
    }

    fn update(&mut self, attributes: &HashMap<&'static str, std::borrow::Cow<'_, str>>) {
        if let Some(custom_attributes) = attributes
            .get("data-meta")
            .and_then(|metadata| parse_custom_attributes(metadata.as_ref()))
        {
            self.attributes = custom_attributes;
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
    bridge: LumisBridgeConfig,
    render_unsafe: bool,
    fence: Mutex<FenceState>,
    /// Comrak's adapter trait can only answer `fmt::Error`, which says nothing
    /// about why Lumis refused a fence. Park the reason here for the NIF to read
    /// once the render has unwound.
    failure: Mutex<Option<String>>,
}

impl LumisAdapter {
    pub fn new(bridge: LumisBridgeConfig, render_unsafe: bool) -> Self {
        Self {
            bridge,
            render_unsafe,
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
        let highlighted = self
            .bridge
            .render(source, language, &fence.attributes, self.render_unsafe)
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
        assert_eq!(fence.attributes.len(), 2);

        fence.reset();
        fence.update(&tag(&[("class", "language-rust")]));

        assert!(fence.attributes.is_empty());
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
