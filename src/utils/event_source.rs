//! Owned-lifecycle Server-Sent-Events subscription (`use_event_source`).
//!
//! Screen-local SSE glue kept leaking: registering an `EventSource` listener
//! by hand forces the screen to own a `wasm_bindgen` `Closure`, and the path
//! of least resistance is `Closure::forget()` — one leaked callback
//! allocation per mount, forever (the 4iiz-Office queue shipped exactly
//! that). This hook is the framework home for the pattern: it registers a
//! named event, and on owner cleanup closes the stream, unregisters the
//! listener, and **drops** the Rust callback. Screens supply only a payload
//! callback — never `Closure::forget()`, never hand-rolled lifecycle glue.
//!
//! Same `SendWrapper` cleanup rationale as `DataTable`'s `ResizeObserver`
//! and `toolbar::Toolbar`: the JS values are neither `Send` nor `Sync`, but
//! `on_cleanup` requires both; this crate only ever runs single-threaded
//! (wasm32 in the browser), and `SendWrapper` encodes that assumption
//! explicitly.

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde::de::DeserializeOwned;
use web_sys::wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{EventSource, MessageEvent};

/// Handle returned by [`use_event_source`] / [`use_event_source_json`].
#[derive(Clone, Copy)]
pub struct EventSourceHandle {
    /// Whether the stream is currently open. `false` before the first `open`
    /// event, after an `error` (the browser auto-reconnects; this flips back
    /// on the next `open`), and permanently when construction failed (no
    /// browser, bad URL).
    pub connected: Signal<bool>,
    /// Number of payloads that failed to deserialize
    /// ([`use_event_source_json`] only; always 0 for the raw variant).
    /// Nonzero here means the client's payload type drifted from what the
    /// server sends.
    pub parse_errors: Signal<u32>,
}

/// Subscribe to a **named** SSE event on `url`, delivering each payload's
/// raw `data` string to `on_message`. See [`use_event_source_json`] for the
/// typed variant.
///
/// Lifecycle is owned by the current reactive owner: when the component that
/// called this unmounts, the stream is closed, the listener unregistered,
/// and the Rust callback dropped. Mount/unmount as often as you like — no
/// allocation outlives the screen.
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::utils::use_event_source;
///
/// #[component]
/// fn Queue() -> impl IntoView {
///     let claimed = RwSignal::new(Vec::<String>::new());
///     let sse = use_event_source("/api/queue/stream", "claimed", move |raw: String| {
///         claimed.update(|v| v.push(raw));
///     });
///     view! {
///         <span>{move || if sse.connected.get() { "live" } else { "reconnecting…" }}</span>
///     }
/// }
/// ```
pub fn use_event_source(
    url: impl Into<String>,
    event: impl Into<String>,
    on_message: impl Fn(String) + 'static,
) -> EventSourceHandle {
    let connected = RwSignal::new(false);
    let handle = EventSourceHandle {
        connected: connected.into(),
        parse_errors: Signal::stored(0),
    };

    let url = url.into();
    let event = event.into();

    // Construction fails outside a browser (or on a malformed URL). A dead
    // handle beats a panic: the screen renders, `connected` just stays false.
    let Ok(es) = EventSource::new(&url) else {
        return handle;
    };

    let on_msg = Closure::wrap(Box::new(move |ev: MessageEvent| {
        // SSE `data` is always a string; anything else is not ours to parse.
        if let Some(s) = ev.data().as_string() {
            on_message(s);
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    if es
        .add_event_listener_with_callback(&event, on_msg.as_ref().unchecked_ref())
        .is_err()
    {
        es.close();
        return handle;
    }

    let on_open = Closure::wrap(Box::new(move |_: web_sys::Event| {
        connected.set(true);
    }) as Box<dyn FnMut(web_sys::Event)>);
    es.set_onopen(Some(on_open.as_ref().unchecked_ref()));

    let on_error = Closure::wrap(Box::new(move |_: web_sys::Event| {
        connected.set(false);
    }) as Box<dyn FnMut(web_sys::Event)>);
    es.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let guard = SendWrapper::new((es, on_msg, on_open, on_error, event));
    on_cleanup(move || {
        let (es, on_msg, on_open, on_error, event) = guard.take();
        let _ = es.remove_event_listener_with_callback(&event, on_msg.as_ref().unchecked_ref());
        es.set_onopen(None);
        es.set_onerror(None);
        es.close();
        // Dropping the `Closure`s is the point: the JS side loses its last
        // reference and the Rust callbacks are freed. `Closure::forget()`
        // never appears in this file.
        drop(on_msg);
        drop(on_open);
        drop(on_error);
    });

    handle
}

/// [`use_event_source`], deserializing each payload as JSON into `T` before
/// handing it to `on_payload`. Payloads that fail to parse are counted on
/// the handle's `parse_errors` and otherwise dropped — a malformed server
/// event must not take the subscription down.
pub fn use_event_source_json<T, F>(
    url: impl Into<String>,
    event: impl Into<String>,
    on_payload: F,
) -> EventSourceHandle
where
    T: DeserializeOwned,
    F: Fn(T) + 'static,
{
    let parse_errors = RwSignal::new(0_u32);
    let mut handle = use_event_source(url, event, move |raw: String| {
        if !dispatch_json(&raw, &on_payload) {
            parse_errors.update(|n| *n += 1);
        }
    });
    handle.parse_errors = parse_errors.into();
    handle
}

/// Parse `raw` as JSON and deliver it; `false` when it doesn't deserialize.
/// Split out of [`use_event_source_json`] so the payload contract is
/// unit-testable without a browser.
fn dispatch_json<T: DeserializeOwned>(raw: &str, on_payload: &impl Fn(T)) -> bool {
    match serde_json::from_str::<T>(raw) {
        Ok(v) => {
            on_payload(v);
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::cell::RefCell;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Claim {
        case_no: String,
        by: String,
    }

    #[test]
    fn dispatch_json_delivers_a_typed_payload() {
        let seen = RefCell::new(Vec::<Claim>::new());
        let ok = dispatch_json(r#"{"case_no":"24-01371","by":"maria"}"#, &|c: Claim| {
            seen.borrow_mut().push(c)
        });
        assert!(ok);
        assert_eq!(
            *seen.borrow(),
            vec![Claim {
                case_no: "24-01371".to_string(),
                by: "maria".to_string()
            }]
        );
    }

    #[test]
    fn dispatch_json_rejects_a_drifted_payload_without_delivering() {
        let seen = RefCell::new(0_usize);
        let ok = dispatch_json(r#"{"unexpected":"shape"}"#, &|_: Claim| {
            *seen.borrow_mut() += 1;
        });
        assert!(!ok, "a payload that fails to parse must report false");
        assert_eq!(*seen.borrow(), 0, "and must not deliver");
    }

    #[test]
    fn dispatch_json_rejects_non_json() {
        assert!(!dispatch_json("not json", &|_: Claim| {}));
    }

    #[test]
    fn dispatch_json_handles_plain_string_payloads() {
        // Servers that send bare JSON strings (e.g. just an id) still parse.
        let seen = RefCell::new(String::new());
        assert!(dispatch_json(r#""24-01371""#, &|s: String| {
            *seen.borrow_mut() = s;
        }));
        assert_eq!(*seen.borrow(), "24-01371");
    }
}
