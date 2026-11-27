#[cfg(not(target_os = "wasi"))]
use lumis_wasm_runtime::HighlightOptions;
use lumis_wasm_runtime::{DEFAULT_MATCH_LIMIT, DEFAULT_TIME_LIMIT};

/// The shared Lumis decoder supplies the limits. Missing or nil fields use defaults;
/// only an explicit zero disables the time limit.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExBudget {
    time_limit: Option<u64>,
    match_limit: Option<u32>,
}

impl ExBudget {
    pub fn new(time_limit: Option<u64>, match_limit: Option<u32>) -> Self {
        Self {
            time_limit,
            match_limit,
        }
    }

    /// How long one block may highlight for, or `None` for no bound.
    pub fn time_limit_ms(self) -> Option<u64> {
        match self.time_limit {
            Some(0) => None,
            Some(ms) => Some(ms),
            None => Some(DEFAULT_TIME_LIMIT),
        }
    }

    pub fn match_limit(self) -> u32 {
        self.match_limit.unwrap_or(DEFAULT_MATCH_LIMIT)
    }

    #[cfg(not(target_os = "wasi"))]
    pub fn highlight_options(self, rainbow_brackets: bool, word_diff: bool) -> HighlightOptions {
        HighlightOptions {
            rainbow_brackets,
            word_diff,
            time_limit_ms: self.time_limit_ms(),
            match_limit: self.match_limit(),
            ..HighlightOptions::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_lumis_time_and_match_limits() {
        let budget = ExBudget::default();
        assert_eq!(budget.time_limit_ms(), Some(5_000));
        assert_eq!(budget.match_limit(), 8192);
    }

    #[test]
    fn zero_removes_only_the_time_limit() {
        let budget = ExBudget::new(Some(0), Some(4096));
        assert_eq!(budget.time_limit_ms(), None);
        assert_eq!(budget.match_limit(), 4096);
    }

    #[cfg(not(target_os = "wasi"))]
    #[test]
    fn highlight_options_carry_the_budget_and_flags() {
        let options = ExBudget::new(Some(250), Some(4096)).highlight_options(true, false);
        assert_eq!(options.time_limit_ms, Some(250));
        assert_eq!(options.match_limit, 4096);
        assert!(options.rainbow_brackets);
        assert!(!options.word_diff);
    }
}
