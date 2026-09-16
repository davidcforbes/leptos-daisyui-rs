//! The quick-action chips a chat composer offers, and the request prompt
//! each one seeds into the input box before the actor edits or sends it.

use leptos::prelude::*;

use super::texts::AiChatWorkspaceTexts;

/// A canned rewrite/expand-style request the composer can pre-fill.
/// `Translate` carries the target language because, unlike the rest, its
/// prompt is not static copy.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuickAction {
    /// Summarize the current answer or document.
    Summarize,
    /// Rewrite the current answer or document.
    Rewrite,
    /// Expand on the current answer or document.
    Expand,
    /// Fix grammar in the current answer or document.
    FixGrammar,
    /// Simplify the current answer or document.
    Simplify,
    /// Add more detail to the current answer or document.
    AddDetails,
    /// Convert the current answer or document to a table.
    ConvertToTable,
    /// Translate the current answer or document into an explicit language.
    Translate {
        /// The requested target language.
        language: String,
    },
}

impl QuickAction {
    /// Stable id for `data-*` hooks and telemetry.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Summarize => "summarize",
            Self::Rewrite => "rewrite",
            Self::Expand => "expand",
            Self::FixGrammar => "fix_grammar",
            Self::Simplify => "simplify",
            Self::AddDetails => "add_details",
            Self::ConvertToTable => "convert_to_table",
            Self::Translate { .. } => "translate",
        }
    }

    /// Every action that needs no free-form parameter, in a fixed-size
    /// array a composer can render as a static chip row. `Translate` is
    /// deliberately excluded — it needs a language, so it renders through
    /// its own control instead.
    pub fn all_static() -> [QuickAction; 7] {
        [
            Self::Summarize,
            Self::Rewrite,
            Self::Expand,
            Self::FixGrammar,
            Self::Simplify,
            Self::AddDetails,
            Self::ConvertToTable,
        ]
    }

    /// The name of the [`AiChatWorkspaceTexts`] field this action's prompt
    /// is read from, exactly as `AiChatWorkspaceTexts::fields` spells it.
    ///
    /// It exists so a rendered chip can publish WHICH field it came from on
    /// `data-ai-chat-label`, which is what lets a locale proof compare the
    /// DOM against the text table by name instead of by position.
    pub fn texts_field(&self) -> &'static str {
        match self {
            Self::Summarize => "qa_summarize",
            Self::Rewrite => "qa_rewrite",
            Self::Expand => "qa_expand",
            Self::FixGrammar => "qa_fix_grammar",
            Self::Simplify => "qa_simplify",
            Self::AddDetails => "qa_add_details",
            Self::ConvertToTable => "qa_convert_to_table",
            Self::Translate { .. } => "qa_translate",
        }
    }

    /// The composer prompt this action seeds, localized by `texts`.
    pub fn prompt(&self, texts: &AiChatWorkspaceTexts) -> String {
        match self {
            Self::Summarize => texts.qa_summarize.clone(),
            Self::Rewrite => texts.qa_rewrite.clone(),
            Self::Expand => texts.qa_expand.clone(),
            Self::FixGrammar => texts.qa_fix_grammar.clone(),
            Self::Simplify => texts.qa_simplify.clone(),
            Self::AddDetails => texts.qa_add_details.clone(),
            Self::ConvertToTable => texts.qa_convert_to_table.clone(),
            Self::Translate { language } => texts.qa_translate.replace("{language}", language),
        }
    }
}

/// The quick-action bar: seven static chips plus the `Translate` control.
///
/// Every chip FILLS the composer and sends nothing. A quick action is a
/// starting point the actor then edits — a bar that sent would turn a
/// mis-click into a turn, and there would be no way to add the sentence the
/// canned prompt is missing.
///
/// `Translate` is not an eighth chip. Its prompt is the only one that is not
/// static copy: `QuickAction::Translate` carries a language, so the bar
/// renders a labelled text input beside its own button and keeps that button
/// disabled until the actor has actually named a language. A uniform eighth
/// chip could only ever substitute an empty string into
/// `AiChatWorkspaceTexts::qa_translate` and seed "Translate this to ".
///
/// Every element whose text is exactly one text-table field publishes that
/// field's name on `data-ai-chat-label`, so a locale proof can assert the
/// rendering against the table BY NAME. The composer's own value is
/// deliberately not tagged: it holds a prompt, not a label.
///
/// ```ignore
/// /// @source inline("btn btn-xs btn-outline input input-xs input-bordered");
/// ```
#[component]
pub fn QuickActionBar(
    /// Localized copy. Both the chip labels and the prompts they seed come
    /// from here, so the bar localizes with the rest of the workspace.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
    /// The composer text a chip fills. Owned by the composite and handed to
    /// the chat panel, so a chip and the actor's own typing write the same
    /// place.
    composer: RwSignal<String>,
    /// Unique prefix for the control ids this bar mints, so two mounted
    /// workspaces never collide on a `label for=` target.
    #[prop(into)]
    id_prefix: String,
) -> impl IntoView {
    // Stored, not captured: a `String` captured by move makes the id closure
    // `FnOnce`, and a `view!` child closure must be `Fn`.
    let prefix = StoredValue::new(id_prefix);
    let language_id = move || prefix.with_value(|p| format!("{p}-qa-language"));
    let language = RwSignal::new(String::new());

    let chips = move || {
        let t = texts.get();
        QuickAction::all_static()
            .into_iter()
            .map(|action| {
                let field = action.texts_field();
                let id = action.as_id();
                let prompt = action.prompt(&t);
                let seeded = prompt.clone();
                view! {
                    <button
                        type="button"
                        class="btn btn-xs btn-outline"
                        data-ai-chat-quick-action=id
                        data-ai-chat-label=field
                        on:click=move |_| composer.set(seeded.clone())
                    >
                        {prompt}
                    </button>
                }
            })
            .collect_view()
    };

    let translate = move |_| {
        let action = QuickAction::Translate {
            language: language.get_untracked(),
        };
        composer.set(action.prompt(&texts.get_untracked()));
    };

    view! {
        <div
            class="flex flex-wrap items-center gap-2"
            role="group"
            aria-label=move || texts.get().quick_actions_label
            data-ai-chat-quick-actions=""
        >
            <span class="text-xs opacity-60" data-ai-chat-label="quick_actions_label">
                {move || texts.get().quick_actions_label}
            </span>
            {chips}
            <label class="text-xs opacity-60" for=language_id data-ai-chat-label="qa_language_label">
                {move || texts.get().qa_language_label}
            </label>
            <input
                id=language_id
                type="text"
                class="input input-xs input-bordered w-32"
                data-ai-chat-quick-action-language=""
                prop:value=move || language.get()
                on:input=move |e| language.set(event_target_value(&e))
            />
            <button
                type="button"
                class="btn btn-xs btn-outline"
                data-ai-chat-quick-action="translate"
                data-ai-chat-label="qa_translate_button"
                prop:disabled=move || language.get().trim().is_empty()
                on:click=translate
            >
                {move || texts.get().qa_translate_button}
            </button>
        </div>
    }
}
