//! The Lumis WASM runtime this NIF highlights with.
//!
//! Parsers are WebAssembly modules resolved and cached under a data directory,
//! not grammars compiled into this library. The `:lumis` application resolves
//! that directory and [`configure_store`] points this runtime at the same one,
//! so a parser is downloaded and compiled once for the whole VM rather than
//! once per NIF.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use lumis_wasm_runtime::{store, Executor};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

static DATA_DIR: Lazy<RwLock<Option<PathBuf>>> = Lazy::new(|| RwLock::new(None));

static EXECUTOR: Lazy<Result<Executor>> = Lazy::new(|| {
    let cache_dir = store::resolve_data_dir(DATA_DIR.read().clone());
    lumis_wasm_runtime::set_compile_cache_dir(cache_dir.clone());

    Executor::new(store::LanguageStore::new(
        store::StoreConfig { cache_dir },
        Box::new(store::HttpFetcher),
    ))
    .context("could not start the Lumis WASM executor")
});

pub fn executor() -> Result<&'static Executor> {
    EXECUTOR.as_ref().map_err(|error| anyhow!("{error:#}"))
}

/// Point the store at the directory `Lumis.data_dir/0` reported.
///
/// Answers `false` once the executor has been built, because the store it was
/// given cannot be swapped underneath it.
#[rustler::nif]
fn configure_lumis_store(data_dir: Option<String>) -> bool {
    if Lazy::get(&EXECUTOR).is_some() {
        return false;
    }

    *DATA_DIR.write() = data_dir.map(PathBuf::from);
    true
}

/// Resolve, download, verify and compile a parser ahead of the first render.
///
/// A cold parser costs a download and a Wasmtime compile; both are slow enough
/// to be worth moving off a request.
#[rustler::nif(schedule = "DirtyCpu")]
fn load_lumis_language(name: &str) -> bool {
    executor()
        .map(|executor| executor.load_named_language(name).is_ok())
        .unwrap_or(false)
}
