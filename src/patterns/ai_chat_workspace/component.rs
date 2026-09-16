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

use super::backend::{ChatWorkspaceBackend, ChatWorkspaceErrorKind};
use super::engine_header::EngineHeader;
use super::evidence_rail::EvidenceRail;
use super::knowledge::{KnowledgeSelection, KnowledgeSource};
use super::knowledge_rail::KnowledgeSourceRail;
use super::provider::{ProviderCard, card_ready_for_ask};
use super::settings_rows::{ProviderSettingsRows, TuningDraft};
use super::status::{TurnNotice, TurnRecord};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_assistant_workspace::AttemptLifecycle;
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

/// How long a turn may sit without reaching a terminal lifecycle before the
/// watchdog fails it, in milliseconds.
pub const WATCHDOG_MS: i64 = 120_000;

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
    let refusal: RwSignal<Option<String>> = RwSignal::new(None);
    let draft = TuningDraft::new();
    // The open session, plus the generation that forces a remount. Reading
    // `generation` inside the panel's view closure is what makes an engine
    // switch REPLACE the panel rather than reuse it: `AiChat` seeds its
    // settings form and its backend selection once, at mount.
    let session: RwSignal<Option<StoredValue<ChatSession, LocalStorage>>> = RwSignal::new(None);
    let generation = RwSignal::new(0u32);
    // The turn the watchdog is timing, and when the composite first saw it.
    let watched: RwSignal<Option<(String, i64)>> = RwSignal::new(None);
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

    // Open (or reopen) a session against one engine.
    //
    // A refused open is an honesty state, not a reset: the previous session
    // stays mounted, so the actor's draft and transcript survive a refusal
    // they did not ask for, and the header explains what happened.
    let open_engine = move |id: String| {
        let Some(card) = card_for(&id) else {
            return;
        };
        draft.seed_from(&card);
        let settings = settings_for(&card);
        let seed_settings = settings.clone();
        let tuning = draft.to_tuning(&card);
        let knowledge = selection.get_untracked();
        let label = card.capabilities.label.clone();
        let switching = session.get_untracked().is_some();
        spawn_local(async move {
            let fut = backend.with_value(|b| b.open_session(&id, &knowledge, settings, tuning));
            match fut.await {
                Ok(transport) => {
                    let notice = texts
                        .get_untracked()
                        .switched_engine
                        .replace("{engine}", &label);
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
                    refusal.try_set(None);
                    // A switch resets the conversation, so the previous
                    // annotations describe a transcript that no longer
                    // exists. The first open announces nothing: there is no
                    // conversation to have been reset.
                    annotations.try_set(if switching {
                        vec![TranscriptAnnotation {
                            anchor: AnnotationAnchor::AtStart,
                            kind: AnnotationKind::Notice,
                            body: AnnotationBody::Text(notice),
                        }]
                    } else {
                        Vec::new()
                    });
                    generation.try_update(|g| *g += 1);
                }
                Err(e) => {
                    let message = match e.kind {
                        ChatWorkspaceErrorKind::Unavailable => e.message,
                        _ => e.message,
                    };
                    refusal.try_set(Some(message));
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
            if let Some(card) = chosen {
                open_engine(card.engine.id.clone());
            }
        });
    };
    boot();

    // The live turn's record, the notices it has grown, and the watchdog.
    let tick = move || {
        let Some(id) = backend.with_value(|b| b.current_turn_id()) else {
            return;
        };
        let started = match watched.get_untracked() {
            Some((ref seen, at)) if *seen == id => at,
            _ => {
                let at = now.get_untracked();
                watched.set(Some((id.clone(), at)));
                announced.set(0);
                at
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
            turn.try_set(Some(record));
            if !terminal && elapsed >= WATCHDOG_MS {
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
                watched.try_set(None);
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
        // the header already carries the reason.
        if card_for(&id).is_some_and(|c| card_ready_for_ask(&c)) {
            open_engine(id);
        } else {
            refusal.set(Some(texts.get_untracked().not_enabled));
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
            open_engine(id);
        }
    });

    let extra_prefix = id_prefix.clone();
    let settings_extra = ViewFn::from(move || {
        view! {
            <ProviderSettingsRows
                card=active_card
                draft=draft
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
                notices=header_notices
                texts=texts
            />
            <div class="grid w-full grid-cols-1 gap-4 lg:grid-cols-[16rem_1fr_16rem]">
                <KnowledgeSourceRail
                    sources=sources
                    selection=selection
                    on_select=on_knowledge_select
                    texts=texts
                    id_prefix=rail_prefix
                />
                <div class="h-[32rem] min-h-0 overflow-hidden rounded-box border border-base-300 bg-base-100">
                    {panel}
                </div>
                <EvidenceRail turn=turn texts=texts />
            </div>
        </div>
    }
}
