//! The `AiChatWorkspace` composite: an opinionated chat workspace assembled
//! from the generic `components::ai_chat::AiChat` panel, this module's pure
//! vocabulary, and a host-implemented [`ChatWorkspaceBackend`].
//!
//! Shaped like `crate::patterns::helpdesk::Helpdesk`: the backend is an `Rc`
//! held in a `StoredValue::new_local`, every string arrives through a `texts`
//! prop, and the clock is overridable so a proof is deterministic.

use leptos::html::Div;
use leptos::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen_futures::spawn_local;

use super::backend::{ChatWorkspaceBackend, ChatWorkspaceErrorKind, WorkspaceRefusal};
use super::engine_header::EngineHeader;
use super::evidence_rail::EvidenceRail;
use super::knowledge::{
    CorpusScope, KnowledgeSelection, KnowledgeSource, MemoryDraft, MemoryRefusal, RecallReceipt,
};
use super::knowledge_rail::{
    KnowledgeDraft, KnowledgeSourceRail, any_ingest_in_flight, with_ingest,
};
use super::provider::{
    AvailabilityReasonCode, ProviderCard, card_ready_for_ask, card_unready_reason,
};
use super::settings_rows::{ProviderSettingsRows, TuningDraft};
use super::status::{TurnNotice, TurnRecord};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_assistant_workspace::{
    AssistantAccess, AttemptLifecycle, MemoryState, RefusalNextAction,
};
use crate::components::ai_chat::AiChatTexts;
use crate::components::ai_chat::{
    AiChat, AnnotationAnchor, AnnotationBody, AnnotationKind, Capabilities, ChatSession,
    ChatSettings, TranscriptAnnotation,
};
use crate::merge_classes;

/// Per-instance sequence, so two mounted workspaces never mint colliding
/// `label for=` targets in one document.
static WORKSPACE_SEQ: AtomicU64 = AtomicU64::new(0);

/// How often the composite asks the backend for the live turn's record, in
/// milliseconds. Deliberately slower than `AiChat`'s own 100 ms transport
/// poll: this is metadata for the header, not the stream.
const TURN_POLL_MS: u64 = 200;

/// The ingest ticker's own beat, in milliseconds. Short, because it only
/// COUNTS; see [`INGEST_TICKS_PER_PHASE`] for what actually advances a phase.
const INGEST_TICK_MS: u64 = 150;

/// How many ticks one ingest phase is held for before the composite re-reads
/// the knowledge sources — 900 ms.
///
/// Both the duration and the COUNTING are findings rather than taste, and the
/// browser lane produced them one after the other:
///
/// * At 250 ms per rung the first TWO rungs were gone before the proof's
///   post-click settle finished; its first sample read `clustering`. A phase
///   display that changes faster than the page can be read is not a phase
///   display — an actor watching a folder index sees the same rungs flash.
/// * Lengthening the interval alone was still not enough, because a
///   FREE-RUNNING interval has an arbitrary phase relative to the click: a
///   tick landing a millisecond after `walking` was applied consumed the rung
///   immediately, and the proof's next sample read `indexing`. Counting ticks
///   and resetting the counter when the reindex starts is what gives every
///   rung — including the first — its full interval, measured from the action
///   that caused it.
const INGEST_TICKS_PER_PHASE: u32 = 6;

/// How long a turn may sit without reaching a terminal lifecycle before the
/// watchdog fails it, in milliseconds.
pub const WATCHDOG_MS: i64 = 120_000;

/// The turn whose record the composite reads on this tick.
///
/// [`ChatWorkspaceBackend::current_turn_id`] reports the turn currently IN
/// FLIGHT, so it drops to `None` in the very call that writes the terminal
/// lifecycle onto the record — `Completed`, `Failed`, `Unavailable` and
/// `Canceled` all mark the turn finished at the same instant they are
/// recorded. A tick that reads only the live id therefore never observes a
/// terminal record at all: the header latches on the last thing it managed to
/// read, which is `validating`, and `outcome`, `usage`, `limitations` and the
/// failure kind stay dark on a perfectly healthy turn.
///
/// Falling back to the turn already being watched keeps the finished turn
/// readable until a new `send` replaces it, which is the contract the rest of
/// this tick is written against (the cancel notice is announced once per turn
/// *id* precisely because the terminal record is re-read every tick).
pub fn tick_turn_id(live: Option<String>, watched: Option<&str>) -> Option<String> {
    live.or_else(|| watched.map(str::to_owned))
}

/// Whether the watchdog should fail the turn on this tick.
///
/// Pure so the threshold is provable without a browser: the composite's own
/// tick supplies `elapsed_ms` from its clock, `terminal` from
/// [`is_terminal`], and `already_fired` from the watched entry. All three
/// gates matter, and each has its own test:
///
/// * `terminal` — a turn that finished at 130 s must not be failed after the
///   fact,
/// * `already_fired` — firing must be once per turn, not once per tick and
///   not once per [`WATCHDOG_MS`] period (see the notice re-announcement this
///   replaced),
/// * `elapsed_ms >= WATCHDOG_MS` — inclusive, so the boundary tick fires.
pub fn watchdog_should_fire(elapsed_ms: i64, terminal: bool, already_fired: bool) -> bool {
    !terminal && !already_fired && elapsed_ms >= WATCHDOG_MS
}

/// Why a session is being (re)opened, and therefore whether the transcript
/// should announce it.
///
/// A session is torn down and rebuilt for two unrelated reasons, and only one
/// of them is a provider switch. Treating "a session already exists" as the
/// discriminator announced `"Switched to {engine}"` for an engine that never
/// changed, every time the actor picked a different folder in the knowledge
/// rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReopenReason {
    /// The workspace is booting; there is no conversation to reset and
    /// nothing to announce.
    Boot,
    /// The actor picked a different engine. This is the only reason that
    /// announces.
    EngineSwitch,
    /// The knowledge selection changed (corpus, query mode or posture). The
    /// session is rebuilt because a turn is opened against a knowledge mix,
    /// but the engine did not change — so the engine-switch wording would be
    /// false. It gets its OWN sentence: see [`reopen_notice`].
    KnowledgeChange,
}

/// Whether reopening for this reason announces a provider switch in the
/// transcript.
pub fn reopen_announces_switch(reason: ReopenReason) -> bool {
    matches!(reason, ReopenReason::EngineSwitch)
}

/// What the transcript should say about a reopen, or `None` for a reopen
/// with nothing to report.
///
/// Three reasons, three answers, and the middle one was the whole point of
/// the hand-off P4 left here. An ENGINE switch names the engine. A KNOWLEDGE
/// change names neither engine nor knowledge source — it says the
/// conversation started over, which is the part the actor can otherwise only
/// discover by noticing their transcript is gone. BOOT says nothing, because
/// there was no conversation to reset.
///
/// Silence for a knowledge change was P4's deliberate placeholder (its proof
/// pinned the silence so that whoever added a notice had to come back and
/// change the assertion, rather than route around it). It cannot stay:
/// `on_knowledge_select` reopens the session, which CLEARS the transcript,
/// and a reset that announces nothing is indistinguishable from a bug.
pub fn reopen_notice(
    reason: ReopenReason,
    engine_label: &str,
    texts: &AiChatWorkspaceTexts,
) -> Option<String> {
    match reason {
        ReopenReason::Boot => None,
        ReopenReason::EngineSwitch => Some(texts.switched_engine.replace("{engine}", engine_label)),
        ReopenReason::KnowledgeChange => Some(texts.knowledge_changed.clone()),
    }
}

/// Whether a lifecycle is one the turn can never leave.
pub fn is_terminal(l: &AttemptLifecycle) -> bool {
    matches!(
        l,
        AttemptLifecycle::Completed(_)
            | AttemptLifecycle::Denied { .. }
            | AttemptLifecycle::Unavailable { .. }
            | AttemptLifecycle::Failed { .. }
            | AttemptLifecycle::Canceled { .. }
            | AttemptLifecycle::Interrupted { .. }
    )
}

/// The localized wording for one turn notice, and the stable kind id its DOM
/// hook carries.
///
/// `TurnNotice` is DATA — a backend never mints presentation text — so this
/// is the one place a notice becomes a sentence.
pub fn notice_text(notice: &TurnNotice, texts: &AiChatWorkspaceTexts) -> (&'static str, String) {
    match notice {
        TurnNotice::Escalated { from, to } => (
            "escalated",
            texts
                .escalated
                .replace("{from}", from.as_str())
                .replace("{to}", to.as_str()),
        ),
        TurnNotice::Truncated => ("truncated", texts.truncated.clone()),
        // A notice this vocabulary does not name still renders, carrying the
        // host's own code, rather than being dropped.
        TurnNotice::Unknown(code) => ("unknown", code.clone()),
    }
}

/// The transcript notice a CANCELED turn earns, or `None` for a turn that
/// was not canceled.
///
/// The two cancels are not one event with a flag: a kept partial leaves the
/// actor something to read and a discarded one leaves them nothing, so they
/// get two sentences. Returning `None` for every other lifecycle is what
/// keeps the composite from announcing a cancel that never happened.
pub fn cancel_notice(lifecycle: &AttemptLifecycle, texts: &AiChatWorkspaceTexts) -> Option<String> {
    match lifecycle {
        AttemptLifecycle::Canceled { discarded: true } => {
            Some(texts.canceled_discarded_notice.clone())
        }
        AttemptLifecycle::Canceled { discarded: false } => Some(texts.canceled_kept_notice.clone()),
        _ => None,
    }
}

/// The `Capabilities` one card should PUBLISH to the generic panel, which is
/// not always the `Capabilities` it carries.
///
/// `AiChat::model_field_mode` derives the Model row's shape from
/// `Capabilities::models` alone — one model is a pin, several are a choice —
/// because a generic panel has no other vocabulary. `ModelSource::Pinned`
/// lives on the CARD, so a card that pins one of a seven-model catalogue
/// (`codex-spark`) published its raw list and the panel offered all seven on
/// an engine the host had pinned. Found by the browser suite on its first
/// real run; the pin is applied here, where both facts are in scope.
///
/// `ModelSource::Discovered` is deliberately NOT collapsed: a discovered list
/// is still a list of choices, only with a freshness stamp the panel does not
/// render.
pub fn published_capabilities(card: &ProviderCard) -> Capabilities {
    let mut caps = card.capabilities.clone();
    if let super::provider::ModelSource::Pinned(model) = &card.model_source {
        caps.models = vec![model.clone()];
    }
    caps
}

/// The chat settings one card should open its session with: the card's own
/// model (when it publishes exactly one, pin included), and the two
/// visibility toggles following what the engine can actually emit.
///
/// A card that cannot emit thinking must not open a session asking for it —
/// `AiChat` renders that toggle disabled and explained, and a session
/// configured the other way would disagree with the control beside it.
pub fn settings_for(card: &ProviderCard) -> ChatSettings {
    let published = published_capabilities(card);
    let models = &published.models;
    ChatSettings {
        model: (models.len() == 1).then(|| models[0].clone()),
        system_prompt: None,
        allowed_tools: None,
        show_thinking: published.supports_thinking,
        show_tool_calls: published.supports_tool_calls,
    }
}

fn js_now_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// # AiChatWorkspace
///
/// The opinionated chat workspace: an engine header, the knowledge rail, the
/// generic chat panel, and the evidence rail, over one
/// [`ChatWorkspaceBackend`].
///
/// ## What it owns that `AiChat` deliberately does not
/// Which engines exist and which of them may be asked
/// ([`card_ready_for_ask`]); what a switch costs (a fresh session, a reset
/// transcript and an announced notice); which provider levers a card exposes
/// (its `TuningSchema`, never the shape of an applied `ProviderTuning`);
/// which knowledge a turn is opened against; and a watchdog that fails a turn
/// the host never finished.
///
/// ## CSS
/// ```css
/// @source inline("rounded-box border border-base-300 bg-base-100 p-4 gap-2 gap-3 gap-4");
/// @source inline("grid grid-cols-1 lg:grid-cols-[16rem_1fr_16rem] h-[32rem] min-h-0 w-full");
/// @source inline("select select-sm select-bordered checkbox checkbox-sm range range-xs");
/// ```
#[component]
pub fn AiChatWorkspace(
    /// The transport seam. `Rc` because every workspace future is `!Send`.
    backend: Rc<dyn ChatWorkspaceBackend>,
    /// Every string the workspace itself renders.
    #[prop(optional, into, default = Signal::stored(AiChatWorkspaceTexts::default()))]
    texts: Signal<AiChatWorkspaceTexts>,
    /// Every string the embedded chat panel renders.
    #[prop(optional, into, default = Signal::stored(AiChatTexts::default()))]
    chat_texts: Signal<AiChatTexts>,
    /// Which engine to open first. `None` picks the first card that
    /// [`card_ready_for_ask`] accepts.
    #[prop(optional, into)]
    initial_engine: Option<String>,
    /// What knowledge to open the first session against.
    #[prop(optional, into)]
    initial_knowledge: Option<KnowledgeSelection>,
    /// A host-supplied clock for the watchdog. `None` falls back to
    /// `js Date.now()`.
    ///
    /// The composite holds a `dyn ChatWorkspaceBackend` and therefore cannot
    /// reach a fixture's inherent clock; a host driving a fixture whose time
    /// only advances per poll must pass that clock in, or the watchdog
    /// threshold can never be crossed.
    #[prop(optional, into)]
    now_ms: Option<Signal<i64>>,
    /// Invoked when the actor presses a refusal's single next-step button,
    /// with the typed action that button offered.
    ///
    /// `RefusalNextAction::RetryLater` and
    /// `RefusalNextAction::NewConversation` are also acted on HERE — the
    /// composite owns reopening a session, so a button that only notified
    /// the host would do nothing in the common case.
    /// `RefusalNextAction::OpenSettings` is notify-only: the settings an
    /// actor must fix (a credential, a plan) live in the host application,
    /// and a workspace that pretended to open them would be lying about
    /// what the click did.
    #[prop(optional, into)]
    on_refusal_action: Option<Callback<RefusalNextAction>>,
    /// Extra classes merged onto the root.
    #[prop(optional, into)]
    class: &'static str,
    /// Node reference for the root `<div>`.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let backend = StoredValue::new_local(backend);
    let instance = WORKSPACE_SEQ.fetch_add(1, Ordering::Relaxed);
    let id_prefix = format!("aichatws-{instance}");
    let now = now_ms.unwrap_or_else(|| Signal::derive(js_now_ms));

    let cards: RwSignal<Vec<ProviderCard>> = RwSignal::new(Vec::new());
    let sources: RwSignal<Vec<KnowledgeSource>> = RwSignal::new(Vec::new());
    let budget: RwSignal<Option<String>> = RwSignal::new(None);
    let engine_id: RwSignal<String> = RwSignal::new(initial_engine.clone().unwrap_or_default());
    let selection: RwSignal<KnowledgeSelection> =
        RwSignal::new(initial_knowledge.unwrap_or_default());
    let annotations: RwSignal<Vec<TranscriptAnnotation>> = RwSignal::new(Vec::new());
    let turn: RwSignal<Option<TurnRecord>> = RwSignal::new(None);
    let refusal: RwSignal<Option<WorkspaceRefusal>> = RwSignal::new(None);
    // Whether the actor's reasoning tier denies every effect. Read from
    // `settings()` at boot, BEFORE the first session is opened, because a
    // denied tier must stop the open rather than explain it afterwards.
    let tier_denied: RwSignal<bool> = RwSignal::new(false);
    // Whether the watchdog has failed the live turn. Separate from the
    // record: `fail_turn` acts on the `ChatSession`, so a stalled turn's own
    // lifecycle never leaves `Running` and the record can never carry this.
    let timed_out: RwSignal<bool> = RwSignal::new(false);
    // The turn id whose cancellation has already been announced, so a
    // terminal lifecycle the composite re-reads on every 200 ms tick becomes
    // one notice rather than five a second.
    let cancel_announced: RwSignal<Option<String>> = RwSignal::new(None);
    let draft = TuningDraft::new();
    // The knowledge rail's two write flows. The DRAFT is held here, not in
    // the rail, because only the composite learns whether a `remember` was
    // accepted — and a refused draft must survive its refusal.
    let knowledge_draft = KnowledgeDraft::new();
    let receipt: RwSignal<Option<RecallReceipt>> = RwSignal::new(None);
    let memory_refusal: RwSignal<Option<MemoryRefusal>> = RwSignal::new(None);
    // Whether the memory store failed to answer. A typed honesty state, not
    // an absent receipt: an unreachable store must not leave the LAST
    // receipt on screen, where it reads as a fresh result.
    let memory_offline: RwSignal<bool> = RwSignal::new(false);
    // Ticks elapsed in the current ingest phase. Reset by a reindex, so the
    // phase clock is measured from the ACTION rather than from an interval
    // that happens to be running.
    let ingest_ticks: RwSignal<u32> = RwSignal::new(0);
    // The open session, plus the generation that forces a remount. Reading
    // `generation` inside the panel's view closure is what makes an engine
    // switch REPLACE the panel rather than reuse it: `AiChat` seeds its
    // settings form and its backend selection once, at mount.
    let session: RwSignal<Option<StoredValue<ChatSession, LocalStorage>>> = RwSignal::new(None);
    let generation = RwSignal::new(0u32);
    // The turn the watchdog is timing, when the composite first saw it, and
    // whether the watchdog has already fired for it. The flag is why this is
    // never cleared while the turn is still current: `fail_turn` acts on the
    // `ChatSession`, not on the backend, so `current_turn_id()` keeps
    // returning the same id — and clearing the entry re-seeded it on the next
    // tick, which reset `announced` to 0 and re-announced every notice the
    // turn had ever carried, once per watchdog period, forever.
    let watched: RwSignal<Option<(String, i64, bool)>> = RwSignal::new(None);
    // How many of the live turn's notices have already been announced, so a
    // notice becomes an annotation exactly once.
    let announced: RwSignal<usize> = RwSignal::new(0);

    let card_for = move |id: &str| {
        cards
            .get_untracked()
            .into_iter()
            .find(|c| c.engine.id == id)
    };

    let push_tuning = move |id: String| {
        let Some(card) = card_for(&id) else {
            return;
        };
        let tuning = draft.to_tuning(&card);
        spawn_local(async move {
            let fut = backend.with_value(|b| b.set_tuning(&id, tuning));
            let _ = fut.await;
        });
    };

    // The typed refusal for a card the workspace will not open a session
    // against, which is a DIFFERENT event from a session that was opened and
    // then failed. The reason code comes off the card (or off the tier), so
    // the next-step button is derived rather than guessed.
    let refuse_card = move |id: &str| {
        let t = texts.get_untracked();
        let (code, message) = if tier_denied.get_untracked() {
            let code = AvailabilityReasonCode::TierEffectsDisabled;
            let message = t.availability_reason(&code);
            (Some(code), message)
        } else {
            match card_for(id).and_then(|c| card_unready_reason(&c)) {
                Some(code) => {
                    let message = t.availability_reason(&code);
                    (Some(code), message)
                }
                None => (None, t.not_enabled.clone()),
            }
        };
        refusal.set(Some(WorkspaceRefusal {
            kind: ChatWorkspaceErrorKind::Refused,
            code,
            message,
            engine_id: Some(id.to_owned()),
        }));
    };

    // Open (or reopen) a session against one engine.
    //
    // A refused open is an honesty state, not a reset: the previous session
    // stays mounted, so the actor's draft and transcript survive a refusal
    // they did not ask for, and the header explains what happened.
    let open_engine = move |id: String, reason: ReopenReason| {
        let Some(card) = card_for(&id) else {
            return;
        };
        draft.seed_from(&card);
        let settings = settings_for(&card);
        let seed_settings = settings.clone();
        let tuning = draft.to_tuning(&card);
        let knowledge = selection.get_untracked();
        let label = card.capabilities.label.clone();
        let refused = id.clone();
        spawn_local(async move {
            let fut = backend.with_value(|b| b.open_session(&id, &knowledge, settings, tuning));
            match fut.await {
                Ok(transport) => {
                    let notice = reopen_notice(reason, &label, &texts.get_untracked());
                    // `ChatSession::new` starts at `ChatSettings::default()`
                    // — both visibility flags FALSE — and the session, not
                    // the transport, is what drops a `Thinking` or
                    // `ToolCall` event before it reaches the transcript. The
                    // settings the session was OPENED with therefore have to
                    // be pushed into it as well, or a thinking-capable
                    // engine streams a transcript with no thinking in it and
                    // `AiChat` seeds its toggles from the wrong values.
                    let mut opened = ChatSession::new(transport);
                    opened.configure(seed_settings);
                    session.try_set(Some(StoredValue::new_local(opened)));
                    engine_id.try_set(id);
                    turn.try_set(None);
                    watched.try_set(None);
                    announced.try_set(0);
                    timed_out.try_set(false);
                    cancel_announced.try_set(None);
                    refusal.try_set(None);
                    // Any reopen resets the conversation, so the previous
                    // annotations describe a transcript that no longer
                    // exists and are always cleared. What replaces them is
                    // `reopen_notice`'s business: a boot says nothing, an
                    // engine switch names the engine, and a knowledge change
                    // says the conversation started over WITHOUT naming an
                    // engine that did not change.
                    annotations.try_set(
                        notice
                            .map(|body| {
                                vec![TranscriptAnnotation {
                                    anchor: AnnotationAnchor::AtStart,
                                    kind: AnnotationKind::Notice,
                                    body: AnnotationBody::Text(body),
                                }]
                            })
                            .unwrap_or_default(),
                    );
                    generation.try_update(|g| *g += 1);
                }
                Err(e) => {
                    // The card's own reason is what picks a truthful next
                    // step; the error kind alone cannot tell "sign in again"
                    // from "wait". A transport error with no card reason
                    // falls back to the action that promises nothing.
                    let code = card_for(&refused).and_then(|c| card_unready_reason(&c));
                    refusal.try_set(Some(WorkspaceRefusal {
                        kind: e.kind,
                        code,
                        message: e.message,
                        engine_id: Some(refused),
                    }));
                }
            }
        });
    };

    // Boot: settings, providers and knowledge once, then the first session.
    let boot = move || {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.settings());
            if let Ok(s) = fut.await {
                let line = s.budget.as_ref().and_then(|b| {
                    b.remaining_display
                        .as_ref()
                        .map(|r| format!("{}: {r}", b.policy_label))
                });
                budget.try_set(line);
                // The tier verdict is read BEFORE the first open below, and
                // it outranks every card: an account that may not use
                // assistant effects at all must not have a session opened
                // for it just because one engine looks healthy.
                tier_denied.try_set(matches!(s.reasoning_tier, AssistantAccess::Denied { .. }));
            }
            let fut = backend.with_value(|b| b.knowledge());
            if let Ok(k) = fut.await {
                sources.try_set(k);
            }
            let fut = backend.with_value(|b| b.providers());
            let Ok(list) = fut.await else {
                return;
            };
            let wanted = initial_engine.clone();
            let chosen = wanted
                .and_then(|id| list.iter().find(|c| c.engine.id == id).cloned())
                .or_else(|| list.iter().find(|c| card_ready_for_ask(c)).cloned());
            cards.try_set(list);
            let Some(card) = chosen else {
                return;
            };
            if tier_denied.get_untracked() {
                // Denied: no session at all, and the header says why. Opening
                // one anyway would render a composer the actor may not use.
                refuse_card(&card.engine.id);
                return;
            }
            open_engine(card.engine.id.clone(), ReopenReason::Boot);
        });
    };
    boot();

    // The live turn's record, the notices it has grown, and the watchdog.
    let tick = move || {
        // A turn goes terminal and stops being "in flight" in the same call,
        // so the live id alone can never resolve to a finished record: fall
        // back to the turn already being watched. See `tick_turn_id`.
        let live = backend.with_value(|b| b.current_turn_id());
        let watching = watched.get_untracked().map(|(id, _, _)| id);
        let Some(id) = tick_turn_id(live, watching.as_deref()) else {
            return;
        };
        let (started, already_fired) = match watched.get_untracked() {
            Some((ref seen, at, fired)) if *seen == id => (at, fired),
            _ => {
                let at = now.get_untracked();
                watched.set(Some((id.clone(), at, false)));
                announced.set(0);
                timed_out.set(false);
                (at, false)
            }
        };
        let elapsed = now.get_untracked() - started;
        spawn_local(async move {
            let fut = backend.with_value(|b| b.turn(&id));
            let Ok(record) = fut.await else {
                return;
            };
            let terminal = is_terminal(&record.lifecycle);
            let t = texts.get_untracked();
            let seen = announced.get_untracked();
            if record.notices.len() > seen {
                let fresh: Vec<TranscriptAnnotation> = record.notices[seen..]
                    .iter()
                    .map(|n| TranscriptAnnotation {
                        anchor: AnnotationAnchor::AtEnd,
                        kind: AnnotationKind::Warning,
                        body: AnnotationBody::Text(notice_text(n, &t).1),
                    })
                    .collect();
                announced.try_set(record.notices.len());
                annotations.try_update(|a| a.extend(fresh));
            }
            // A cancel is announced once per turn, by id: the record stays
            // terminal for as long as it is the live turn, and the composite
            // re-reads it every tick. A WATCHDOG failure is excluded: it
            // reaches the transport through `cancel()` too, and announcing
            // "you stopped this answer" for a turn the actor never touched
            // would be the same lie the watchdog notice exists to replace.
            let fresh_cancel = cancel_notice(&record.lifecycle, &t)
                .filter(|_| !timed_out.get_untracked())
                .filter(|_| {
                    cancel_announced.get_untracked().as_deref() != Some(record.id.as_str())
                });
            if let Some(body) = fresh_cancel {
                cancel_announced.try_set(Some(record.id.clone()));
                annotations.try_update(|a| {
                    a.push(TranscriptAnnotation {
                        anchor: AnnotationAnchor::AtEnd,
                        kind: AnnotationKind::Warning,
                        body: AnnotationBody::Text(body),
                    })
                });
            }
            turn.try_set(Some(record));
            if watchdog_should_fire(elapsed, terminal, already_fired) {
                let message = t.turn_timed_out.clone();
                if let Some(s) = session.get_untracked() {
                    let _ = s.try_update_value(|s| s.fail_turn(message.clone()));
                }
                annotations.try_update(|a| {
                    a.push(TranscriptAnnotation {
                        anchor: AnnotationAnchor::AtEnd,
                        kind: AnnotationKind::Warning,
                        body: AnnotationBody::Text(message),
                    })
                });
                timed_out.try_set(true);
                // Marked fired, NOT cleared: clearing re-seeds the same
                // turn id on the next tick and replays every notice.
                watched.try_update(|w| {
                    if let Some(entry) = w.as_mut() {
                        entry.2 = true;
                    }
                });
            }
        });
    };
    if let Ok(handle) = leptos::leptos_dom::helpers::set_interval_with_handle(
        tick,
        std::time::Duration::from_millis(TURN_POLL_MS),
    ) {
        on_cleanup(move || handle.clear());
    }

    let active_card = Signal::derive(move || {
        let id = engine_id.get();
        cards.get().into_iter().find(|c| c.engine.id == id)
    });
    let backend_caps: Signal<Vec<Capabilities>> =
        Signal::derive(move || cards.get().iter().map(published_capabilities).collect());
    let assistant_label = Signal::derive(move || {
        active_card
            .get()
            .map(|c| c.capabilities.label)
            .unwrap_or_default()
    });
    let active_id = Signal::derive(move || {
        let id = engine_id.get();
        (!id.is_empty()).then_some(id)
    });
    let header_notices = Signal::derive(move || {
        let t = texts.get();
        turn.get()
            .map(|r| r.notices.iter().map(|n| notice_text(n, &t)).collect())
            .unwrap_or_default()
    });

    let on_engine_change = Callback::new(move |id: String| {
        // A card that cannot be asked never gets a session opened against it;
        // the header already carries the reason. A denied tier refuses every
        // card, however healthy the card itself looks.
        if !tier_denied.get_untracked() && card_for(&id).is_some_and(|c| card_ready_for_ask(&c)) {
            open_engine(id, ReopenReason::EngineSwitch);
        } else {
            refuse_card(&id);
        }
    });
    // "New session" is `AiChat`'s own control: it restarts the session and
    // clears the transcript, and the composite only learns through this
    // callback. Without it the header goes on publishing the FINISHED turn's
    // outcome, usage, limitations and honesty — and the annotations it grew —
    // over a conversation that no longer exists. The `tick_turn_id` fallback
    // made that permanent rather than transient, so the stale state is a
    // completed answer, not a passing `validating`.
    //
    // Nothing is announced: the actor pressed the button, so a notice row on
    // the empty transcript would be the composite telling them what they just
    // did. Same reasoning as `ReopenReason::Boot`.
    let on_panel_restart = Callback::new(move |()| {
        turn.set(None);
        watched.set(None);
        announced.set(0);
        timed_out.set(false);
        cancel_announced.set(None);
        annotations.set(Vec::new());
        refusal.set(None);
    });
    let refusal_action = Callback::new(move |action: RefusalNextAction| {
        if let Some(cb) = on_refusal_action {
            cb.run(action);
        }
        let refused = refusal
            .get_untracked()
            .and_then(|r| r.engine_id)
            .unwrap_or_else(|| engine_id.get_untracked());
        match action {
            // The composite owns both of these, so the button does what it
            // says: try the refused engine again, or start over on the
            // engine that is running.
            RefusalNextAction::RetryLater => {
                if !refused.is_empty() {
                    open_engine(refused, ReopenReason::EngineSwitch);
                }
            }
            RefusalNextAction::NewConversation => {
                let id = engine_id.get_untracked();
                if !id.is_empty() {
                    open_engine(id, ReopenReason::Boot);
                }
            }
            // Notify-only: see the prop's own documentation.
            RefusalNextAction::OpenSettings => {}
        }
    });
    let on_permission_mode_change = Callback::new(move |mode: String| {
        draft.permission_mode.set(Some(mode));
        push_tuning(engine_id.get_untracked());
    });
    let on_tuning_change = Callback::new(move |()| push_tuning(engine_id.get_untracked()));
    let on_knowledge_select = Callback::new(move |next: KnowledgeSelection| {
        selection.set(next);
        let id = engine_id.get_untracked();
        if !id.is_empty() {
            open_engine(id, ReopenReason::KnowledgeChange);
        }
    });

    // Re-read the published knowledge sources. Every memory mutation goes
    // through here rather than patching the local copy, so the rail shows
    // what the HOST holds — a `set_memory_state` the host refused must not
    // leave a confirmed badge on screen.
    let refresh_knowledge = move || {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.knowledge());
            if let Ok(k) = fut.await {
                sources.try_set(k);
            }
        });
    };

    let on_reindex = Callback::new(move |scope: CorpusScope| {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.reindex(&scope));
            if let Ok(status) = fut.await {
                // Restart the phase clock, so the FIRST rung gets a full
                // interval rather than whatever is left of a tick that was
                // already in flight.
                ingest_ticks.try_set(0);
                // The reindex's OWN status is applied, rather than a
                // `knowledge()` refresh: see `with_ingest` for why a refresh
                // here makes the first phase unobservable.
                sources.try_update(|s| {
                    *s = with_ingest(std::mem::take(s), &scope, status);
                });
            }
        });
    });

    let on_memory_flags = Callback::new(move |(use_enabled, capture): (bool, bool)| {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.set_memory_flags(use_enabled, capture));
            let _ = fut.await;
            refresh_knowledge();
        });
    });

    let on_recall = Callback::new(move |query: String| {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.recall(&query));
            match fut.await {
                Ok(r) => {
                    memory_offline.try_set(false);
                    receipt.try_set(Some(r));
                }
                Err(_) => {
                    // No fake receipt, and the previous one is dropped: a
                    // receipt is evidence that a search RAN, so leaving a
                    // stale one beside an unreachable store would be the
                    // workspace vouching for a search that did not happen.
                    memory_offline.try_set(true);
                    receipt.try_set(None);
                }
            }
        });
    });

    let on_remember = Callback::new(move |d: MemoryDraft| {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.remember(d));
            match fut.await {
                Ok(Ok(_)) => {
                    memory_offline.try_set(false);
                    memory_refusal.try_set(None);
                    // Only an ACCEPTED write clears the draft. A refusal
                    // leaves the actor's own words in the box to edit.
                    knowledge_draft.remember_body.try_set(String::new());
                    refresh_knowledge();
                }
                Ok(Err(refusal)) => {
                    memory_offline.try_set(false);
                    memory_refusal.try_set(Some(refusal));
                }
                Err(_) => {
                    memory_offline.try_set(true);
                    memory_refusal.try_set(None);
                }
            }
        });
    });

    let on_memory_state = Callback::new(move |(id, state): (String, MemoryState)| {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.set_memory_state(&id, state));
            let _ = fut.await;
            refresh_knowledge();
        });
    });

    // Advance the ingest ladder only while a corpus is actually mid-ingest,
    // and only once every `INGEST_TICKS_PER_PHASE` ticks. An unconditional
    // poll would hold the fixture's phase ladder open forever and log a
    // `Knowledge` call several times a second on a workspace nobody is
    // indexing anything on; a free-running one would hand the first rung an
    // arbitrary fraction of its interval (see the constants).
    if let Ok(handle) = leptos::leptos_dom::helpers::set_interval_with_handle(
        move || {
            if !any_ingest_in_flight(&sources.get_untracked()) {
                ingest_ticks.try_set(0);
                return;
            }
            let next = ingest_ticks.get_untracked() + 1;
            if next >= INGEST_TICKS_PER_PHASE {
                ingest_ticks.try_set(0);
                refresh_knowledge();
            } else {
                ingest_ticks.try_set(next);
            }
        },
        std::time::Duration::from_millis(INGEST_TICK_MS),
    ) {
        on_cleanup(move || handle.clear());
    }

    let extra_prefix = id_prefix.clone();
    let settings_extra = ViewFn::from(move || {
        view! {
            <ProviderSettingsRows
                card=active_card
                draft=draft
                texts=texts
                id_prefix=extra_prefix.clone()
                on_change=on_tuning_change
            />
        }
        .into_any()
    });

    let rail_prefix = id_prefix.clone();
    let panel = move || {
        generation.track();
        session.get().map(|s| {
            view! {
                <AiChat
                    session=s
                    class="h-full"
                    show_settings=true
                    texts=chat_texts
                    assistant_label=assistant_label
                    backends=backend_caps
                    active_backend_id=active_id
                    on_backend_change=on_engine_change
                    permission_mode=Signal::derive(move || draft.permission_mode.get())
                    on_permission_mode_change=on_permission_mode_change
                    annotations=annotations
                    on_restart=on_panel_restart
                    settings_extra=settings_extra.clone()
                />
            }
        })
    };

    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!("lds-aichat-workspace flex w-full flex-col gap-4", class)
            }
            data-ai-chat-workspace=""
            data-ai-chat-workspace-engine=move || engine_id.get()
            data-ai-chat-workspace-locale=move || texts.get().locale_id
        >
            <EngineHeader
                cards=cards
                engine_id=engine_id
                turn=turn
                budget=budget
                refusal=refusal
                tier_denied=tier_denied
                timed_out=timed_out
                notices=header_notices
                texts=texts
                on_refusal_action=refusal_action
            />
            <div class="grid w-full grid-cols-1 gap-4 lg:grid-cols-[16rem_1fr_16rem]">
                <KnowledgeSourceRail
                    sources=sources
                    selection=selection
                    on_select=on_knowledge_select
                    on_reindex=on_reindex
                    on_memory_flags=on_memory_flags
                    on_recall=on_recall
                    on_remember=on_remember
                    on_memory_state=on_memory_state
                    receipt=receipt
                    memory_offline=memory_offline
                    refusal=memory_refusal
                    draft=knowledge_draft
                    texts=texts
                    id_prefix=rail_prefix
                />
                <div class="h-[32rem] min-h-0 overflow-hidden rounded-box border border-base-300 bg-base-100">
                    {panel}
                </div>
                <EvidenceRail turn=turn selection=selection texts=texts />
            </div>
        </div>
    }
}
