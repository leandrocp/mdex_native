use lumis_wasm_runtime::{HighlightOptions, DEFAULT_MATCH_LIMIT, DEFAULT_TIME_LIMIT};
use rustler::{Decoder, NifResult, Term};

use super::optional_field;

mod atoms {
    rustler::atoms! { time_limit, match_limit }
}

/// Lumis sends the budget as a map. Missing or nil fields use its defaults;
/// only an explicit zero disables the time limit.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExBudget {
    time_limit: Option<u64>,
    match_limit: Option<u32>,
}

impl<'a> Decoder<'a> for ExBudget {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(Self {
            time_limit: optional_field(term, atoms::time_limit())?,
            match_limit: optional_field(term, atoms::match_limit())?,
        })
    }
}

impl ExBudget {
    pub fn highlight_options(self, rainbow_brackets: bool) -> HighlightOptions {
        HighlightOptions {
            rainbow_brackets,
            time_limit_ms: match self.time_limit {
                Some(0) => None,
                Some(ms) => Some(ms),
                None => Some(DEFAULT_TIME_LIMIT),
            },
            match_limit: self.match_limit.unwrap_or(DEFAULT_MATCH_LIMIT),
            ..HighlightOptions::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_lumis_time_and_match_limits() {
        let options = ExBudget::default().highlight_options(false);
        assert_eq!(options.time_limit_ms, Some(5_000));
        assert_eq!(options.match_limit, 8192);
    }

    #[test]
    fn zero_removes_only_the_time_limit() {
        let options = ExBudget {
            time_limit: Some(0),
            match_limit: Some(4096),
        }
        .highlight_options(true);
        assert_eq!(options.time_limit_ms, None);
        assert_eq!(options.match_limit, 4096);
        assert!(options.rainbow_brackets);
    }
}
