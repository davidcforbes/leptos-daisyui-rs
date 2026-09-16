//! The AI-chat showcase page: the opinionated `AiChatWorkspace` composite
//! over the seeded in-memory backend, with the language and mode switches
//! that later phases of this epic extend.
//!
//! Sections knowledge / usage / quick actions arrive in P6-P7; this page
//! deliberately ships only what it can demonstrate today.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::AiChatTexts;
use leptos_daisyui_rs::patterns::{
    AiChatWorkspace, AiChatWorkspaceTexts, AvailabilityReasonCode, ChatWorkspaceBackend,
    InMemoryChatWorkspaceBackend, ModelSource, ProviderCard, ai_chat_honesty_state_for_code,
    ai_chat_honesty_tone, desktop_provider_catalogue,
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

/// The showcase page for the AI-chat workspace.
#[component]
pub fn AiChatDemo() -> impl IntoView {
    let mode = RwSignal::new(WorkspaceMode::Fixture);
    let language = RwSignal::new(Language::En);
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
        view! {
            <label class="flex items-center gap-2 text-sm">
                <input
                    type="radio"
                    name="ai-chat-mode"
                    class="radio radio-sm"
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
