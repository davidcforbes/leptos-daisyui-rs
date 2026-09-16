//! The AI-chat showcase page: the opinionated `AiChatWorkspace` composite
//! over the seeded in-memory backend, with the language and mode switches
//! that later phases of this epic extend.
//!
//! The page body is [`AiChatPage`], which takes the locale it opens in, so
//! `/components/ai-chat` and `/components/ai-chat-es` are the SAME page under
//! two text tables rather than two pages that must be kept in step. The ES
//! route exists so the visual audits see a Spanish layout: Spanish copy is
//! reliably longer than English, and a row that only just fits in EN is the
//! kind of defect no English-only page can show.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::AiChatTexts;
use leptos_daisyui_rs::patterns::{
    AiChatWorkspace, AiChatWorkspaceTexts, AvailabilityReasonCode, ChatWorkspaceBackend,
    CorpusQueryMode, GroundingVerdict, InMemoryChatWorkspaceBackend, IngestPhase, MemoryRefusal,
    ModelSource, ProviderCard, RecallCorpus, ai_chat_honesty_state_for_code, ai_chat_honesty_tone,
    desktop_provider_catalogue,
};
use std::rc::Rc;

/// Every availability reason this workspace names, in the order the
/// vocabulary declares them. `Unknown` is included deliberately: a host code
/// this crate has never heard of still has to render, and showing that it
/// does is the point of the row.
fn every_reason_code() -> Vec<AvailabilityReasonCode> {
    vec![
        AvailabilityReasonCode::TierEffectsDisabled,
        AvailabilityReasonCode::NotArmed,
        AvailabilityReasonCode::CliMissing,
        AvailabilityReasonCode::CliVersionUnsupported,
        AvailabilityReasonCode::CliVersionUntested,
        AvailabilityReasonCode::CliProtocolUnsupported,
        AvailabilityReasonCode::CredentialKeyUnavailable,
        AvailabilityReasonCode::NotSignedIn,
        AvailabilityReasonCode::SignInExpired,
        AvailabilityReasonCode::BudgetExhausted,
        AvailabilityReasonCode::EngineBusy,
        AvailabilityReasonCode::SignInShapeUnavailable,
        AvailabilityReasonCode::EngineProcessNotRunning,
        AvailabilityReasonCode::ModelNotInstalled,
        AvailabilityReasonCode::Unknown("a_host_code_we_do_not_name".to_owned()),
    ]
}

/// The lifecycle ladder, as the id each state publishes on
/// `data-ai-chat-turn-status` paired with the field that labels it.
fn lifecycle_rows(texts: &AiChatWorkspaceTexts) -> Vec<(&'static str, String)> {
    vec![
        ("admitted", texts.state_admitted.clone()),
        ("queued", texts.state_queued.clone()),
        ("running", texts.state_running.clone()),
        ("validating", texts.state_validating.clone()),
        ("completed", texts.state_completed.clone()),
        ("completed", texts.state_declined.clone()),
        ("denied", texts.state_denied.clone()),
        ("unavailable", texts.state_unavailable.clone()),
        ("failed", texts.state_failed.clone()),
        ("canceled", texts.state_canceled_kept.clone()),
        ("canceled", texts.state_canceled_discarded.clone()),
        ("interrupted", texts.state_interrupted.clone()),
    ]
}

/// What a finished turn PRODUCED, which is a different question from what
/// state it is in — and the reason the workspace publishes two hooks.
/// Every typed knowledge id the rail renders, with its localized label, so
/// the page shows the vocabulary rather than describing it.
fn knowledge_rows(texts: &AiChatWorkspaceTexts) -> Vec<(&'static str, String)> {
    let mut rows: Vec<(&'static str, String)> = vec![];
    for phase in [
        IngestPhase::Idle,
        IngestPhase::Walking,
        IngestPhase::Indexing,
        IngestPhase::Clustering,
        IngestPhase::Ready,
    ] {
        rows.push((phase.as_id(), texts.ingest_phase_label(&phase)));
    }
    for mode in [
        CorpusQueryMode::FullText,
        CorpusQueryMode::Similarity,
        CorpusQueryMode::Llm,
        CorpusQueryMode::Fused,
    ] {
        rows.push((mode.as_id(), texts.query_mode_name(mode)));
    }
    for corpus in [
        RecallCorpus::Words,
        RecallCorpus::Meaning,
        RecallCorpus::Graph,
        RecallCorpus::Thread,
    ] {
        let id: &'static str = match corpus {
            RecallCorpus::Words => "words",
            RecallCorpus::Meaning => "meaning",
            RecallCorpus::Graph => "graph",
            _ => "thread",
        };
        rows.push((id, texts.recall_corpus_label(&corpus)));
    }
    rows
}

/// The curation gap and the guardrail, in the words the rail uses. Nothing
/// here may imply that anything extracts a memory: every entry in this
/// workspace was written by an explicit action.
fn curation_rows(texts: &AiChatWorkspaceTexts) -> Vec<(&'static str, String)> {
    let mut rows = vec![
        ("candidate", texts.awaiting_curation.clone()),
        ("confirmed", texts.state_confirmed.clone()),
        ("withdrawn", texts.state_withdrawn.clone()),
    ];
    for refusal in [
        MemoryRefusal::ContainsMatterNumber,
        MemoryRefusal::ContainsEmail,
        MemoryRefusal::ContainsPhone,
        MemoryRefusal::CuratedKindOnly,
    ] {
        rows.push((refusal.as_str(), texts.memory_refusal_label(&refusal)));
    }
    for verdict in [
        GroundingVerdict::Grounded { sources: 1 },
        GroundingVerdict::NotFound,
        GroundingVerdict::AssistantOnly,
    ] {
        rows.push((verdict.as_id(), texts.grounding_label(&verdict)));
    }
    rows
}

fn outcome_rows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("completed", "The engine answered the question."),
        (
            "declined",
            "The engine healthily declined, and said what it could not cover. Never an error.",
        ),
        (
            "truncated",
            "The answer stopped short. Announced, never folded into completed.",
        ),
        ("failed", "We tried and it broke."),
        ("unavailable", "It refused to run."),
        ("canceled", "The actor stopped it."),
    ]
}

/// Which backend the page is driving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkspaceMode {
    /// The seeded in-process fixture.
    Fixture,
    /// A live host backend. Present so the choice is visible, inert until
    /// P9 wires it: selecting it explains itself and leaves the fixture
    /// running, rather than mounting a workspace with nothing behind it.
    Live,
}

/// Which text table both the workspace and the chat panel render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    /// English.
    En,
    /// Spanish.
    Es,
}

fn model_source_label(source: &ModelSource) -> String {
    match source {
        ModelSource::FreeText => "Free text".to_owned(),
        ModelSource::Fixed => "Fixed list".to_owned(),
        ModelSource::Pinned(m) => format!("Pinned to {m}"),
        ModelSource::Discovered { as_of } => format!("Discovered {as_of}"),
        // `ModelSource` is `#[non_exhaustive]`: a shape this page does not
        // name still renders, rather than failing to compile the showcase.
        _ => "Unknown".to_owned(),
    }
}

fn tuning_label(card: &ProviderCard) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if card.tuning.reasoning_effort {
        parts.push("reasoning effort");
    }
    if card.tuning.temperature {
        parts.push("temperature");
    }
    if card.tuning.codex_levers {
        parts.push("Codex levers");
    }
    if card.tuning.permission_mode {
        parts.push("permission mode");
    }
    if parts.is_empty() {
        "No tuning".to_owned()
    } else {
        parts.join(", ")
    }
}

/// How one engine's usage is billed, and therefore whether the workspace can
/// ever show a cost for it.
///
/// Derived from the catalogue rather than stored on a card: an engine that
/// authenticates with a key the host holds is billed per call, and one that
/// runs behind a subscription CLI or on the local machine is not. The
/// workspace's own accounting is the authority for what the numbers MEAN —
/// `UsageTotals` keeps plan and metered apart forever, and
/// `UsageTotals::cost_incomplete` is what makes an accumulated metered cost
/// a floor rather than a total.
fn billing_row(card: &ProviderCard) -> (&'static str, &'static str, &'static str) {
    if card.capabilities.needs_api_key {
        (
            "Metered",
            "Reported per turn, and a floor once any turn arrives unpriced",
            "Inside output",
        )
    } else if matches!(card.model_source, ModelSource::Discovered { .. }) {
        (
            "Neither",
            "Runs on this machine, so nothing is billed and no cost is shown",
            "Inside output",
        )
    } else {
        (
            "Plan",
            "Covered by a subscription, so the workspace shows no per-turn cost",
            "Inside output",
        )
    }
}

/// The showcase page for the AI-chat workspace, in English.
#[component]
pub fn AiChatDemo() -> impl IntoView {
    view! { <AiChatPage initial=Language::En /> }
}

/// The same page, opened in Spanish. Its own route so the layout audits
/// measure a Spanish rendering rather than an English one.
#[component]
pub fn AiChatEsDemo() -> impl IntoView {
    view! { <AiChatPage initial=Language::Es /> }
}

/// The showcase page body.
#[component]
fn AiChatPage(
    /// Which text table the page opens in. The switch below still works;
    /// this only decides where it starts.
    initial: Language,
) -> impl IntoView {
    let mode = RwSignal::new(WorkspaceMode::Fixture);
    let language = RwSignal::new(initial);
    let texts = Signal::derive(move || match language.get() {
        Language::En => AiChatWorkspaceTexts::default(),
        Language::Es => AiChatWorkspaceTexts::es(),
    });
    let chat_texts = Signal::derive(move || match language.get() {
        Language::En => AiChatTexts::default(),
        Language::Es => AiChatTexts::es(),
    });
    // Stored, not captured: `Rc<dyn ChatWorkspaceBackend>` is `!Send`, and
    // `ContentLayout`'s children must be. The handle is `Send`; the value it
    // guards never leaves this thread.
    let backend = StoredValue::new_local(
        Rc::new(InMemoryChatWorkspaceBackend::seeded()) as Rc<dyn ChatWorkspaceBackend>
    );

    let mode_radio = move |value: WorkspaceMode, label: &'static str| {
        // Only the Live option carries `data-ai-chat-live-mode`; `None`
        // omits the attribute entirely, so the hook names one control rather
        // than both with a value a proof would have to read.
        let live_hook = (value == WorkspaceMode::Live).then_some("");
        view! {
            <label class="flex items-center gap-2 text-sm">
                <input
                    type="radio"
                    name="ai-chat-mode"
                    class="radio radio-sm"
                    data-ai-chat-live-mode=live_hook
                    prop:checked=move || mode.get() == value
                    on:change=move |_| mode.set(value)
                />
                <span>{label}</span>
            </label>
        }
    };
    let language_radio = move |value: Language, label: &'static str| {
        view! {
            <label class="flex items-center gap-2 text-sm">
                <input
                    type="radio"
                    name="ai-chat-language"
                    class="radio radio-sm"
                    prop:checked=move || language.get() == value
                    on:change=move |_| language.set(value)
                />
                <span>{label}</span>
            </label>
        }
    };

    let provider_cards = move || {
        desktop_provider_catalogue()
            .into_iter()
            .map(|card| {
                let picker = card.picker_label.clone();
                let label = card.capabilities.label.clone();
                let models = card.capabilities.models.len();
                let source = model_source_label(&card.model_source);
                let tuning = tuning_label(&card);
                let thinking = card.capabilities.supports_thinking;
                let tools = card.capabilities.supports_tool_calls;
                let key = card.capabilities.needs_api_key;
                view! {
                    <li
                        class="flex flex-col gap-1 rounded-box border border-base-300 bg-base-100 p-4"
                        data-ai-chat-provider-card=card.engine.id.clone()
                    >
                        <span class="text-sm font-semibold">{picker}</span>
                        <span class="text-xs opacity-60">{label}</span>
                        <span class="text-xs opacity-70">
                            {format!("{models} model(s) \u{b7} {source}")}
                        </span>
                        <span class="text-xs opacity-70">{tuning}</span>
                        <span class="text-xs opacity-70">
                            {format!(
                                "thinking: {} \u{b7} tools: {} \u{b7} key: {}",
                                if thinking { "yes" } else { "no" },
                                if tools { "yes" } else { "no" },
                                if key { "host-held" } else { "none" },
                            )}
                        </span>
                    </li>
                }
            })
            .collect_view()
    };

    view! {
        <ContentLayout
            title="AiChatWorkspace"
            description="The opinionated chat workspace: an engine header, the knowledge rail, the generic AiChat panel and the evidence rail, all over one ChatWorkspaceBackend. This page drives the seeded in-memory fixture, so every engine, document and turn below is reproducible."
        >
            <Section title="Backend" col=true>
                <div class="flex flex-wrap items-center gap-4" data-testid="ai-chat-mode">
                    {mode_radio(WorkspaceMode::Fixture, "Fixture")}
                    {mode_radio(WorkspaceMode::Live, "Live")}
                </div>
                <Show when=move || mode.get() == WorkspaceMode::Live>
                    <p class="text-sm opacity-70" data-ai-chat-live-mode-reason="">
                        "A live backend is not wired up on this page yet, so the fixture keeps \
                         running. Nothing below changes until it is."
                    </p>
                </Show>
            </Section>

            <Section title="Language" col=true>
                <div class="flex flex-wrap items-center gap-4" data-testid="ai-chat-language">
                    {language_radio(Language::En, "EN")}
                    {language_radio(Language::Es, "ES")}
                </div>
                <p class="text-sm opacity-70">
                    "Both text tables swap in place: the workspace's own copy and the chat \
                     panel's. The workspace root carries the active table's locale id on \
                     data-ai-chat-workspace-locale."
                </p>
            </Section>

            <Section title="Workspace" col=true>
                <p class="text-sm opacity-70">
                    "The quick-action bar under the header seeds the composer and sends \
                     nothing, so a canned prompt is a starting point the actor edits. Seven \
                     actions are static chips; Translate is not one of them, because its \
                     prompt is the only one that is not static copy \u{2014} it needs a \
                     language, so it carries its own input and stays disabled until one is \
                     named. Every chip's label and the prompt it fills come from the same \
                     text table, so both swap with the language switch above."
                </p>
                <div class="w-full" data-testid="ai-chat-workspace">
                    {move || {
                        view! {
                            <AiChatWorkspace
                                backend=backend.get_value()
                                texts=texts
                                chat_texts=chat_texts
                            />
                        }
                    }}
                </div>
            </Section>

            <Section title="Usage and cost" col=true>
                <p class="text-sm opacity-70">
                    "Plan-covered and metered usage are never added together: they answer two \
                     different questions, and a single total would misreport both. Reasoning \
                     tokens are a SUBSET of output tokens for every engine here, never an \
                     addition to them, so a workspace that summed the two would double-count \
                     the thinking. A metered cost the backend never reported is shown as \
                     absent rather than as zero, and once any turn in a session arrives \
                     unpriced the accumulated figure stays a floor for the rest of that \
                     session's life. The live counters for the running conversation are in \
                     the workspace header above; this table is the vocabulary behind them."
                </p>
                <div class="w-full overflow-x-auto" data-testid="ai-chat-usage">
                    <table class="table table-sm">
                        <thead>
                            <tr>
                                <th>"Engine"</th>
                                <th>"Billing"</th>
                                <th>"Reasoning tokens"</th>
                                <th>"Cost known?"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {desktop_provider_catalogue()
                                .into_iter()
                                .map(|card| {
                                    let (billing, cost, reasoning) = billing_row(&card);
                                    let picker = card.picker_label.clone();
                                    view! {
                                        <tr data-ai-chat-usage-row=card.engine.id.clone()>
                                            <td class="font-semibold">{picker}</td>
                                            <td data-ai-chat-usage-billing=billing>{billing}</td>
                                            <td class="opacity-70">{reasoning}</td>
                                            <td class="opacity-70">{cost}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>
            </Section>

            <Section title="Knowledge sources" col=true>
                <p class="text-sm opacity-70">
                    "Four knowledge sources mix into a turn, and each one is typed rather \
                     than a string. A corpus carries an ingest phase (idle, walking, \
                     indexing, clustering, ready, failed) and is queried one of four ways. \
                     A grounded turn that matches nothing answers that it found nothing, \
                     rather than guessing. Personal memory is recalled through a receipt \
                     that names the lane every hit came from, and an entry an actor writes \
                     is a candidate until a curator confirms it \u{2014} written, and not \
                     yet findable."
                </p>
                <div class="grid w-full grid-cols-1 gap-4 lg:grid-cols-2">
                    <ul class="flex flex-col gap-2" data-testid="ai-chat-knowledge">
                        {move || {
                            let t = texts.get();
                            knowledge_rows(&t)
                                .into_iter()
                                .map(|(id, label)| {
                                    view! {
                                        <li
                                            class="flex items-center justify-between gap-3 rounded-box border border-base-300 bg-base-100 p-3 text-sm"
                                            data-ai-chat-knowledge-row=id
                                        >
                                            <span class="font-mono text-xs opacity-60">{id}</span>
                                            <span>{label}</span>
                                        </li>
                                    }
                                })
                                .collect_view()
                        }}
                    </ul>
                    <ul class="flex flex-col gap-2" data-testid="ai-chat-curation">
                        {move || {
                            let t = texts.get();
                            curation_rows(&t)
                                .into_iter()
                                .map(|(id, meaning)| {
                                    view! {
                                        <li
                                            class="flex flex-col gap-1 rounded-box border border-base-300 bg-base-100 p-3 text-sm"
                                            data-ai-chat-curation-row=id
                                        >
                                            <span class="font-mono text-xs opacity-60">{id}</span>
                                            <span class="text-xs opacity-70">{meaning}</span>
                                        </li>
                                    }
                                })
                                .collect_view()
                        }}
                    </ul>
                </div>
            </Section>

            <Section title="Honesty" col=true>
                <p class="text-sm opacity-70">
                    "Every availability reason is typed, and each one maps to an honesty state, \
                     a tone and exactly one next step. Three states are kept apart on purpose: \
                     not_enabled (the account or host says no and nothing was attempted), \
                     unavailable (something was attempted and refused to run) and failed \
                     (something was attempted and broke). A host code this vocabulary has never \
                     seen still renders, carrying the host's own code."
                </p>
                <ul
                    class="grid w-full grid-cols-1 gap-3 lg:grid-cols-2"
                    data-testid="ai-chat-honesty"
                >
                    {move || {
                        let t = texts.get();
                        every_reason_code()
                            .into_iter()
                            .map(|code| {
                                let state = ai_chat_honesty_state_for_code(&code);
                                let tone = ai_chat_honesty_tone(state);
                                let action = code.next_action().as_str();
                                let copy = t.availability_reason(&code);
                                let id = code.as_code().to_owned();
                                let label = id.clone();
                                view! {
                                    <li
                                        class="flex flex-col gap-1 rounded-box border border-base-300 bg-base-100 p-4"
                                        data-ai-chat-reason-row=id
                                    >
                                        <span class=tone>{state}</span>
                                        <span class="text-xs font-semibold">{label}</span>
                                        <span class="text-xs opacity-70">{copy}</span>
                                        <span class="text-xs opacity-60">{action}</span>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Section>

            <Section title="Lifecycle" col=true>
                <p class="text-sm opacity-70">
                    "A turn walks admitted, queued, running, validating and then one terminal \
                     state. The workspace publishes where it is on data-ai-chat-turn-status and \
                     what it produced on data-ai-chat-outcome, because those are two different \
                     questions: a completed turn may still have declined or been cut short."
                </p>
                <div class="grid w-full grid-cols-1 gap-4 lg:grid-cols-2">
                    <ul class="flex flex-col gap-2" data-testid="ai-chat-lifecycle">
                        {move || {
                            let t = texts.get();
                            lifecycle_rows(&t)
                                .into_iter()
                                .map(|(id, label)| {
                                    view! {
                                        <li
                                            class="flex items-center justify-between gap-3 rounded-box border border-base-300 bg-base-100 p-3 text-sm"
                                            data-ai-chat-lifecycle-row=id
                                        >
                                            <span class="font-mono text-xs opacity-60">{id}</span>
                                            <span>{label}</span>
                                        </li>
                                    }
                                })
                                .collect_view()
                        }}
                    </ul>
                    <ul class="flex flex-col gap-2" data-testid="ai-chat-outcomes">
                        {outcome_rows()
                            .into_iter()
                            .map(|(id, meaning)| {
                                view! {
                                    <li
                                        class="flex flex-col gap-1 rounded-box border border-base-300 bg-base-100 p-3 text-sm"
                                        data-ai-chat-outcome-row=id
                                    >
                                        <span class="font-mono text-xs opacity-60">{id}</span>
                                        <span class="text-xs opacity-70">{meaning}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </div>
            </Section>

            <Section title="Providers" col=true>
                <ul
                    class="grid w-full grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3"
                    data-testid="ai-chat-providers"
                >
                    {provider_cards}
                </ul>
            </Section>
        </ContentLayout>
    }
}
