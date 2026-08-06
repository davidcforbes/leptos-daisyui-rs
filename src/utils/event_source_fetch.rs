//! Authenticated Server-Sent-Events over `fetch` (`use_event_source_fetch`).
//!
//! [`use_event_source`](super::use_event_source) owns a native `EventSource`
//! — which cannot set an `Authorization` header. An application that falls
//! back from cookies to bearer sessions must otherwise duplicate
//! fetch/ReadableStream parsing and reconnection by hand or lose live
//! updates (4iiz-Office op-rrp9). This hook is the universal primitive: the
//! same named-event + owned-lifecycle contract as `use_event_source`, but
//! the stream is a `fetch` with caller-supplied headers, parsed by a pure
//! SSE parser, with spec-shaped reconnection (`retry:` honored,
//! `Last-Event-ID` resent). Authentication *policy* stays with the caller:
//! the `headers` callback runs before every (re)connect, so a short-lived
//! token is re-read each time.
//!
//! Lifecycle is owner-bound like everything else in this family: on cleanup
//! the fetch is aborted, the loop stops, and the callbacks drop.

use super::event_source::EventSourceHandle;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::Cell;
use std::rc::Rc;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{js_sys, window};

/// One parsed SSE event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SseEvent {
    /// Event type: the last `event:` field before dispatch, or `"message"`.
    pub event: String,
    /// The data lines, joined with `\n` (per spec).
    pub data: String,
    /// The event id, if the stream set one (`id:` field).
    pub id: Option<String>,
}

/// Incremental SSE stream parser — the
/// [WHATWG event-stream grammar](https://html.spec.whatwg.org/multipage/server-sent-events.html)
/// as a pure state machine, so the wire protocol is unit-testable without a
/// browser. Feed it chunks as they arrive (any split, mid-line or mid-UTF-8
/// is fine — pair with [`take_utf8_prefix`] for byte streams); it returns
/// the events completed by each chunk.
#[derive(Debug, Default)]
pub struct SseParser {
    line_buf: String,
    event_type: String,
    data: Vec<String>,
    id: Option<String>,
    /// Last `retry:` value seen (reconnection delay in ms).
    pub retry_ms: Option<u32>,
    /// Last dispatched/seen event id — resent as `Last-Event-ID`.
    pub last_event_id: Option<String>,
}

impl SseParser {
    /// Feed a decoded chunk; returns the events it completed.
    pub fn push(&mut self, chunk: &str) -> Vec<SseEvent> {
        let mut out = Vec::new();
        for ch in chunk.chars() {
            match ch {
                '\n' => {
                    let line = std::mem::take(&mut self.line_buf);
                    // CRLF: the \r belongs to the terminator, not the line.
                    let line = line.strip_suffix('\r').unwrap_or(&line);
                    if let Some(ev) = self.take_line(line) {
                        out.push(ev);
                    }
                }
                _ => self.line_buf.push(ch),
            }
        }
        out
    }

    /// Process one complete line; a blank line may dispatch an event.
    fn take_line(&mut self, line: &str) -> Option<SseEvent> {
        if line.is_empty() {
            return self.dispatch();
        }
        // Comment lines (keep-alives) start with a colon.
        if let Some(rest) = line.strip_prefix(':') {
            let _ = rest;
            return None;
        }
        let (field, value) = match line.split_once(':') {
            // Spec: strip exactly one leading space from the value.
            Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
            None => (line, ""),
        };
        match field {
            "data" => self.data.push(value.to_string()),
            "event" => self.event_type = value.to_string(),
            // An id containing NUL is ignored per spec.
            "id" if !value.contains('\0') => {
                self.id = Some(value.to_string());
                self.last_event_id = Some(value.to_string());
            }
            "retry" => {
                if let Ok(ms) = value.parse::<u32>() {
                    self.retry_ms = Some(ms);
                }
            }
            _ => {}
        }
        None
    }

    fn dispatch(&mut self) -> Option<SseEvent> {
        let event_type = std::mem::take(&mut self.event_type);
        let data = std::mem::take(&mut self.data);
        let id = self.id.take();
        // Spec: an empty data buffer dispatches nothing (and resets type).
        if data.is_empty() {
            return None;
        }
        Some(SseEvent {
            event: if event_type.is_empty() {
                "message".to_string()
            } else {
                event_type
            },
            data: data.join("\n"),
            id,
        })
    }
}

/// Split the longest valid-UTF-8 prefix out of `buf`, leaving any trailing
/// incomplete multi-byte sequence for the next network chunk. Chunk
/// boundaries fall anywhere, including inside a character — decoding each
/// chunk independently would corrupt exactly the non-ASCII payloads a
/// localized stream carries.
pub fn take_utf8_prefix(buf: &mut Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(s) => {
            let s = s.to_string();
            buf.clear();
            s
        }
        Err(e) => {
            let valid = e.valid_up_to();
            let s = String::from_utf8_lossy(&buf[..valid]).into_owned();
            buf.drain(..valid);
            s
        }
    }
}

/// Reconnect delay: the server's `retry:` value when it sent one, else 3s
/// (the common browser default).
pub fn reconnect_delay_ms(retry_ms: Option<u32>) -> u32 {
    retry_ms.unwrap_or(3000)
}

/// A `Promise`-backed sleep, so the reconnect loop yields to the browser.
async fn sleep_ms(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(w) = window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms.min(i32::MAX as u32) as i32,
            );
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// Subscribe to a **named** SSE event on `url` over `fetch`, with
/// caller-supplied request headers — the authenticated sibling of
/// [`use_event_source`](super::use_event_source), for streams a native
/// `EventSource` cannot reach (bearer `Authorization`, custom headers).
///
/// `headers` runs before every connect *and reconnect*, so short-lived
/// credentials are re-read each attempt; issuing them is the caller's
/// business. Reconnection honors the server's `retry:` field (default 3s)
/// and resends `Last-Event-ID`. Events whose `event` type doesn't match
/// `event` are dropped (`"message"` matches unnamed events).
///
/// Lifecycle is owned by the reactive owner: unmounting aborts the in-flight
/// fetch, stops the reconnect loop, and drops the callbacks.
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::utils::use_event_source_fetch;
///
/// #[component]
/// fn Queue() -> impl IntoView {
///     let sse = use_event_source_fetch(
///         "/api/queue/stream",
///         "claimed",
///         || vec![("Authorization".to_string(), format!("Bearer {}", "token"))],
///         move |raw: String| { let _ = raw; },
///     );
///     view! { <span>{move || if sse.connected.get() { "live" } else { "reconnecting…" }}</span> }
/// }
/// ```
pub fn use_event_source_fetch(
    url: impl Into<String>,
    event: impl Into<String>,
    headers: impl Fn() -> Vec<(String, String)> + 'static,
    on_message: impl Fn(String) + 'static,
) -> EventSourceHandle {
    let connected = RwSignal::new(false);
    let handle = EventSourceHandle {
        connected: connected.into(),
        parse_errors: Signal::stored(0),
    };

    let url = url.into();
    let event = event.into();
    let stopped = Rc::new(Cell::new(false));
    let controller = web_sys::AbortController::new().ok();

    {
        let stopped = stopped.clone();
        let signal = controller.as_ref().map(|c| c.signal());
        leptos::task::spawn_local(async move {
            let mut parser = SseParser::default();
            loop {
                if stopped.get() {
                    return;
                }
                // (Re)connect with freshly-read headers.
                let init = web_sys::RequestInit::new();
                init.set_method("GET");
                if let Some(sig) = signal.as_ref() {
                    init.set_signal(Some(sig));
                }
                let req_headers = web_sys::Headers::new().ok();
                if let Some(h) = req_headers.as_ref() {
                    let _ = h.set("Accept", "text/event-stream");
                    if let Some(id) = parser.last_event_id.as_deref() {
                        let _ = h.set("Last-Event-ID", id);
                    }
                    for (k, v) in headers() {
                        let _ = h.set(&k, &v);
                    }
                    init.set_headers(h);
                }

                let attempt: Result<(), JsValue> = async {
                    let w = window().ok_or_else(|| JsValue::from_str("no window"))?;
                    let resp = wasm_bindgen_futures::JsFuture::from(
                        w.fetch_with_str_and_init(&url, &init),
                    )
                    .await?;
                    let resp: web_sys::Response = resp.dyn_into()?;
                    if !resp.ok() {
                        return Err(JsValue::from_str("non-2xx"));
                    }
                    let body = resp.body().ok_or_else(|| JsValue::from_str("no body"))?;
                    let reader: web_sys::ReadableStreamDefaultReader =
                        body.get_reader().dyn_into()?;
                    connected.set(true);
                    let mut bytes = Vec::<u8>::new();
                    loop {
                        if stopped.get() {
                            let _ = reader.cancel();
                            return Ok(());
                        }
                        let chunk = wasm_bindgen_futures::JsFuture::from(reader.read()).await?;
                        let done = js_sys::Reflect::get(&chunk, &"done".into())?
                            .as_bool()
                            .unwrap_or(true);
                        if done {
                            return Ok(());
                        }
                        let value = js_sys::Reflect::get(&chunk, &"value".into())?;
                        bytes.extend(js_sys::Uint8Array::new(&value).to_vec());
                        let text = take_utf8_prefix(&mut bytes);
                        for ev in parser.push(&text) {
                            if ev.event == event {
                                on_message(ev.data);
                            }
                        }
                    }
                }
                .await;
                let _ = attempt;

                connected.set(false);
                if stopped.get() {
                    return;
                }
                sleep_ms(reconnect_delay_ms(parser.retry_ms)).await;
            }
        });
    }

    // Owner cleanup: stop the loop and abort any in-flight fetch/read. The
    // spawned future then returns at its next checkpoint and its captures
    // (the callbacks) drop.
    let guard = SendWrapper::new((stopped, controller));
    on_cleanup(move || {
        let (stopped, controller) = guard.take();
        stopped.set(true);
        if let Some(c) = controller {
            c.abort();
        }
    });

    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(parser: &mut SseParser, chunks: &[&str]) -> Vec<SseEvent> {
        chunks.iter().flat_map(|c| parser.push(c)).collect()
    }

    // ── SseParser ──

    #[test]
    fn a_simple_event_dispatches_on_the_blank_line() {
        let mut p = SseParser::default();
        let evs = p.push("event: claimed\ndata: 24-01371\n\n");
        assert_eq!(
            evs,
            vec![SseEvent {
                event: "claimed".into(),
                data: "24-01371".into(),
                id: None
            }]
        );
    }

    #[test]
    fn unnamed_events_are_type_message() {
        let mut p = SseParser::default();
        let evs = p.push("data: hi\n\n");
        assert_eq!(evs[0].event, "message");
    }

    #[test]
    fn multi_line_data_joins_with_newline() {
        let mut p = SseParser::default();
        let evs = p.push("data: line one\ndata: line two\n\n");
        assert_eq!(evs[0].data, "line one\nline two");
    }

    #[test]
    fn chunk_boundaries_anywhere_do_not_split_events() {
        let mut p = SseParser::default();
        let evs = all(&mut p, &["ev", "ent: cla", "imed\nda", "ta: x\n", "\n"]);
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].event, "claimed");
        assert_eq!(evs[0].data, "x");
    }

    #[test]
    fn crlf_line_endings_parse_identically() {
        let mut p = SseParser::default();
        let evs = p.push("event: claimed\r\ndata: x\r\n\r\n");
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].data, "x");
    }

    #[test]
    fn comment_keepalives_dispatch_nothing() {
        let mut p = SseParser::default();
        assert!(p.push(": ping\n\n").is_empty());
    }

    #[test]
    fn empty_data_buffer_dispatches_nothing_and_resets_type() {
        let mut p = SseParser::default();
        assert!(p.push("event: claimed\n\n").is_empty());
        // The type must not leak into the next event.
        let evs = p.push("data: x\n\n");
        assert_eq!(evs[0].event, "message");
    }

    #[test]
    fn id_field_sets_last_event_id_for_reconnects() {
        let mut p = SseParser::default();
        let evs = p.push("id: 41\ndata: x\n\n");
        assert_eq!(evs[0].id.as_deref(), Some("41"));
        assert_eq!(p.last_event_id.as_deref(), Some("41"));
    }

    #[test]
    fn id_with_nul_is_ignored() {
        let mut p = SseParser::default();
        p.push("id: a\0b\ndata: x\n\n");
        assert_eq!(p.last_event_id, None);
    }

    #[test]
    fn retry_field_sets_the_reconnect_delay() {
        let mut p = SseParser::default();
        p.push("retry: 250\n");
        assert_eq!(p.retry_ms, Some(250));
        assert_eq!(reconnect_delay_ms(p.retry_ms), 250);
        assert_eq!(reconnect_delay_ms(None), 3000);
    }

    #[test]
    fn value_keeps_leading_spaces_beyond_the_first() {
        let mut p = SseParser::default();
        let evs = p.push("data:  padded\n\n");
        assert_eq!(evs[0].data, " padded");
    }

    #[test]
    fn field_without_colon_is_a_field_with_empty_value() {
        // Spec: "data" alone appends an empty string.
        let mut p = SseParser::default();
        let evs = p.push("data\ndata: x\n\n");
        assert_eq!(evs[0].data, "\nx");
    }

    // ── take_utf8_prefix ──

    #[test]
    fn utf8_split_across_chunks_reassembles() {
        // "é" is 0xC3 0xA9; split between the bytes.
        let mut buf = vec![b'a', 0xC3];
        let first = take_utf8_prefix(&mut buf);
        assert_eq!(first, "a");
        assert_eq!(buf, vec![0xC3], "incomplete tail must be kept");
        buf.push(0xA9);
        assert_eq!(take_utf8_prefix(&mut buf), "é");
        assert!(buf.is_empty());
    }

    #[test]
    fn plain_ascii_drains_fully() {
        let mut buf = b"data: x\n\n".to_vec();
        assert_eq!(take_utf8_prefix(&mut buf), "data: x\n\n");
        assert!(buf.is_empty());
    }
}
