//! The Lumis WASM runtime this NIF highlights with.
//!
//! Parsers are WebAssembly modules that arrive as `lumis_wasm_*` dependencies,
//! not grammars compiled into this library. The `:lumis` application reports
//! where they were installed and where compiled modules are kept, and
//! [`configure_lumis_store`] points this runtime at the same directories, so a
//! parser is compiled once for the whole VM rather than once per NIF.
//!
//! Highlighting runs on this module's own threads rather than the caller's.
//! They exist for their 8 MiB stacks: nested injections recurse per layer and
//! overflow the BEAM dirty-scheduler default, which crashes the VM rather than
//! erroring. `lumis_nif` keeps a pool of its own for the same reason.

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use anyhow::{anyhow, Context, Result};
use lumis_core::events::HighlightEvent;
use lumis_wasm_runtime::{catalog, store, Runtime, RuntimeError};
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};

/// Directories the store reads, as `MDExNative.Application` configured them.
///
/// Elixir cannot set an OS environment variable this NIF would see, so the
/// `:lumis` application's answers arrive over a NIF call at boot instead.
#[derive(Default)]
struct StorePaths {
    data_dir: Option<PathBuf>,
    /// `priv/parsers` of every installed `lumis_wasm_*` application.
    ///
    /// Empty is a real answer — the project depends on no parsers, so it may
    /// load none. Nothing is fetched to make up the difference.
    installed_dirs: Vec<PathBuf>,
}

static STORE_PATHS: Lazy<RwLock<StorePaths>> = Lazy::new(|| RwLock::new(StorePaths::default()));

static EXECUTOR: Lazy<Result<Executor>> = Lazy::new(Executor::new);

enum Job {
    LoadNamed {
        name: String,
        reply: mpsc::SyncSender<Result<(), RuntimeError>>,
    },
    Highlight {
        source: String,
        language: String,
        rainbow_brackets: bool,
        reply: mpsc::SyncSender<Result<Vec<HighlightEvent<'static>>, RuntimeError>>,
    },
}

pub struct Executor {
    sender: mpsc::SyncSender<Job>,
}

impl Executor {
    fn new() -> Result<Self> {
        // Sized to the machine, not capped, matching `lumis_nif`.
        let workers = thread::available_parallelism().map_or(1, usize::from);

        // Built on one of these threads too: loading a language compiles WASM,
        // which recurses as deeply as highlighting does.
        let runtime = thread::Builder::new()
            .name("mdex-lumis-init".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || -> Result<Runtime, RuntimeError> {
                let runtime = Runtime::with_worker_limit(workers)?.with_store(language_store());
                for language in catalog::LANGUAGES {
                    runtime.declare_language(language.id, language.aliases);
                }
                Ok(runtime)
            })
            .context("could not spawn the Lumis WASM runtime initializer")?
            .join()
            .map_err(|_| anyhow!("Lumis WASM runtime initialization panicked"))?
            .context("could not start the Lumis WASM runtime")?;

        let runtime = Arc::new(runtime);
        let (sender, receiver) = mpsc::sync_channel::<Job>(workers * 2);
        let receiver = Arc::new(Mutex::new(receiver));

        for index in 0..workers {
            let runtime = Arc::clone(&runtime);
            let receiver = Arc::clone(&receiver);
            thread::Builder::new()
                .name(format!("mdex-lumis-{index}"))
                .stack_size(8 * 1024 * 1024)
                .spawn(move || loop {
                    let Ok(job) = receiver.lock().recv() else {
                        return;
                    };
                    match job {
                        Job::LoadNamed { name, reply } => {
                            let _ = reply.send(runtime.load_named_language(&name));
                        }
                        Job::Highlight {
                            source,
                            language,
                            rainbow_brackets,
                            reply,
                        } => {
                            let events = runtime.highlight(&source, &language, rainbow_brackets);
                            let _ = reply.send(events);
                        }
                    }
                })
                .context("could not spawn a Lumis WASM worker")?;
        }

        Ok(Self { sender })
    }

    fn submit<T>(
        &self,
        job: impl FnOnce(mpsc::SyncSender<Result<T, RuntimeError>>) -> Job,
    ) -> Result<T, RuntimeError> {
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender.send(job(reply)).map_err(|_| {
            RuntimeError::Highlight("the Lumis WASM executor is unavailable".into())
        })?;
        answer.recv().map_err(|_| {
            RuntimeError::Highlight("the Lumis WASM executor stopped before answering".into())
        })?
    }

    pub fn highlight(
        &self,
        source: &str,
        language: &str,
        rainbow_brackets: bool,
    ) -> Result<Vec<HighlightEvent<'static>>, RuntimeError> {
        self.submit(|reply| Job::Highlight {
            source: source.to_string(),
            language: language.to_string(),
            rainbow_brackets,
            reply,
        })
    }

    fn load_named_language(&self, name: &str) -> Result<(), RuntimeError> {
        self.submit(|reply| Job::LoadNamed {
            name: name.to_string(),
            reply,
        })
    }
}

/// The store the `:lumis` application persists under, resolved the same way.
///
/// `installed_dirs` is always `Some`: an Elixir project declares the languages
/// it may render by depending on them, so the store never reaches past that set
/// and the fetcher it would reach through refuses rather than going unused.
fn language_store() -> store::LanguageStore {
    let paths = STORE_PATHS.read();
    let cache_dir = store::resolve_data_dir(paths.data_dir.clone());
    lumis_wasm_runtime::set_compile_cache_dir(cache_dir.clone());

    store::LanguageStore::new(
        store::StoreConfig {
            cache_dir,
            installed_dirs: Some(paths.installed_dirs.clone()),
        },
        Box::new(store::NoNetwork),
    )
}

pub fn executor() -> Result<&'static Executor> {
    EXECUTOR.as_ref().map_err(|error| anyhow!("{error:#}"))
}

/// Point the store at the directories the `:lumis` application reported.
///
/// Answers `false` once the executor has been built, because the store it was
/// given cannot be swapped underneath it.
#[rustler::nif]
fn configure_lumis_store(data_dir: Option<String>, installed_dirs: Vec<String>) -> bool {
    if Lazy::get(&EXECUTOR).is_some() {
        return false;
    }

    let mut paths = STORE_PATHS.write();
    paths.data_dir = data_dir.map(PathBuf::from);
    paths.installed_dirs = installed_dirs.into_iter().map(PathBuf::from).collect();

    true
}

/// Compile a parser ahead of the first render.
///
/// A cold parser costs a Wasmtime compile, slow enough to be worth moving off a
/// request. The bytes are already on disk — they arrived as a dependency.
#[rustler::nif(schedule = "DirtyCpu")]
fn load_lumis_language(name: &str) -> bool {
    executor()
        .map(|executor| executor.load_named_language(name).is_ok())
        .unwrap_or(false)
}
