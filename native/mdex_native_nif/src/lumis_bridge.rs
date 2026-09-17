//! Reaching Lumis's highlighter without linking a second copy of it.
//!
//! The `:lumis` NIF exposes its executor through a resource whose `dyncall`
//! callback this module invokes with `enif_dynamic_resource_call`. Only plain
//! C types cross: neither NIF can see the other's Rust, and neither frees what
//! the other allocated. The V1 `HighlightCall` and `EventC` layouts are spelled
//! identically in `lumis_nif` and are frozen from the first Lumis release that
//! ships the resource; after that a different layout gets a new resource and
//! function name. Until then any change to either struct also bumps
//! `BRIDGE_ABI`, so a stale local build fails loudly instead of misreading.

use std::ffi::c_void;

use lumis_core::events::HighlightEvent;
use rustler::sys::{enif_dynamic_resource_call, ErlNifEnv, ERL_NIF_TERM};
use rustler::{Env, Term};

/// Identifies a valid V1 call; it does not negotiate a different struct layout.
const BRIDGE_ABI: u32 = 1;
const UNKNOWN_SCOPE_INDEX: usize = usize::MAX;

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
    scope: *const u8,
    scope_len: usize,
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
    /// `0` or `1`; a `u8` so no byte is an invalid value across the boundary.
    rainbow_brackets: u8,
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
    /// `nil` when the caller has no Lumis to offer, which must not be published
    /// as if it were a resource: dyncall would refuse it and report a bridge
    /// failure where the real answer is that there is no bridge.
    pub fn new(env: Env<'_>, resource: Term<'_>) -> Self {
        if !resource.is_atom() {
            ACTIVE.set(Some((env.as_c_arg(), resource.as_c_arg())));
        }
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
    let mut sink = Sink::default();

    let mut call = HighlightCall {
        abi: BRIDGE_ABI,
        source: source.as_ptr(),
        source_len: source.len(),
        language: language.as_ptr(),
        language_len: language.len(),
        rainbow_brackets: u8::from(rainbow_brackets),
        sink: Some(collect),
        sink_ctx: std::ptr::from_mut(&mut sink).cast::<c_void>(),
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

    if sink.panicked {
        return Err(BridgeError::Other(
            "mdex_native panicked while collecting Lumis events".to_string(),
        ));
    }

    match call.status {
        STATUS_OK => Ok(sink.events),
        STATUS_LANGUAGE_NOT_LOADED => Err(BridgeError::LanguageNotLoaded(read_error(&call))),
        _ => Err(BridgeError::Other(read_error(&call))),
    }
}

/// What the provider pushes into, owned by this side for the length of one call.
#[derive(Default)]
struct Sink {
    events: Vec<HighlightEvent<'static>>,
    /// Set instead of unwinding: a panic leaving `collect` would cross
    /// `extern "C"` into the provider's frames, which aborts the VM.
    panicked: bool,
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
    let sink = &mut *ctx.cast::<Sink>();

    let pushed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        push_event(&mut sink.events, event);
    }));

    if pushed.is_err() {
        sink.panicked = true;
    }
}

unsafe fn push_event(events: &mut Vec<HighlightEvent<'static>>, event: EventC) {
    events.push(match event.kind {
        EVENT_START => HighlightEvent::Start {
            scope_index: scope_index(event.scope, event.scope_len),
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

unsafe fn scope_index(scope: *const u8, scope_len: usize) -> usize {
    if scope.is_null() {
        return UNKNOWN_SCOPE_INDEX;
    }

    std::str::from_utf8(std::slice::from_raw_parts(scope, scope_len))
        .ok()
        .and_then(|scope| {
            lumis_core::highlights::HIGHLIGHT_NAMES
                .binary_search(&scope)
                .ok()
        })
        .unwrap_or(UNKNOWN_SCOPE_INDEX)
}

#[cfg(test)]
mod tests {
    use super::{
        collect, EventC, HighlightCall, Sink, BRIDGE_ABI, EVENT_START, UNKNOWN_SCOPE_INDEX,
    };
    use lumis_core::events::HighlightEvent;
    use std::ffi::c_void;
    use std::mem::{offset_of, size_of};

    fn start_event(scope: &str) -> EventC {
        EventC {
            kind: EVENT_START,
            scope: scope.as_ptr(),
            scope_len: scope.len(),
            start: 0,
            end: 0,
            language: b"elixir".as_ptr(),
            language_len: b"elixir".len(),
        }
    }

    fn collect_event(event: EventC) -> HighlightEvent<'static> {
        let mut sink = Sink::default();
        unsafe {
            collect(std::ptr::from_mut(&mut sink).cast::<c_void>(), event);
        }
        assert!(!sink.panicked);
        sink.events.pop().unwrap()
    }

    fn scope_of(event: EventC) -> usize {
        match collect_event(event) {
            HighlightEvent::Start { scope_index, .. } => scope_index,
            other => panic!("expected a Start event, got {other:?}"),
        }
    }

    #[test]
    fn a_null_name_is_unknown() {
        let mut event = start_event("function");
        event.scope = std::ptr::null();
        event.scope_len = 0;

        assert_eq!(scope_of(event), UNKNOWN_SCOPE_INDEX);
    }

    #[test]
    fn an_empty_name_is_unknown() {
        assert_eq!(scope_of(start_event("")), UNKNOWN_SCOPE_INDEX);
    }

    #[test]
    fn a_name_that_is_not_utf8_is_unknown() {
        let bytes: &[u8] = b"\xff\xfe";
        let mut event = start_event("function");
        event.scope = bytes.as_ptr();
        event.scope_len = bytes.len();

        assert_eq!(scope_of(event), UNKNOWN_SCOPE_INDEX);
    }

    #[test]
    fn resolves_provider_scope_names_against_the_consumer_table() {
        let event = collect_event(start_event("function"));
        let function = lumis_core::highlights::HIGHLIGHT_NAMES
            .binary_search(&"function")
            .unwrap();

        assert_eq!(
            event,
            HighlightEvent::Start {
                scope_index: function,
                language: "elixir".to_string(),
            }
        );
    }

    #[test]
    fn leaves_a_future_provider_scope_unstyled() {
        let event = collect_event(start_event("future.scope"));

        assert_eq!(
            event,
            HighlightEvent::Start {
                scope_index: UNKNOWN_SCOPE_INDEX,
                language: "elixir".to_string(),
            }
        );
        assert_eq!(event.scope(), None);
    }

    #[test]
    fn consumer_scope_names_remain_sorted_for_binary_search() {
        assert!(lumis_core::highlights::HIGHLIGHT_NAMES
            .windows(2)
            .all(|names| names[0] < names[1]));
    }

    #[test]
    fn v1_layout_is_frozen() {
        assert_eq!(BRIDGE_ABI, 1);

        let event_layout = [
            size_of::<EventC>(),
            offset_of!(EventC, kind),
            offset_of!(EventC, scope),
            offset_of!(EventC, scope_len),
            offset_of!(EventC, start),
            offset_of!(EventC, end),
            offset_of!(EventC, language),
            offset_of!(EventC, language_len),
        ];
        let call_layout = [
            size_of::<HighlightCall>(),
            offset_of!(HighlightCall, abi),
            offset_of!(HighlightCall, source),
            offset_of!(HighlightCall, source_len),
            offset_of!(HighlightCall, language),
            offset_of!(HighlightCall, language_len),
            offset_of!(HighlightCall, rainbow_brackets),
            offset_of!(HighlightCall, sink),
            offset_of!(HighlightCall, sink_ctx),
            offset_of!(HighlightCall, status),
            offset_of!(HighlightCall, error),
            offset_of!(HighlightCall, error_len),
        ];

        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(event_layout, [56, 0, 8, 16, 24, 32, 40, 48]);
            assert_eq!(call_layout, [88, 0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80]);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(event_layout, [28, 0, 4, 8, 12, 16, 20, 24]);
            assert_eq!(call_layout, [44, 0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40]);
        }
    }
}
