//! Live mode: a [`ChatWorkspaceBackend`] over a locally-running
//! `editmark-server` (`http://127.0.0.1:8099` by default).
//!
//! **Opt-in, never the default, and no automated lane ever reaches it.** The
//! showcase opens on the seeded fixture; choosing Live changes only what a
//! later, explicit Connect would talk to. Selecting the mode performs ZERO
//! network I/O — see [`request_intent`], which is the one function both the
//! page and its proof read, so "selected" and "connect requested" cannot
//! drift back into being the same thing.
//!
//! The native guard `no_lane_touches_the_live_server`
//! (`src/patterns/ai_chat_workspace/tests.rs`) scans `tests/` AND `demo/src/`
//! for the live host's port and transport, and names THIS file as its one
//! exception. Nothing else under `demo/src/` may reach a live server; a page
//! a browser lane drives must never be able to.
//!
//! ## What the server actually serves
//! `editmark-server` mounts its chat API under `/api/chat` and binds
//! `127.0.0.1` with no auth; its localhost CORS allows the demo's own origin,
//! so no Trunk proxy is needed over plain http. It serves `backends`,
//! `session`, `send`, `events`, `cancel`, `restart` and a `DELETE` close —
//! and it does NOT serve `configure`, `capabilities` or `snapshot`. The three
//! absences shape this file:
//!
//! - settings are applied only at session OPEN, so a `Configure` closes and
//!   re-opens the session, and the actor is told the conversation reset
//!   rather than discovering it ([`CONFIGURE_NOTICE`]);
//! - a backend's capabilities are read pre-session from `backends`;
//! - a missed event cannot be re-read, so the transcript is the stream.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use leptos::task::spawn_local;
use leptos_daisyui_rs::components::ai_assistant_workspace::{
    AssistantAccess, AssistantCapabilities, AssistantConnection, AssistantEngine, AssistantMemory,
    AssistantMemoryEntry, AssistantPreferences, AssistantSettings, ConnectionState,
    EngineAvailability, MemoryState, SignInShape,
};
use leptos_daisyui_rs::components::ai_chat::transport::paths;
use leptos_daisyui_rs::components::ai_chat::wire::{
    DocumentInput, OpenSessionRequest, OpenSessionResponse, SendRequest,
};
use leptos_daisyui_rs::components::{
    BridgeCommand, Capabilities, ChatSettings, ChatTransport, SseBridgeHandle, SseBridgeTransport,
    StreamEvent, parse_sse_data,
};
use leptos_daisyui_rs::patterns::{
    ChatPosture, ChatWorkspaceBackend, ChatWorkspaceError, ChatWorkspaceErrorKind, CorpusQueryMode,
    CorpusScope, IngestPhase, IngestStatus, KnowledgeSelection, KnowledgeSource, MemoryDraft,
    MemoryRefusal, ModelSource, ProviderCard, ProviderTuning, RecallReceipt, TuningSchema,
    TurnRecord, WorkspaceFuture, ai_chat_settings_for_card, card_ready_for_ask,
};
use leptos_daisyui_rs::utils::use_event_source_fetch;
use wasm_bindgen::{JsCast, JsValue};

/// The base URL the page falls back to when nothing is stored and nothing is
/// typed: `editmark-server`'s own default bind address.
pub const DEFAULT_BASE: &str = "http://127.0.0.1:8099";

/// The path `editmark-server` mounts its chat API under. Appended to the
/// base once, in [`service_base`], so no route builder has to know it.
pub const SERVICE_PREFIX: &str = "/api/chat";

/// `localStorage` key for the live-mode preferences. Stores the base URL and
/// the selected backend id ONLY — never the mode, so a reload always comes
/// up on the fixture.
pub const PREFS_KEY: &str = "ldui.demo.ai_chat.live";

/// How often the command pump drains the bridge, in milliseconds.
pub const PUMP_INTERVAL_MS: u32 = 50;

/// The sentence shown when a settings change forced a session re-open.
///
/// The server applies `ChatSettings` only at session open, so applying them
/// means closing the session and opening another: the conversation on the
/// server side is gone. Saying so is the whole point — an actor who is not
/// told discovers it by asking a follow-up question that lands on an engine
/// with no memory of the exchange.
pub const CONFIGURE_NOTICE: &str =
    "Settings applied: server session re-opened (conversation reset)";

/// The name the sample working document is opened under.
pub const SAMPLE_DOC_NAME: &str = "ldui-live-sample.md";

/// The sample working document Live mode grounds its session on.
///
/// Sent as `DocumentInput::Bytes`, which is the browser's path: the server
/// writes its own working copy under its workspace, and this page never
/// names a file on the operator's disk.
pub const SAMPLE_DOC: &str = "# Live mode sample\n\nThis document is the working copy the demo's \
live session is opened against. Edit it through the chat, or ignore it and ask a general \
question.\n";

// ---------------------------------------------------------------------------
// Mode, status and intent — the pure, natively-testable core
// ---------------------------------------------------------------------------

/// Which backend the showcase page is driving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceMode {
    /// The seeded in-process fixture. Always the default, always what a
    /// reload comes back to.
    Fixture,
    /// A live host backend. Selecting it connects to nothing; see
    /// [`request_intent`].
    Live,
}

impl WorkspaceMode {
    /// The stable wire string this mode publishes on a `data-*` hook.
    ///
    /// Never `{:?}`: a `Debug` variant name is free to be renamed, a wire
    /// string is a contract a proof reads. Same idiom as
    /// `ChatWorkspaceErrorKind::as_str`.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Live => "live",
        }
    }
}

/// Where the live connection currently stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveStatus {
    /// Nothing has been attempted.
    Idle,
    /// The pre-session backend list is being fetched.
    FetchingBackends,
    /// A session is being opened.
    Opening,
    /// A session is open and streaming.
    Ready {
        /// The server's session id.
        session_id: String,
        /// The backend id the session was opened against.
        backend: String,
    },
    /// The server did not answer.
    Unreachable {
        /// The base URL that did not answer.
        base: String,
        /// A display-safe detail.
        detail: String,
    },
    /// The server answered, and advertised no backends at all.
    NoBackends,
    /// The session the page held no longer exists on the server.
    SessionLost,
}

impl LiveStatus {
    /// The stable wire string this status publishes on
    /// `data-ai-chat-live-status`.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::FetchingBackends => "fetching_backends",
            Self::Opening => "opening",
            Self::Ready { .. } => "ready",
            Self::Unreachable { .. } => "unreachable",
            Self::NoBackends => "no_backends",
            Self::SessionLost => "session_lost",
        }
    }

    /// The open session's id, when there is one.
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Ready { session_id, .. } => Some(session_id.as_str()),
            _ => None,
        }
    }
}

/// The one request Live mode ever issues on its own initiative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveRequest {
    /// `GET .../backends` — the pre-session probe a Connect starts with.
    FetchBackends,
}

/// What a (mode, connect-pressed) pair should REQUEST — the single place the
/// page and its proof agree on what selecting Live does.
///
/// Selecting Live is a choice about where a later connection would go, so it
/// produces `None`: no probe, no capabilities read, no session. Only an
/// explicit Connect while Live is selected produces a request.
pub fn request_intent(mode: WorkspaceMode, connect_pressed: bool) -> Option<LiveRequest> {
    match (mode, connect_pressed) {
        (WorkspaceMode::Live, true) => Some(LiveRequest::FetchBackends),
        _ => None,
    }
}

/// What selecting a mode must do to a session that is already live.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeSwitch {
    /// Nothing to tear down.
    Nothing,
    /// A live session is open and the page is leaving Live mode, so the
    /// session must be closed BEFORE the mode changes.
    DisconnectFirst,
}

/// Selecting Fixture while a live session is open must tear it down.
///
/// Otherwise the live backend keeps driving the workspace with its SSE reader
/// attached and its command pump running, while `data-ai-chat-mode` publishes
/// `fixture` — the page asserting, in a stable hook, the opposite of what it
/// is doing. The Disconnect control also lives inside the Live section, so it
/// would be hidden at exactly the moment it is the only way back.
pub fn mode_switch_effect(target: WorkspaceMode, connected: bool) -> ModeSwitch {
    match (target, connected) {
        (WorkspaceMode::Fixture, true) => ModeSwitch::DisconnectFirst,
        _ => ModeSwitch::Nothing,
    }
}

/// Whether the page is driving the LIVE backend — the single rule the
/// workspace mount and the `data-ai-chat-mode` attribute both read.
///
/// Both halves are required. A session that is connected is not being driven
/// once the actor has selected Fixture, and a mode that says Live means
/// nothing until a session actually exists.
pub fn drives_live(mode: WorkspaceMode, connected: bool) -> bool {
    mode == WorkspaceMode::Live && connected
}

/// Whether a Connect that reached this status may replace the fixture
/// workspace with the live one.
///
/// ONLY [`LiveStatus::Ready`]. A Connect makes two requests — the backend
/// list, then the session open — and promoting on the first unmounts the
/// fixture for a Connect that has not succeeded yet: kill the server between
/// the two, or have the open rejected, and the page is left showing a live
/// workspace with no session and no fixture behind it.
pub fn promote_on(status: &LiveStatus) -> bool {
    matches!(status, LiveStatus::Ready { .. })
}

/// Normalizes an operator-typed base URL to an origin this page can build
/// routes against: a blank entry falls back to [`DEFAULT_BASE`], a bare
/// `host:port` gains `http://`, a trailing slash is trimmed, and any path
/// the operator deliberately typed is preserved (a reverse proxy may mount
/// the server under one).
pub fn normalize_base(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return DEFAULT_BASE.to_owned();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("http://{trimmed}")
    }
}

/// The chat API's own base: the origin plus [`SERVICE_PREFIX`], which is what
/// every `paths::` builder is handed.
pub fn service_base(base: &str) -> String {
    format!("{}{SERVICE_PREFIX}", normalize_base(base))
}

/// The two things Live mode remembers between reloads. The MODE is
/// deliberately absent: a reload always comes up on the fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivePrefs {
    /// The base URL last used.
    pub base: String,
    /// The backend id last selected, when one was.
    pub backend: Option<String>,
}

impl Default for LivePrefs {
    fn default() -> Self {
        Self {
            base: DEFAULT_BASE.to_owned(),
            backend: None,
        }
    }
}

/// Reads a stored preferences payload, ignoring every field but `base` and
/// `backend`.
///
/// A payload carrying a `mode` (an older serialization, or a hand-edited
/// one) is not an error and is not honoured: the mode is not a preference,
/// so there is nothing to restore it into.
pub fn parse_prefs(json: &str) -> LivePrefs {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return LivePrefs::default();
    };
    let base = value
        .get("base")
        .and_then(|b| b.as_str())
        .map(normalize_base)
        .unwrap_or_else(|| DEFAULT_BASE.to_owned());
    let backend = value
        .get("backend")
        .and_then(|b| b.as_str())
        .filter(|b| !b.is_empty())
        .map(str::to_owned);
    LivePrefs { base, backend }
}

/// Serializes the two remembered fields, and only those.
pub fn prefs_json(prefs: &LivePrefs) -> String {
    serde_json::json!({ "base": prefs.base, "backend": prefs.backend }).to_string()
}

/// Reads the stored preferences. Every failure path — no window, storage
/// blocked, a private window, unparseable content — yields the defaults
/// rather than propagating, because the page has to render either way.
pub fn load_prefs() -> LivePrefs {
    let stored = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(PREFS_KEY).ok().flatten());
    match stored {
        Some(json) => parse_prefs(&json),
        None => LivePrefs::default(),
    }
}

/// Stores the preferences, silently doing nothing when storage is
/// unavailable.
pub fn store_prefs(prefs: &LivePrefs) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = storage.set_item(PREFS_KEY, &prefs_json(prefs));
    }
}

// ---------------------------------------------------------------------------
// HTTP
// ---------------------------------------------------------------------------

/// One HTTP outcome, kept typed so a 404 (the session is gone) is never
/// confused with "the server did not answer".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpFailure {
    /// The request never reached an answer.
    Unreachable(String),
    /// The server answered that the session does not exist.
    SessionGone,
    /// The server answered with some other non-2xx status.
    Status(u16),
}

impl HttpFailure {
    /// A display-safe detail for a banner or a transcript row.
    pub fn detail(&self) -> String {
        match self {
            Self::Unreachable(d) => d.clone(),
            Self::SessionGone => "the session no longer exists".to_owned(),
            Self::Status(code) => format!("HTTP {code}"),
        }
    }
}

fn js_detail(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            value
                .dyn_ref::<js_sys::Error>()
                .map(|e| String::from(e.message()))
        })
        .unwrap_or_else(|| "the request failed".to_owned())
}

/// One JSON request against the live server. `body` is a serialized payload
/// sent as `application/json`; `None` sends no body at all.
async fn request(method: &str, url: &str, body: Option<String>) -> Result<String, HttpFailure> {
    let window = web_sys::window()
        .ok_or_else(|| HttpFailure::Unreachable("no browser window".to_owned()))?;
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    if let Some(payload) = body {
        let headers =
            web_sys::Headers::new().map_err(|e| HttpFailure::Unreachable(js_detail(e)))?;
        let _ = headers.set("Content-Type", "application/json");
        init.set_headers(&headers);
        init.set_body(&JsValue::from_str(&payload));
    }
    let response = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str_and_init(url, &init))
        .await
        .map_err(|e| HttpFailure::Unreachable(js_detail(e)))?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|_| HttpFailure::Unreachable("not an HTTP response".to_owned()))?;
    if response.status() == 404 {
        return Err(HttpFailure::SessionGone);
    }
    if !response.ok() {
        return Err(HttpFailure::Status(response.status()));
    }
    let text = response
        .text()
        .map_err(|e| HttpFailure::Unreachable(js_detail(e)))?;
    let text = wasm_bindgen_futures::JsFuture::from(text)
        .await
        .map_err(|e| HttpFailure::Unreachable(js_detail(e)))?;
    Ok(text.as_string().unwrap_or_default())
}

/// `GET .../backends` — the pre-session probe. The ONLY request a Connect
/// makes before anything is opened, and the only one this module issues
/// outside an open session.
pub async fn fetch_backends(base: &str) -> Result<Vec<Capabilities>, HttpFailure> {
    let body = request("GET", &paths::backends(&service_base(base)), None).await?;
    serde_json::from_str(&body)
        .map_err(|e| HttpFailure::Unreachable(format!("unreadable backend list: {e}")))
}

/// A `Promise`-backed sleep, so the command pump yields to the browser
/// between drains instead of spinning.
async fn sleep_ms(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms.min(i32::MAX as u32) as i32,
            );
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

// ---------------------------------------------------------------------------
// The backend
// ---------------------------------------------------------------------------

/// The live session's mutable state. One session at a time; a re-open
/// replaces the id and the SSE reader while KEEPING the transport, so the
/// composite's `ChatSession` survives a settings change.
#[derive(Default)]
struct LiveSession {
    id: Option<String>,
    handle: Option<SseBridgeHandle>,
    /// The owner the SSE reader was mounted under, keyed to the session id:
    /// disposing it stops that stream, which is what makes a re-open follow
    /// the new session instead of listening to a dead one.
    sse: Option<Owner>,
    settings: ChatSettings,
}

struct LiveCore {
    base: String,
    service: String,
    backends: Vec<Capabilities>,
    status: RwSignal<LiveStatus>,
    notice: RwSignal<Option<String>>,
    /// The page's reactive owner, so every SSE reader is mounted as its
    /// child and dies with the page.
    owner: Owner,
    session: RefCell<LiveSession>,
    pump: Cell<bool>,
    /// The transport of a session opened by an explicit `connect`, waiting
    /// for the composite to adopt it on its first `open_session`.
    ///
    /// This is what lets a failed Connect be reported BEFORE the workspace is
    /// swapped: the page opens the session itself, and the composite's own
    /// open then reuses it rather than opening a second one.
    pending: RefCell<Option<(String, Box<dyn ChatTransport>)>>,
}

/// A [`ChatWorkspaceBackend`] over a locally-running `editmark-server`.
///
/// Built only by an explicit Connect, from a backend list the server really
/// answered with. Every method that the server has no route for returns a
/// typed `Unsupported` error rather than a silent no-op or a fabricated
/// success — the composite's honesty surface renders it, and the actor finds
/// out that Live mode cannot do something at the moment they ask for it.
#[derive(Clone)]
pub struct LiveChatWorkspaceBackend {
    core: Rc<LiveCore>,
}

impl LiveChatWorkspaceBackend {
    /// Builds a backend for a server that has ALREADY answered its backend
    /// list. Taking the list rather than fetching it keeps this constructor
    /// free of I/O and keeps the probe on the explicit Connect path.
    pub fn new(
        base: &str,
        backends: Vec<Capabilities>,
        status: RwSignal<LiveStatus>,
        notice: RwSignal<Option<String>>,
        owner: Owner,
    ) -> Self {
        let base = normalize_base(base);
        let service = service_base(&base);
        Self {
            core: Rc::new(LiveCore {
                base,
                service,
                backends,
                status,
                notice,
                owner,
                session: RefCell::new(LiveSession::default()),
                pump: Cell::new(false),
                pending: RefCell::new(None),
            }),
        }
    }

    /// The engine this backend would open first — the same card the composite
    /// picks, so a session opened here is the session it goes on to adopt.
    fn first_ready_engine(&self) -> Option<ProviderCard> {
        let as_of = now_iso();
        self.core
            .backends
            .iter()
            .map(|c| card_for(c, &as_of))
            .find(card_ready_for_ask)
    }

    /// Open the first session as an EXPLICIT, page-driven action.
    ///
    /// The composite opens sessions as a consequence of being mounted, so a
    /// page that mounts it to connect has already replaced the fixture by the
    /// time an open can fail. Doing the open here inverts that: the workspace
    /// is swapped only after this returns `Ok`, and the transport is handed to
    /// the composite's own `open_session` rather than a second session being
    /// opened for it.
    pub async fn connect(&self) -> Result<String, ChatWorkspaceError> {
        let Some(card) = self.first_ready_engine() else {
            self.core.status.set(LiveStatus::NoBackends);
            return Err(ChatWorkspaceError {
                kind: ChatWorkspaceErrorKind::Unavailable,
                message: "The server advertised no engine this workspace can ask.".to_owned(),
            });
        };
        let engine_id = card.engine.id.clone();
        let transport = self
            .open_session(
                &engine_id,
                &KnowledgeSelection::default(),
                ai_chat_settings_for_card(&card),
                ProviderTuning::default(),
            )
            .await?;
        *self.core.pending.borrow_mut() = Some((engine_id.clone(), transport));
        Ok(engine_id)
    }

    /// Closes the open session and stops the pump and the SSE reader. Safe
    /// to call when nothing is open.
    pub fn disconnect(&self) {
        let core = self.core.clone();
        core.pump.set(false);
        core.pending.borrow_mut().take();
        let (id, sse) = {
            let mut session = core.session.borrow_mut();
            session.handle = None;
            (session.id.take(), session.sse.take())
        };
        if let Some(owner) = sse {
            owner.cleanup();
        }
        if let Some(id) = id {
            let url = paths::session_close(&core.service, &id);
            spawn_local(async move {
                let _ = request("DELETE", &url, None).await;
            });
        }
        core.status.set(LiveStatus::Idle);
    }

    fn ready<T: 'static>(v: Result<T, ChatWorkspaceError>) -> WorkspaceFuture<T> {
        Box::pin(std::future::ready(v))
    }

    /// The typed refusal for everything `editmark-server` has no route for.
    fn unsupported<T: 'static>(feature: &str) -> WorkspaceFuture<T> {
        Self::ready(Err(ChatWorkspaceError {
            kind: ChatWorkspaceErrorKind::Unsupported,
            message: format!("{feature} is not available in Live mode"),
        }))
    }
}

impl LiveCore {
    /// Opens a server session and returns its id.
    async fn open_remote(
        &self,
        backend_id: &str,
        settings: &ChatSettings,
    ) -> Result<String, HttpFailure> {
        let req = OpenSessionRequest {
            document: DocumentInput::Bytes {
                name: SAMPLE_DOC_NAME.to_owned(),
                bytes: SAMPLE_DOC.to_owned(),
            },
            settings: settings.clone(),
            backend: Some(backend_id.to_owned()),
        };
        let payload = serde_json::to_string(&req)
            .map_err(|e| HttpFailure::Unreachable(format!("unserializable open request: {e}")))?;
        let body = request("POST", &paths::open_session(&self.service), Some(payload)).await?;
        let opened: OpenSessionResponse = serde_json::from_str(&body)
            .map_err(|e| HttpFailure::Unreachable(format!("unreadable open response: {e}")))?;
        Ok(opened.session_id)
    }

    /// Mounts the SSE reader for one session id under its own child owner,
    /// disposing whichever stream was running before.
    fn mount_sse(self: &Rc<Self>, id: &str) {
        let previous = self.session.borrow_mut().sse.take();
        if let Some(owner) = previous {
            owner.cleanup();
        }
        let Some(handle) = self.session.borrow().handle.clone() else {
            return;
        };
        let url = paths::session_events(&self.service, id);
        let child = self.owner.with(Owner::new);
        child.with(|| {
            use_event_source_fetch(url, "message", Vec::new, move |raw: String| {
                if let Some(event) = parse_sse_data(&raw) {
                    handle.push_event(event);
                }
            });
        });
        self.session.borrow_mut().sse = Some(child);
    }

    /// Starts the command pump once; later calls are no-ops.
    fn start_pump(self: &Rc<Self>) {
        if self.pump.replace(true) {
            return;
        }
        let core = self.clone();
        spawn_local(async move {
            loop {
                if !core.pump.get() {
                    return;
                }
                let commands = core
                    .session
                    .borrow()
                    .handle
                    .as_ref()
                    .map(|h| h.take_commands())
                    .unwrap_or_default();
                for command in commands {
                    core.relay(command).await;
                }
                sleep_ms(PUMP_INTERVAL_MS).await;
            }
        });
    }

    /// Relays one bridge command to the server.
    async fn relay(self: &Rc<Self>, command: BridgeCommand) {
        let Some(id) = self.session.borrow().id.clone() else {
            return;
        };
        let outcome = match command {
            BridgeCommand::Send(req) => {
                let payload = serde_json::to_string(&SendRequest { prompt: req.prompt })
                    .unwrap_or_else(|_| "{}".to_owned());
                request(
                    "POST",
                    &paths::session_send(&self.service, &id),
                    Some(payload),
                )
                .await
                .map(|_| ())
            }
            BridgeCommand::Restart => {
                request("POST", &paths::session_restart(&self.service, &id), None)
                    .await
                    .map(|_| ())
            }
            BridgeCommand::Cancel => {
                request("POST", &paths::session_cancel(&self.service, &id), None)
                    .await
                    .map(|_| ())
            }
            BridgeCommand::Configure(settings) => self.reconfigure(settings).await,
        };
        if let Err(failure) = outcome {
            self.fail(failure);
        }
    }

    /// Applies settings the only way this server can: close the session,
    /// open another with them, and SAY that the conversation was reset.
    async fn reconfigure(self: &Rc<Self>, settings: ChatSettings) -> Result<(), HttpFailure> {
        let backend_id = match &self.status.get_untracked() {
            LiveStatus::Ready { backend, .. } => backend.clone(),
            _ => self
                .backends
                .first()
                .map(|b| b.id.clone())
                .unwrap_or_default(),
        };
        let old = self.session.borrow().id.clone();
        if let Some(id) = old {
            let _ = request("DELETE", &paths::session_close(&self.service, &id), None).await;
        }
        let id = self.open_remote(&backend_id, &settings).await?;
        {
            let mut session = self.session.borrow_mut();
            session.id = Some(id.clone());
            session.settings = settings;
        }
        self.mount_sse(&id);
        self.status.set(LiveStatus::Ready {
            session_id: id,
            backend: backend_id,
        });
        self.notice.set(Some(CONFIGURE_NOTICE.to_owned()));
        Ok(())
    }

    /// A mid-session failure: the transcript carries the error, the banner
    /// carries the state, and the page keeps rendering.
    fn fail(&self, failure: HttpFailure) {
        let detail = failure.detail();
        if let Some(handle) = self.session.borrow().handle.clone() {
            handle.push_event(StreamEvent::Error(format!(
                "Live server unreachable at {}: {detail}",
                self.base
            )));
        }
        let next = match failure {
            HttpFailure::SessionGone => LiveStatus::SessionLost,
            _ => LiveStatus::Unreachable {
                base: self.base.clone(),
                detail,
            },
        };
        self.status.set(next);
    }
}

/// The `ProviderCard` for one advertised backend.
///
/// Every listed engine is `Available`: the server answered with it, and an
/// engine it did NOT list is simply absent from the picker rather than shown
/// as broken. An empty model list becomes `ModelSource::FreeText`, never a
/// discovery claim about a list that does not exist.
pub fn card_for(capabilities: &Capabilities, as_of: &str) -> ProviderCard {
    let model_source = if capabilities.models.is_empty() {
        ModelSource::FreeText
    } else {
        ModelSource::Discovered {
            as_of: as_of.to_owned(),
        }
    };
    ProviderCard {
        engine: engine_for(capabilities),
        picker_label: capabilities.label.clone(),
        model_source,
        tuning: TuningSchema {
            reasoning_effort: false,
            temperature: false,
            codex_levers: false,
            permission_mode: !capabilities.permission_modes.is_empty(),
        },
        capabilities: capabilities.clone(),
    }
}

/// The governed engine projection for one advertised backend. The server has
/// no auth, so the connection is `NotApplicable` rather than a claimed
/// sign-in.
fn engine_for(capabilities: &Capabilities) -> AssistantEngine {
    AssistantEngine {
        id: capabilities.id.clone(),
        revision: 1,
        label: capabilities.label.clone(),
        availability: EngineAvailability::Enabled,
        connection: AssistantConnection {
            state: ConnectionState::NotApplicable,
            shape: SignInShape::Operator,
            device: None,
            account_label: None,
            verified_at: None,
            expires_at: None,
            reason: None,
        },
        capabilities: ask_grants(),
    }
}

fn ask_grants() -> AssistantCapabilities {
    use leptos_daisyui_rs::components::ai_assistant_workspace::AssistantCapability;
    AssistantCapabilities {
        granted: vec![
            AssistantCapability::Ask,
            AssistantCapability::Cancel,
            AssistantCapability::NewConversation,
            AssistantCapability::Copy,
        ],
        details: vec![],
    }
}

/// An ISO-8601 stamp from the browser clock, or an empty string off-browser.
fn now_iso() -> String {
    String::from(js_sys::Date::new_0().to_iso_string())
}

impl ChatWorkspaceBackend for LiveChatWorkspaceBackend {
    fn settings(&self) -> WorkspaceFuture<AssistantSettings> {
        let core = self.core.clone();
        let preferences = AssistantPreferences {
            engine_id: core.backends.first().map(|b| b.id.clone()),
            memory_use: false,
            memory_capture: false,
        };
        Self::ready(Ok(AssistantSettings {
            owner_actor_id: "live-operator".to_owned(),
            accepted_revision: 1,
            proposed_revision: 1,
            accepted: preferences.clone(),
            proposed: preferences,
            reasoning_tier: AssistantAccess::Granted,
            engines: core.backends.iter().map(engine_for).collect(),
            budget: None,
            // Grants nothing: this server has no settings store, no
            // sign-in and no memory service, so offering those controls
            // would be offering actions that can only fail.
            capabilities: AssistantCapabilities::default(),
        }))
    }

    fn providers(&self) -> WorkspaceFuture<Vec<ProviderCard>> {
        let as_of = now_iso();
        let cards = self
            .core
            .backends
            .iter()
            .map(|c| card_for(c, &as_of))
            .collect();
        Self::ready(Ok(cards))
    }

    fn knowledge(&self) -> WorkspaceFuture<Vec<KnowledgeSource>> {
        Self::ready(Ok(vec![KnowledgeSource::Corpus {
            scope: CorpusScope::File(SAMPLE_DOC_NAME.to_owned()),
            ingest: IngestStatus {
                phase: IngestPhase::Ready,
                files_seen: 1,
                files_indexed: 1,
                clusters: 0,
                as_of: now_iso(),
            },
            query_mode: CorpusQueryMode::FullText,
            // Assistant, not Grounded: the session is opened on one working
            // document and the engine is free to answer generally, so
            // claiming a grounded posture would promise a guarantee this
            // host does not enforce.
            posture: ChatPosture::Assistant,
        }]))
    }

    fn reindex(&self, _scope: &CorpusScope) -> WorkspaceFuture<IngestStatus> {
        Self::unsupported("Re-indexing a corpus")
    }

    fn open_session(
        &self,
        engine_id: &str,
        _knowledge: &KnowledgeSelection,
        settings: ChatSettings,
        _tuning: ProviderTuning,
    ) -> WorkspaceFuture<Box<dyn ChatTransport>> {
        let core = self.core.clone();
        let engine_id = engine_id.to_owned();
        Box::pin(async move {
            // Adopt the session an explicit Connect already opened, rather
            // than closing it and opening a second one for the same engine.
            let adopted = {
                let mut pending = core.pending.borrow_mut();
                match pending.as_ref() {
                    Some((id, _)) if *id == engine_id => pending.take().map(|(_, t)| t),
                    _ => None,
                }
            };
            if let Some(transport) = adopted {
                return Ok(transport);
            }
            core.status.set(LiveStatus::Opening);
            let previous = {
                let mut session = core.session.borrow_mut();
                session.handle = None;
                (session.id.take(), session.sse.take())
            };
            if let Some(owner) = previous.1 {
                owner.cleanup();
            }
            if let Some(id) = previous.0 {
                let _ = request("DELETE", &paths::session_close(&core.service, &id), None).await;
            }
            let id = match core.open_remote(&engine_id, &settings).await {
                Ok(id) => id,
                Err(failure) => {
                    let detail = failure.detail();
                    core.status.set(match failure {
                        HttpFailure::SessionGone => LiveStatus::SessionLost,
                        _ => LiveStatus::Unreachable {
                            base: core.base.clone(),
                            detail: detail.clone(),
                        },
                    });
                    return Err(ChatWorkspaceError {
                        kind: ChatWorkspaceErrorKind::Network,
                        message: format!("Live server unreachable at {}: {detail}", core.base),
                    });
                }
            };
            let (transport, handle) = SseBridgeTransport::new();
            {
                let mut session = core.session.borrow_mut();
                session.id = Some(id.clone());
                session.handle = Some(handle);
                session.settings = settings;
            }
            core.mount_sse(&id);
            core.start_pump();
            core.status.set(LiveStatus::Ready {
                session_id: id,
                backend: engine_id,
            });
            Ok(Box::new(transport) as Box<dyn ChatTransport>)
        })
    }

    fn set_tuning(&self, _engine_id: &str, _tuning: ProviderTuning) -> WorkspaceFuture<()> {
        // Every card this backend publishes declares an empty tuning schema
        // except permission mode, which the server takes at open; there is
        // nothing to push mid-session, so this succeeds having done nothing
        // rather than refusing a request the composite makes on every open.
        Self::ready(Ok(()))
    }

    fn turn(&self, _turn_id: &str) -> WorkspaceFuture<TurnRecord> {
        // No turn-record route exists: the transcript IS the stream. The
        // composite treats a failed `turn()` as "nothing to show" and keeps
        // rendering, so Live mode has a conversation without a lifecycle
        // ladder rather than a fabricated one.
        Self::unsupported("A turn's accepted record")
    }

    fn current_turn_id(&self) -> Option<String> {
        None
    }

    fn recall(&self, _query: &str) -> WorkspaceFuture<RecallReceipt> {
        Self::unsupported("Recall")
    }

    fn remember(
        &self,
        _draft: MemoryDraft,
    ) -> WorkspaceFuture<Result<AssistantMemoryEntry, MemoryRefusal>> {
        Self::unsupported("Writing a memory")
    }

    fn set_memory_flags(
        &self,
        _use_enabled: bool,
        _capture_enabled: bool,
    ) -> WorkspaceFuture<AssistantMemory> {
        Self::unsupported("Memory use and capture")
    }

    fn set_memory_state(
        &self,
        _id: &str,
        _state: MemoryState,
    ) -> WorkspaceFuture<AssistantMemoryEntry> {
        Self::unsupported("Confirming a memory")
    }

    fn sign_in(&self, _engine_id: &str) -> WorkspaceFuture<AssistantConnection> {
        Self::unsupported("Sign-in")
    }

    fn sign_out(&self, _engine_id: &str) -> WorkspaceFuture<AssistantConnection> {
        Self::unsupported("Sign-out")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_base_trims_adds_scheme_and_keeps_a_path() {
        assert_eq!(
            normalize_base("http://127.0.0.1:8099/"),
            "http://127.0.0.1:8099"
        );
        assert_eq!(normalize_base("127.0.0.1:8099"), "http://127.0.0.1:8099");
        assert_eq!(normalize_base(""), DEFAULT_BASE);
        assert_eq!(normalize_base("   "), DEFAULT_BASE);
        assert_eq!(
            normalize_base("https://chat.example.test/edit/"),
            "https://chat.example.test/edit",
            "a deliberately-typed path survives: a reverse proxy may mount \
             the server under one"
        );
        assert_eq!(
            service_base("127.0.0.1:8099"),
            "http://127.0.0.1:8099/api/chat"
        );
    }

    /// The mode is not a preference, so nothing can restore it.
    #[test]
    fn live_mode_is_never_restored_from_storage() {
        // A payload that tries its hardest: a mode field, in two spellings,
        // beside the two fields that ARE remembered.
        let stored = r#"{"mode":"live","workspace_mode":"Live",
                         "base":"127.0.0.1:9000/","backend":"mock"}"#;
        let prefs = parse_prefs(stored);
        assert_eq!(prefs.base, "http://127.0.0.1:9000");
        assert_eq!(prefs.backend.as_deref(), Some("mock"));
        // Round-tripping keeps only the two fields, so a mode cannot be
        // re-introduced by a later write either.
        let written = prefs_json(&prefs);
        assert!(!written.contains("mode"), "{written}");
        let back = parse_prefs(&written);
        assert_eq!(back, prefs);
        // And the page's own type has no mode to restore INTO: the only way
        // to reach Live is to select it, every time.
        let _: LivePrefs = LivePrefs::default();
        assert_eq!(LivePrefs::default().base, DEFAULT_BASE);
        assert_eq!(parse_prefs("not json"), LivePrefs::default());
    }

    /// Selecting Live is a choice about where a LATER connection would go.
    #[test]
    fn selecting_live_does_not_connect() {
        assert_eq!(request_intent(WorkspaceMode::Live, false), None);
        assert_eq!(request_intent(WorkspaceMode::Fixture, false), None);
        assert_eq!(request_intent(WorkspaceMode::Fixture, true), None);
        // …and the one pair that does request something asks for exactly one
        // thing: the pre-session probe.
        assert_eq!(
            request_intent(WorkspaceMode::Live, true),
            Some(LiveRequest::FetchBackends)
        );
    }

    /// I1: the mode attribute must never describe a backend the page is not
    /// driving, and leaving Live must take the session with it.
    #[test]
    fn switching_to_fixture_tears_a_live_session_down() {
        assert_eq!(
            mode_switch_effect(WorkspaceMode::Fixture, true),
            ModeSwitch::DisconnectFirst,
            "leaving Live with a session open must close it: the Disconnect \
             control lives inside the Live section and is hidden the moment \
             the mode changes"
        );
        assert_eq!(
            mode_switch_effect(WorkspaceMode::Fixture, false),
            ModeSwitch::Nothing
        );
        assert_eq!(
            mode_switch_effect(WorkspaceMode::Live, true),
            ModeSwitch::Nothing,
            "re-selecting Live does not disturb the session it is already on"
        );
        assert_eq!(
            mode_switch_effect(WorkspaceMode::Live, false),
            ModeSwitch::Nothing
        );

        // And the mount reads BOTH facts, so `fixture` on the hook and a live
        // backend under the workspace cannot coexist.
        assert!(!drives_live(WorkspaceMode::Fixture, true));
        assert!(!drives_live(WorkspaceMode::Live, false));
        assert!(!drives_live(WorkspaceMode::Fixture, false));
        assert!(drives_live(WorkspaceMode::Live, true));
    }

    /// I4: a Connect makes two requests, and only the second one earns the
    /// swap.
    #[test]
    fn a_failed_connect_never_promotes_the_live_workspace() {
        assert!(promote_on(&LiveStatus::Ready {
            session_id: "s1".to_owned(),
            backend: "mock".to_owned(),
        }));
        // Every non-Ready status, including the one a backend list produces.
        for status in [
            LiveStatus::Idle,
            LiveStatus::FetchingBackends,
            LiveStatus::Opening,
            LiveStatus::NoBackends,
            LiveStatus::SessionLost,
            LiveStatus::Unreachable {
                base: DEFAULT_BASE.to_owned(),
                detail: "refused".to_owned(),
            },
        ] {
            assert!(
                !promote_on(&status),
                "{} must leave the fixture mounted and driving",
                status.as_id()
            );
        }
    }

    #[test]
    fn modes_and_statuses_publish_stable_ids() {
        assert_eq!(WorkspaceMode::Fixture.as_id(), "fixture");
        assert_eq!(WorkspaceMode::Live.as_id(), "live");
        let statuses = [
            (LiveStatus::Idle, "idle"),
            (LiveStatus::FetchingBackends, "fetching_backends"),
            (LiveStatus::Opening, "opening"),
            (
                LiveStatus::Ready {
                    session_id: "s1".to_owned(),
                    backend: "mock".to_owned(),
                },
                "ready",
            ),
            (
                LiveStatus::Unreachable {
                    base: DEFAULT_BASE.to_owned(),
                    detail: "refused".to_owned(),
                },
                "unreachable",
            ),
            (LiveStatus::NoBackends, "no_backends"),
            (LiveStatus::SessionLost, "session_lost"),
        ];
        for (status, id) in &statuses {
            assert_eq!(status.as_id(), *id);
        }
        assert_eq!(
            statuses[3].0.session_id(),
            Some("s1"),
            "only a ready status names a session"
        );
        assert_eq!(LiveStatus::Idle.session_id(), None);
    }

    /// A card is built from what the server said, and says nothing more.
    #[test]
    fn a_card_never_claims_a_discovery_it_did_not_make() {
        let bare = Capabilities {
            id: "mock".into(),
            label: "Mock (local test)".into(),
            needs_api_key: false,
            models: vec![],
            permission_modes: vec![],
            supports_thinking: false,
            supports_tool_calls: false,
        };
        let card = card_for(&bare, "2026-09-16T00:00:00Z");
        assert_eq!(
            card.model_source,
            ModelSource::FreeText,
            "an empty model list is free text, not a discovery of nothing"
        );
        assert!(leptos_daisyui_rs::patterns::card_ready_for_ask(&card));
        let listed = Capabilities {
            models: vec!["llama3".into()],
            ..bare
        };
        assert_eq!(
            card_for(&listed, "2026-09-16T00:00:00Z").model_source,
            ModelSource::Discovered {
                as_of: "2026-09-16T00:00:00Z".to_owned()
            }
        );
    }

    #[test]
    fn http_failures_stay_distinguishable() {
        assert_eq!(HttpFailure::Status(503).detail(), "HTTP 503");
        assert_ne!(
            HttpFailure::SessionGone.detail(),
            HttpFailure::Unreachable("refused".to_owned()).detail(),
            "a session that is gone is not a server that never answered"
        );
    }
}
