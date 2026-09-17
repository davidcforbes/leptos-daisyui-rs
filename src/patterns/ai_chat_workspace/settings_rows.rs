//! The provider-specific settings rows `AiChatWorkspace` feeds into
//! `components::ai_chat::AiChat`'s `settings_extra` slot, plus the draft
//! state they edit.
//!
//! `AiChat` stays provider-blind: it renders the generic rows (model, system
//! prompt, allowed tools, the two visibility toggles) from the selected
//! backend's `Capabilities`, and everything an engine knows by name arrives
//! through this slot. Which rows exist is decided by the card's
//! [`TuningSchema`] — the CAPABILITY to tune — never by which fields happen
//! to be populated on the currently applied [`ProviderTuning`], which only
//! says what IS tuned right now.

use leptos::prelude::*;

use super::provider::{CodexLevers, GroqTuning, ProviderCard, ProviderTuning, ReasoningEffort};
use super::texts::AiChatWorkspaceTexts;

/// The three reasoning-effort levels a picker offers, in ascending order.
/// `ReasoningEffort::Unknown` is deliberately absent: it exists to preserve a
/// host value this vocabulary does not name, not to be offered as a choice.
pub const EFFORT_CHOICES: [ReasoningEffort; 3] = [
    ReasoningEffort::Low,
    ReasoningEffort::Medium,
    ReasoningEffort::High,
];

/// Which option index the reasoning-effort select should mark `selected`, or
/// `None` when the draft holds no effort and the leading engine-default
/// option must be chosen instead.
///
/// The same rule, and the same reason, as `ai_chat`'s own
/// `permission_selection`: with three options and nothing selected a browser
/// displays the FIRST one, so an unchosen effort renders as "low" while
/// `TuningDraft::to_tuning` sends `None` and the engine runs at its own
/// default. A control must never display a level the host never chose.
/// `ReasoningEffort::Unknown` deliberately matches nothing here — it
/// preserves a host value this vocabulary cannot offer as a choice, so it
/// also falls back to the engine-default option rather than silently
/// selecting a level it is not.
pub fn effort_selection(chosen: Option<&ReasoningEffort>) -> Option<usize> {
    let chosen = chosen?;
    if matches!(chosen, ReasoningEffort::Unknown(_)) {
        return None;
    }
    EFFORT_CHOICES
        .iter()
        .position(|e| e.as_str() == chosen.as_str())
}

/// Live, per-engine tuning state the settings rows edit.
///
/// Held as separate signals rather than one `RwSignal<ProviderTuning>` so a
/// control can write its own field without reading and rewriting the others,
/// and so [`Self::to_tuning`] is the single place a schema gate is applied.
#[derive(Clone, Copy)]
pub struct TuningDraft {
    /// The chosen reasoning effort, or `None` for the engine's own default.
    pub effort: RwSignal<Option<ReasoningEffort>>,
    /// The Groq sampling temperature.
    pub temperature: RwSignal<f32>,
    /// The Codex CLI execution levers.
    pub codex: RwSignal<CodexLevers>,
    /// The chosen CLI permission mode, or `None` when the engine has none.
    pub permission_mode: RwSignal<Option<String>>,
}

impl Default for TuningDraft {
    fn default() -> Self {
        Self::new()
    }
}

impl TuningDraft {
    /// A draft with every lever at its neutral value.
    pub fn new() -> Self {
        Self {
            effort: RwSignal::new(None),
            temperature: RwSignal::new(DEFAULT_TEMPERATURE),
            codex: RwSignal::new(CodexLevers::default()),
            permission_mode: RwSignal::new(None),
        }
    }

    /// Reset the draft to the card's own starting point, BEFORE the settings
    /// popover renders.
    ///
    /// The permission seed matters: `AiChat` renders a disabled "not set"
    /// placeholder for `None`, so a host that leaves it empty shows a
    /// permission control claiming nothing is chosen while the engine runs at
    /// whatever it defaults to. The card's `default` mode is preferred when it
    /// publishes one, else its first published mode.
    pub fn seed_from(&self, card: &ProviderCard) {
        self.effort.set(None);
        self.temperature.set(DEFAULT_TEMPERATURE);
        self.codex.set(CodexLevers::default());
        let modes = &card.capabilities.permission_modes;
        let seeded = if !card.tuning.permission_mode || modes.is_empty() {
            None
        } else if modes.iter().any(|m| m == "default") {
            Some("default".to_owned())
        } else {
            modes.first().cloned()
        };
        self.permission_mode.set(seeded);
    }

    /// The tuning payload for one card, with every lever the card's
    /// [`TuningSchema`](super::TuningSchema) does not declare left `None`.
    ///
    /// The gate is why switching engines cannot smuggle the previous engine's
    /// levers into the next one's session: a Groq temperature is simply not
    /// representable in a Codex payload, whatever the draft still holds.
    pub fn to_tuning(&self, card: &ProviderCard) -> ProviderTuning {
        ProviderTuning {
            permission_mode: card
                .tuning
                .permission_mode
                .then(|| self.permission_mode.get_untracked())
                .flatten(),
            reasoning_effort: card
                .tuning
                .reasoning_effort
                .then(|| self.effort.get_untracked())
                .flatten(),
            groq: card.tuning.temperature.then(|| GroqTuning {
                temperature: self.temperature.get_untracked(),
            }),
            codex: card.tuning.codex_levers.then(|| self.codex.get_untracked()),
        }
    }
}

/// The temperature a Groq card starts at when nothing has been chosen.
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Which Codex lever one checkbox owns. The wire id is the checkbox's
/// `data-ai-chat-codex-lever` value, so a proof names a lever rather than a
/// position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodexLever {
    /// The CLI's built-in web-search tool.
    WebSearch,
    /// Suppression of the CLI's plugin surface.
    SuppressPlugins,
    /// Disabling the CLI's sandboxed code-execution mode.
    DisableCodeMode,
}

impl CodexLever {
    /// Every lever, in render order.
    pub const ALL: [CodexLever; 3] = [
        CodexLever::WebSearch,
        CodexLever::SuppressPlugins,
        CodexLever::DisableCodeMode,
    ];

    /// The `data-ai-chat-codex-lever` value for this lever.
    pub fn as_id(&self) -> &'static str {
        match self {
            CodexLever::WebSearch => "web_search",
            CodexLever::SuppressPlugins => "suppress_plugins",
            CodexLever::DisableCodeMode => "disable_code_mode",
        }
    }

    /// Read this lever out of a [`CodexLevers`] value.
    pub fn read(&self, levers: &CodexLevers) -> bool {
        match self {
            CodexLever::WebSearch => levers.web_search,
            CodexLever::SuppressPlugins => levers.suppress_plugins,
            CodexLever::DisableCodeMode => levers.disable_code_mode,
        }
    }

    /// Write this lever into a [`CodexLevers`] value.
    pub fn write(&self, levers: &mut CodexLevers, value: bool) {
        match self {
            CodexLever::WebSearch => levers.web_search = value,
            CodexLever::SuppressPlugins => levers.suppress_plugins = value,
            CodexLever::DisableCodeMode => levers.disable_code_mode = value,
        }
    }

    /// This lever's localized label. It is a visible string inside the
    /// workspace root, so it lives in the workspace's own text table — a
    /// phase that ships a string owns its texts field.
    pub fn label(&self, texts: &AiChatWorkspaceTexts) -> String {
        match self {
            CodexLever::WebSearch => texts.codex_web_search.clone(),
            CodexLever::SuppressPlugins => texts.codex_suppress_plugins.clone(),
            CodexLever::DisableCodeMode => texts.codex_disable_code_mode.clone(),
        }
    }
}

/// The provider-specific settings rows for one card.
///
/// Every control carries a real `label for=` association against an id minted
/// from `id_prefix`, which the composite derives from its own instance
/// sequence so two mounted workspaces never collide.
///
/// `on_change` fires after every edit, so the composite can push the new
/// tuning at the backend immediately: `AiChat`'s Apply button is wired to
/// `ChatSession::configure` and has no host callback, so a lever that waited
/// for Apply would never reach `set_tuning` at all.
#[component]
pub fn ProviderSettingsRows(
    /// The currently selected card, or `None` before providers have loaded.
    #[prop(into)]
    card: Signal<Option<ProviderCard>>,
    /// The draft these rows edit.
    draft: TuningDraft,
    /// Localized copy. Every visible string these rows render comes from
    /// here; none of them is an English literal.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
    /// Unique prefix for the control ids these rows mint.
    #[prop(into)]
    id_prefix: String,
    /// Fired after any lever changes.
    on_change: Callback<()>,
) -> impl IntoView {
    // Stored, not captured: a `String` captured by move makes every id
    // closure `FnOnce`, and a `view!` child closure must be `Fn`.
    let prefix = StoredValue::new(id_prefix);
    let effort_id = move || prefix.with_value(|p| format!("{p}-effort"));
    let temperature_id = move || prefix.with_value(|p| format!("{p}-temperature"));
    let lever_id =
        move |lever: CodexLever| prefix.with_value(|p| format!("{p}-codex-{}", lever.as_id()));

    let shows_effort = move || card.get().is_some_and(|c| c.tuning.reasoning_effort);
    let shows_temperature = move || card.get().is_some_and(|c| c.tuning.temperature);
    let shows_levers = move || card.get().is_some_and(|c| c.tuning.codex_levers);
    let needs_key = move || card.get().is_some_and(|c| c.capabilities.needs_api_key);

    let effort_options = move || {
        let chosen = draft.effort.get();
        let selected_index = effort_selection(chosen.as_ref());
        // The leading option is what the control shows when the host has
        // chosen no effort, so the browser can never present its first real
        // level as though it had been picked (see `effort_selection`).
        let engine_default = {
            let label = texts.get().effort_engine_default;
            let selected = selected_index.is_none();
            view! { <option value=String::new() selected=selected>{label}</option> }
        };
        let levels = EFFORT_CHOICES
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let value = e.as_str().to_owned();
                let selected = selected_index == Some(i);
                let label = value.clone();
                view! { <option value=value selected=selected>{label}</option> }
            })
            .collect_view();
        view! { {engine_default} {levels} }
    };

    let levers = move || {
        let levers_now = draft.codex.get();
        CodexLever::ALL
            .iter()
            .map(|lever| {
                let lever = *lever;
                let id = lever_id(lever);
                let label_for = id.clone();
                let label = lever.label(&texts.get());
                let checked = lever.read(&levers_now);
                view! {
                    <label class="flex items-center justify-between gap-2 text-xs" for=label_for>
                        <span class="opacity-60">{label}</span>
                        <input
                            id=id
                            type="checkbox"
                            class="checkbox checkbox-sm"
                            data-ai-chat-codex-lever=lever.as_id()
                            prop:checked=checked
                            on:change=move |e| {
                                let value = event_target_checked(&e);
                                draft.codex.update(|l| lever.write(l, value));
                                on_change.run(());
                            }
                        />
                    </label>
                }
            })
            .collect_view()
    };

    view! {
        <div class="flex flex-col gap-2" data-ai-chat-provider-rows="">
            <Show when=shows_effort>
                <label class="flex flex-col gap-1 text-xs" for=effort_id>
                    <span class="opacity-60" data-ai-chat-label="effort_label">
                        {move || texts.get().effort_label}
                    </span>
                    <select
                        id=effort_id
                        data-ai-chat-effort-select=""
                        class="select select-sm select-bordered w-full"
                        on:change=move |e| {
                            let picked = event_target_value(&e);
                            // The engine-default option carries an empty
                            // value; it means "no effort chosen", not an
                            // effort named "".
                            draft
                                .effort
                                .set((!picked.is_empty()).then(|| ReasoningEffort::parse(&picked)));
                            on_change.run(());
                        }
                    >
                        {effort_options}
                    </select>
                </label>
            </Show>
            <Show when=shows_temperature>
                <label class="flex flex-col gap-1 text-xs" for=temperature_id>
                    <span class="opacity-60">
                        {move || {
                            format!(
                                "{} {:.2}",
                                texts.get().temperature_label,
                                draft.temperature.get(),
                            )
                        }}
                    </span>
                    <input
                        id=temperature_id
                        type="range"
                        class="range range-xs"
                        min="0"
                        max="2"
                        step="0.05"
                        data-ai-chat-temperature=""
                        prop:value=move || draft.temperature.get().to_string()
                        on:input=move |e| {
                            if let Ok(v) = event_target_value(&e).parse::<f32>() {
                                draft.temperature.set(v);
                                on_change.run(());
                            }
                        }
                    />
                </label>
            </Show>
            <Show when=shows_levers>
                <div class="flex flex-col gap-2">
                    {levers}
                    <p
                        class="text-xs opacity-60"
                        data-ai-chat-no-mcp=""
                        data-ai-chat-label="codex_no_mcp"
                    >
                        {move || texts.get().codex_no_mcp}
                    </p>
                </div>
            </Show>
            <Show when=needs_key>
                <p
                    class="text-xs opacity-60"
                    data-ai-chat-credential-note=""
                    data-ai-chat-label="credential_note"
                >
                    {move || texts.get().credential_note}
                </p>
            </Show>
        </div>
    }
}
