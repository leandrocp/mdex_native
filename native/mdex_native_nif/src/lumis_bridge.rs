//! Reaching Lumis's highlighter without linking a second copy of it.
//!
//! The `:lumis` NIF exposes its executor through a resource whose `dyncall`
//! callback this module invokes with `enif_dynamic_resource_call`. Only plain
//! C types cross: neither NIF can see the other's Rust, and neither frees what
//! the other allocated. `HighlightCall` and `EventC` are spelled identically in
//! `lumis_nif` and versioned by `abi`.

use std::ffi::c_void;

use lumis_core::events::HighlightEvent;
use rustler::sys::{enif_dynamic_resource_call, ErlNifEnv, ERL_NIF_TERM};
use rustler::{Env, Term};

/// Must match `BRIDGE_ABI` in `lumis_nif`.
const BRIDGE_ABI: u32 = 1;

const EVENT_START: u8 = 0;
const EVENT_SOURCE: u8 = 1;

const STATUS_OK: i32 = 0;
const STATUS_LANGUAGE_NOT_LOADED: i32 = 1;

mod atoms {
    rustler::atoms! {
        lumis_native = "Elixir.Lumis.Native",
        mdex_bridge_v1 = "MDExBridgeV1",
    }
}

#[repr(C)]
struct EventC {
    kind: u8,
    scope_index: u32,
    start: usize,
    end: usize,
    language: *const u8,
    language_len: usize,
}

#[repr(C)]
struct HighlightCall {
    abi: u32,
    source: *const u8,
    source_len: usize,
    language: *const u8,
    language_len: usize,
    rainbow_brackets: bool,
    sink: Option<unsafe extern "C" fn(*mut c_void, EventC)>,
    sink_ctx: *mut c_void,
    status: i32,
    error: *const u8,
    error_len: usize,
}

/// Why a fence could not be highlighted, at the granularity the caller acts on.
pub enum BridgeError {
    /// Lumis has no parser for this language and could not obtain one.
    LanguageNotLoaded(String),
    Other(String),
}

thread_local! {
    /// The env and resource of the NIF call currently on this thread's stack.
    ///
    /// Comrak requires its highlighter adapter to be `Send + Sync`, which a raw
    /// `*mut ErlNifEnv` is not, and an env is only valid for its own call. Both
    /// problems go away by keeping it here rather than in the adapter: a render
    /// that somehow ran on another thread finds no bridge and errors, instead
    /// of using an env that does not belong to it.
    static ACTIVE: std::cell::Cell<Option<(*mut ErlNifEnv, ERL_NIF_TERM)>> =
        const { std::cell::Cell::new(None) };
}

/// Publishes the bridge for the length of one NIF call.
pub struct BridgeScope;

impl BridgeScope {
    pub fn new(env: Env<'_>, resource: Term<'_>) -> Self {
        ACTIVE.set(Some((env.as_c_arg(), resource.as_c_arg())));
        Self
    }
}

impl Drop for BridgeScope {
    fn drop(&mut self) {
        ACTIVE.set(None);
    }
}

/// Whether a render on this thread can reach the `:lumis` highlighter.
pub fn available() -> bool {
    ACTIVE.get().is_some()
}

/// Syntax events for `source`, produced by the `:lumis` NIF's executor.
pub fn highlight(
    source: &str,
    language: &str,
    rainbow_brackets: bool,
) -> Result<Vec<HighlightEvent<'static>>, BridgeError> {
    let Some((env, resource)) = ACTIVE.get() else {
        return Err(BridgeError::Other(
            "no :lumis bridge is active for this render".to_string(),
        ));
    };
    let mut events: Vec<HighlightEvent<'static>> = Vec::new();

    let mut call = HighlightCall {
        abi: BRIDGE_ABI,
        source: source.as_ptr(),
        source_len: source.len(),
        language: language.as_ptr(),
        language_len: language.len(),
        rainbow_brackets,
        sink: Some(collect),
        sink_ctx: std::ptr::from_mut(&mut events).cast::<c_void>(),
        status: -1,
        error: std::ptr::null(),
        error_len: 0,
    };

    let called = unsafe {
        enif_dynamic_resource_call(
            env,
            atoms::lumis_native().as_c_arg(),
            atoms::mdex_bridge_v1().as_c_arg(),
            resource,
            std::ptr::from_mut(&mut call).cast::<c_void>(),
        )
    };

    if called != 0 {
        return Err(BridgeError::Other(
            "the :lumis NIF did not accept the MDEx bridge call".to_string(),
        ));
    }

    match call.status {
        STATUS_OK => Ok(events),
        STATUS_LANGUAGE_NOT_LOADED => Err(BridgeError::LanguageNotLoaded(read_error(&call))),
        _ => Err(BridgeError::Other(read_error(&call))),
    }
}

fn read_error(call: &HighlightCall) -> String {
    if call.error.is_null() {
        return "Lumis gave no reason".to_string();
    }

    // Borrowed from the provider and only valid until its next bridge call on
    // this thread, so it is copied here rather than held.
    unsafe {
        String::from_utf8_lossy(std::slice::from_raw_parts(call.error, call.error_len)).into_owned()
    }
}

/// Called once per event, from inside Lumis's `dyncall`.
unsafe extern "C" fn collect(ctx: *mut c_void, event: EventC) {
    let events = &mut *ctx.cast::<Vec<HighlightEvent<'static>>>();

    events.push(match event.kind {
        EVENT_START => HighlightEvent::Start {
            scope_index: event.scope_index as usize,
            language: if event.language.is_null() {
                String::new()
            } else {
                String::from_utf8_lossy(std::slice::from_raw_parts(
                    event.language,
                    event.language_len,
                ))
                .into_owned()
            },
        },
        EVENT_SOURCE => HighlightEvent::Source {
            start: event.start,
            end: event.end,
        },
        _ => HighlightEvent::End,
    });
}
