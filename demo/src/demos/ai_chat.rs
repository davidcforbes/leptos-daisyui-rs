//! The AI-chat showcase page: the opinionated `AiChatWorkspace` composite
//! over the seeded in-memory backend, with the language and mode switches
//! that later phases of this epic extend.
//!
//! Sections honesty / lifecycle / knowledge / usage arrive in P5-P7; this
//! page deliberately ships only what it can demonstrate today.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::AiChatTexts;
use leptos_daisyui_rs::patterns::{
    AiChatWorkspace, AiChatWorkspaceTexts, ChatWorkspaceBackend, InMemoryChatWorkspaceBackend,
    ModelSource, ProviderCard, desktop_provider_catalogue,
};
use std::rc::Rc;

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
