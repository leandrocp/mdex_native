//! The Lumis runtime of a `target_os = "wasi"` build, with the interface of
//! `lumis_runtime.rs`.
//!
//! Wasmtime cannot run inside WebAssembly, so parsers cannot arrive as WASM
//! modules here. The grammars are the ones the `lumis` crate links statically,
//! every language it has, and there is no store to configure.
//!
//! Highlighting runs on one thread of this module's own, for the same 8 MiB
//! stack the native runtime gives its workers: nested injections recurse once
//! per layer. One, because every thread's stack comes out of the module's
//! single linear memory, which never shrinks.

use std::sync::mpsc;
use std::thread;

use anyhow::{anyhow, Context, Result};
use lumis::highlight::{highlight_events_for_render, Budget, HighlightError, HighlightOptions};
use lumis::languages::Language;
use lumis_core::events::HighlightEvent;
use lumis_core::formatter::BudgetExhausted;
use once_cell::sync::Lazy;

use crate::types::options::ExBudget;

static EXECUTOR: Lazy<Result<Executor>> = Lazy::new(Executor::new);

/// What `lumis_wasm_runtime::HighlightOutput` carries that a render uses.
pub struct HighlightOutput {
    pub events: Vec<HighlightEvent<'static>>,
    pub budget: Option<BudgetExhausted>,
}

struct Job {
    source: String,
    language: String,
    rainbow_brackets: bool,
    word_diff: bool,
    budget: ExBudget,
    reply: mpsc::SyncSender<Result<HighlightOutput>>,
}

pub struct Executor {
    sender: mpsc::SyncSender<Job>,
}

impl Executor {
    fn new() -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel::<Job>(2);

        thread::Builder::new()
            .name("mdex-lumis-0".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                for job in receiver {
                    let _ = job.reply.send(highlight(&job));
                }
            })
            .context("could not spawn the Lumis worker")?;

        Ok(Self { sender })
    }

    pub fn highlight(
        &self,
        source: &str,
        language: &str,
        rainbow_brackets: bool,
        word_diff: bool,
        budget: ExBudget,
    ) -> Result<HighlightOutput> {
        let (reply, answer) = mpsc::sync_channel(1);

        self.sender
            .send(Job {
                source: source.to_string(),
                language: language.to_string(),
                rainbow_brackets,
                word_diff,
                budget,
                reply,
            })
            .map_err(|_| anyhow!("the Lumis executor is unavailable"))?;

        answer
            .recv()
            .map_err(|_| anyhow!("the Lumis executor stopped before answering"))?
    }
}

/// Reports the budget the way the native runtime does: a render that ran out
/// of time is the whole source, unhighlighted, marked `Time`.
fn highlight(job: &Job) -> Result<HighlightOutput> {
    let language: Language = job.language.parse()?;
    let options = HighlightOptions::new()
        .rainbow_brackets(job.rainbow_brackets)
        .word_diff(job.word_diff)
        .budget(
            Budget::new()
                .match_limit(job.budget.match_limit())
                .time_limit(job.budget.time_limit_ms()),
        );

    match highlight_events_for_render(&job.source, language, options) {
        Ok((events, exceeded_match_limit)) => Ok(HighlightOutput {
            events,
            budget: exceeded_match_limit.then_some(BudgetExhausted::Matches),
        }),
        Err(HighlightError::TimeLimit) => Ok(HighlightOutput {
            events: vec![HighlightEvent::Source {
                start: 0,
                end: job.source.len(),
            }],
            budget: Some(BudgetExhausted::Time),
        }),
        Err(error) => Err(error.into()),
    }
}

pub fn executor() -> Result<&'static Executor> {
    EXECUTOR.as_ref().map_err(|error| anyhow!("{error:#}"))
}

/// Nothing to point anywhere: the grammars are compiled in.
#[cfg_attr(not(test), rustler::nif)]
fn configure_lumis_store(_data_dir: Option<String>, _installed_dirs: Vec<String>) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlight(source: &str, language: &str, budget: ExBudget) -> HighlightOutput {
        executor()
            .unwrap()
            .highlight(source, language, false, false, budget)
            .unwrap()
    }

    #[test]
    fn highlights_with_the_linked_grammars() {
        let output = highlight("defmodule A do end", "elixir", ExBudget::default());

        assert!(output.budget.is_none());
        assert!(output
            .events
            .iter()
            .any(|event| matches!(event, HighlightEvent::Start { .. })));
    }

    #[test]
    fn a_render_out_of_time_is_plain_and_marked() {
        let source = "let x = [1, 2, 3];\n".repeat(100_000);
        let output = highlight(&source, "rust", ExBudget::new(Some(1), None));

        assert!(matches!(output.budget, Some(BudgetExhausted::Time)));
        assert!(matches!(
            output.events[..],
            [HighlightEvent::Source { start: 0, end }] if end == source.len()
        ));
    }

    #[test]
    fn an_unknown_language_is_an_error() {
        let output =
            executor()
                .unwrap()
                .highlight("x", "not-a-language", false, false, ExBudget::default());

        assert!(output.is_err());
    }

    #[test]
    fn renders_a_highlighted_code_fence() {
        let html = crate::lumis_render::render_code_fence(
            "IO.puts(:hi)\n",
            Some("elixir"),
            None,
            false,
            false,
            ExBudget::default(),
            &Default::default(),
            &[],
        )
        .unwrap();

        assert!(html.contains("<span"), "{html}");
    }
}
