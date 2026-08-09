//! SSE bridge transport for ai-chat-engine's HTTP/SSE service (beads-efgp).
//!
//! `ai_chat_core::ChatTransport` is `Send`, and its docs are explicit that a
//! browser SSE client should not drive JS handles through the trait — wasm
//! closures and `EventSource`/fetch handles are `!Send`. So the web face
//! splits the problem:
//!
//! - [`SseBridgeTransport`] implements `ChatTransport` with **std-only state**
//!   (two queues behind `Arc<Mutex<…>>`): the component's `ChatSession` drives
//!   it exactly like any other transport, and it is fully testable natively.
//! - [`SseBridgeHandle`] is the async glue's side of the bridge: an
//!   Effect/`spawn_local` drains [`SseBridgeHandle::take_commands`] into HTTP
//!   `POST`s against the service (paths from [`paths`]) and pushes every SSE
//!   frame through [`parse_sse_data`] + [`SseBridgeHandle::push_event`] —
//!   `crate::utils::use_event_source_fetch` (ldui-7b5) is the intended SSE
//!   reader.
//!
//! The wire contract is `ai_chat_core::wire` (relocated from the engine so
//! this crate never depends on it): open a session with
//! `wire::OpenSessionRequest` (picking a backend id from
//! [`ai_chat_core::Capabilities`], e.g. fetched from
//! [`paths::session_capabilities`]), then stream `GET …/events`, where each
//! frame's `data:` is one serialized [`StreamEvent`].
//!
//! ```ignore
//! let (transport, handle) = SseBridgeTransport::new();
//! let session = StoredValue::new_local(ChatSession::new(Box::new(transport)));
//! // async glue (Effect + spawn_local, app-specific auth):
//! //   for cmd in handle.take_commands() { POST the matching wire request }
//! //   on each SSE frame: parse_sse_data(&data).map(|ev| handle.push_event(ev));
//! view! { <AiChat session=session /> }
//! ```

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use ai_chat_core::{ChatError, ChatRequest, ChatSettings, ChatTransport, StreamEvent, WakeFn};

/// Service URL builders for ai-chat-engine's HTTP/SSE routes. `base` is the
/// service origin (no trailing slash), e.g. `http://127.0.0.1:7317`.
pub mod paths {
    /// `POST` — open a session (`wire::OpenSessionRequest` →
    /// `wire::OpenSessionResponse`).
    pub fn open_session(base: &str) -> String {
        format!("{base}/session")
    }
    /// `POST` — send a turn (`wire::SendRequest`).
    pub fn session_send(base: &str, id: &str) -> String {
        format!("{base}/session/{id}/send")
    }
    /// `GET` (SSE) — stream the session's `StreamEvent`s.
    pub fn session_events(base: &str, id: &str) -> String {
        format!("{base}/session/{id}/events")
    }
    /// `GET` — the session backend's `ai_chat_core::Capabilities` JSON.
    pub fn session_capabilities(base: &str, id: &str) -> String {
        format!("{base}/session/{id}/capabilities")
    }
    /// `GET` — `wire::SessionSnapshot` (transcript + waiting flag).
    pub fn session_snapshot(base: &str, id: &str) -> String {
        format!("{base}/session/{id}")
    }
    /// `POST` — interrupt the in-flight turn.
    pub fn session_cancel(base: &str, id: &str) -> String {
        format!("{base}/session/{id}/cancel")
    }
    /// `POST` — clear/respawn the session.
    pub fn session_restart(base: &str, id: &str) -> String {
        format!("{base}/session/{id}/restart")
    }
    /// `POST` — apply `ChatSettings`.
    pub fn session_configure(base: &str, id: &str) -> String {
        format!("{base}/session/{id}/configure")
    }
}

/// Parse one SSE frame's `data:` payload into a [`StreamEvent`]. The service
/// writes `data: <StreamEvent JSON>\n\n` per event; anything unparseable
/// (comments, keep-alives, foreign frames) is `None` rather than an error so
/// a stream survives protocol additions.
pub fn parse_sse_data(data: &str) -> Option<StreamEvent> {
    serde_json::from_str(data.trim()).ok()
}

/// One transport action the async glue must relay to the service.
#[derive(Clone, Debug, PartialEq)]
pub enum BridgeCommand {
    /// `POST` [`paths::session_send`] with `wire::SendRequest { prompt }`.
    Send(ChatRequest),
    /// `POST` [`paths::session_restart`].
    Restart,
    /// `POST` [`paths::session_cancel`].
    Cancel,
    /// `POST` [`paths::session_configure`] with the settings JSON.
    Configure(ChatSettings),
}

#[derive(Default)]
struct Shared {
    commands: Mutex<VecDeque<BridgeCommand>>,
    events: Mutex<VecDeque<StreamEvent>>,
    wake: Mutex<Option<WakeFn>>,
}

/// The `ChatTransport` half of the SSE bridge — std-only, `Send`, and
/// natively testable. See the module docs for the wiring picture.
pub struct SseBridgeTransport {
    shared: Arc<Shared>,
}

/// The async glue's half: drain commands, push events. Cloneable so the SSE
/// reader and the command pump can hold their own.
#[derive(Clone)]
pub struct SseBridgeHandle {
    shared: Arc<Shared>,
}

impl SseBridgeTransport {
    /// Create the connected pair.
    pub fn new() -> (Self, SseBridgeHandle) {
        let shared = Arc::new(Shared::default());
        (
            Self {
                shared: shared.clone(),
            },
            SseBridgeHandle { shared },
        )
    }

    fn push_command(&self, cmd: BridgeCommand) {
        self.shared
            .commands
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(cmd);
    }
}

impl ChatTransport for SseBridgeTransport {
    fn send(&mut self, req: ChatRequest) -> Result<(), ChatError> {
        self.push_command(BridgeCommand::Send(req));
        Ok(())
    }

    fn try_recv(&mut self) -> Option<StreamEvent> {
        self.shared
            .events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pop_front()
    }

    fn restart(&mut self) -> Result<(), ChatError> {
        // Locally queued turns are void once the session restarts.
        self.shared
            .events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.push_command(BridgeCommand::Restart);
        Ok(())
    }

    fn cancel(&mut self) -> Result<(), ChatError> {
        self.push_command(BridgeCommand::Cancel);
        Ok(())
    }

    fn configure(&mut self, settings: ChatSettings) {
        self.push_command(BridgeCommand::Configure(settings));
    }

    fn set_wake(&mut self, wake: WakeFn) {
        *self.shared.wake.lock().unwrap_or_else(|e| e.into_inner()) = Some(wake);
    }
}

impl SseBridgeHandle {
    /// Take every queued command (FIFO) for relaying to the service.
    pub fn take_commands(&self) -> Vec<BridgeCommand> {
        self.shared
            .commands
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .drain(..)
            .collect()
    }

    /// Whether any command is waiting (poll cheaply before an async hop).
    pub fn has_commands(&self) -> bool {
        !self
            .shared
            .commands
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }

    /// Enqueue a received event for the session and fire the wake callback.
    pub fn push_event(&self, ev: StreamEvent) {
        self.shared
            .events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(ev);
        let wake = self
            .shared
            .wake
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let Some(w) = wake {
            w();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn send_restart_cancel_configure_queue_commands_in_order() {
        let (mut t, h) = SseBridgeTransport::new();
        t.send(ChatRequest {
            prompt: "hi".into(),
            ..Default::default()
        })
        .unwrap();
        t.cancel().unwrap();
        t.configure(ChatSettings::default());
        t.restart().unwrap();
        let cmds = h.take_commands();
        assert!(matches!(&cmds[0], BridgeCommand::Send(r) if r.prompt == "hi"));
        assert!(matches!(cmds[1], BridgeCommand::Cancel));
        assert!(matches!(cmds[2], BridgeCommand::Configure(_)));
        assert!(matches!(cmds[3], BridgeCommand::Restart));
        assert!(h.take_commands().is_empty(), "drain is destructive");
    }

    #[test]
    fn events_flow_handle_to_transport_and_wake_fires() {
        let (mut t, h) = SseBridgeTransport::new();
        let woke = Arc::new(AtomicUsize::new(0));
        let w = woke.clone();
        t.set_wake(Arc::new(move || {
            w.fetch_add(1, Ordering::SeqCst);
        }));
        h.push_event(StreamEvent::TextDelta("a".into()));
        h.push_event(StreamEvent::Done);
        assert_eq!(woke.load(Ordering::SeqCst), 2, "wake fires per event");
        assert_eq!(t.try_recv(), Some(StreamEvent::TextDelta("a".into())));
        assert_eq!(t.try_recv(), Some(StreamEvent::Done));
        assert_eq!(t.try_recv(), None);
    }

    #[test]
    fn restart_discards_undelivered_events() {
        let (mut t, h) = SseBridgeTransport::new();
        h.push_event(StreamEvent::TextDelta("stale".into()));
        t.restart().unwrap();
        assert_eq!(t.try_recv(), None, "stale events dropped on restart");
    }

    #[test]
    fn sse_data_parses_the_services_stream_event_json() {
        // The service writes `data: <serde_json(StreamEvent)>` frames.
        let json = serde_json::to_string(&StreamEvent::TextDelta("hi".into())).unwrap();
        assert_eq!(
            parse_sse_data(&json),
            Some(StreamEvent::TextDelta("hi".into()))
        );
        assert_eq!(parse_sse_data("  \n"), None, "keep-alives are skipped");
        assert_eq!(parse_sse_data("not json"), None);
    }

    #[test]
    fn capabilities_json_deserializes_for_the_backend_picker() {
        // GET /session/{id}/capabilities serves the backend's Capabilities.
        let j = r#"{"id":"codex-cli","label":"Codex CLI (OpenAI)","needs_api_key":false,
                    "models":["gpt-5.6"],"permission_modes":[],
                    "supports_thinking":true,"supports_tool_calls":true}"#;
        let c: ai_chat_core::Capabilities = serde_json::from_str(j).unwrap();
        assert_eq!(c.label, "Codex CLI (OpenAI)");
        assert!(
            c.permission_modes.is_empty(),
            "Codex is sandboxed, no modes"
        );
    }

    #[test]
    fn wire_requests_build_against_the_service_paths() {
        use ai_chat_core::wire::{DocumentInput, OpenSessionRequest, SendRequest};
        let open = OpenSessionRequest {
            document: DocumentInput::Folder {
                path: String::new(),
            },
            settings: Default::default(),
            backend: Some("codex-cli".into()),
        };
        // Round-trips (the service deserializes exactly this shape)…
        let back: OpenSessionRequest =
            serde_json::from_str(&serde_json::to_string(&open).unwrap()).unwrap();
        assert_eq!(open, back);
        let _ = serde_json::to_string(&SendRequest { prompt: "p".into() }).unwrap();
        // …against these routes.
        assert_eq!(paths::open_session("http://x"), "http://x/session");
        assert_eq!(
            paths::session_send("http://x", "s1"),
            "http://x/session/s1/send"
        );
        assert_eq!(
            paths::session_events("http://x", "s1"),
            "http://x/session/s1/events"
        );
        assert_eq!(
            paths::session_capabilities("http://x", "s1"),
            "http://x/session/s1/capabilities"
        );
    }

    /// The whole point of the bridge: it drives a real `ChatSession`.
    #[test]
    fn a_chat_session_runs_over_the_bridge() {
        use ai_chat_core::ChatSession;
        let (t, h) = SseBridgeTransport::new();
        let mut s = ChatSession::new(Box::new(t));
        s.send(ChatRequest {
            prompt: "hello".into(),
            ..Default::default()
        })
        .unwrap();
        let cmds = h.take_commands();
        assert!(matches!(&cmds[0], BridgeCommand::Send(r) if r.prompt == "hello"));
        h.push_event(StreamEvent::TextDelta("world".into()));
        h.push_event(StreamEvent::Done);
        s.poll();
        let msgs = s.messages();
        assert!(
            msgs.iter().any(|m| m.content.contains("world")),
            "assistant reply reached the transcript: {msgs:?}"
        );
    }
}
