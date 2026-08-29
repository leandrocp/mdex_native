use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::Mutex;

use comrak::adapters::SyntaxHighlighterAdapter;
use rustler::env::SavedTerm;
use rustler::{Atom, Encoder, Error, OwnedEnv, Term};

rustler::atoms! {
    provider_module = "Elixir.Lumis.Native",
    bridge_name = "lumis_mdex_bridge_v1",
    ok,
    nil,
}

pub struct LumisBridgeConfig {
    config: Mutex<(OwnedEnv, SavedTerm)>,
}

impl fmt::Debug for LumisBridgeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LumisBridgeConfig").finish()
    }
}

impl LumisBridgeConfig {
    pub fn new<'a>(bridge: Term<'a>, formatter: Option<Term<'a>>) -> Self {
        let env = OwnedEnv::new();
        let formatter = formatter.unwrap_or_else(|| nil().to_term(bridge.get_env()));
        let config = (bridge, formatter).encode(bridge.get_env());
        let config = env.save(config);

        Self {
            config: Mutex::new((env, config)),
        }
    }

    fn render(
        &self,
        source: &str,
        language: Option<&str>,
        attributes: &HashMap<String, String>,
        render_unsafe: bool,
    ) -> Result<String, Error> {
        let config = self.config.lock().map_err(|_| Error::BadArg)?;

        config.0.run(|env| {
            let (bridge, formatter): (Term<'_>, Term<'_>) = config.1.load(env).decode()?;
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
            .map_err(|_| Error::Atom("lumis_bridge_call_failed"))?;

            let response = unsafe { Term::new(env, call_term) };
            let (status, value): (Atom, String) = response.decode()?;

            if status == ok() {
                Ok(value)
            } else {
                Err(Error::Term(Box::new(value)))
            }
        })
    }
}

pub struct LumisAdapter {
    bridge: LumisBridgeConfig,
    render_unsafe: bool,
    stored_attributes: Mutex<HashMap<String, String>>,
    stored_language: Mutex<Option<String>>,
}

impl LumisAdapter {
    pub fn new(bridge: LumisBridgeConfig, render_unsafe: bool) -> Self {
        Self {
            bridge,
            render_unsafe,
            stored_attributes: Mutex::new(HashMap::new()),
            stored_language: Mutex::new(None),
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

    fn update_state(
        &self,
        attributes: &HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> fmt::Result {
        let custom_attributes = attributes
            .get("data-meta")
            .and_then(|metadata| Self::parse_custom_attributes(metadata.as_ref()));
        if let Some(custom_attributes) = custom_attributes {
            *self.stored_attributes.lock().map_err(|_| fmt::Error)? = custom_attributes;
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
            *self.stored_language.lock().map_err(|_| fmt::Error)? = language;
        }

        Ok(())
    }
}

impl SyntaxHighlighterAdapter for LumisAdapter {
    fn write_pre_tag(
        &self,
        _output: &mut dyn Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> fmt::Result {
        *self.stored_language.lock().map_err(|_| fmt::Error)? = None;
        self.stored_attributes
            .lock()
            .map_err(|_| fmt::Error)?
            .clear();
        self.update_state(&attributes)
    }

    fn write_code_tag(
        &self,
        _output: &mut dyn Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'_, str>>,
    ) -> fmt::Result {
        self.update_state(&attributes)
    }

    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        language: Option<&str>,
        source: &str,
    ) -> fmt::Result {
        let stored_language = self.stored_language.lock().map_err(|_| fmt::Error)?;
        let language = stored_language.as_deref().or(language);
        let attributes = self.stored_attributes.lock().map_err(|_| fmt::Error)?;
        let highlighted = self
            .bridge
            .render(source, language, &attributes, self.render_unsafe)
            .map_err(|_| fmt::Error)?;

        output.write_str(&highlighted)
    }
}
