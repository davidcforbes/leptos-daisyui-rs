//! `AiChatWorkspace`'s left rail: which corpus a turn is grounded in, how it
//! is queried, and whether the engine may fall back to general assistance.
//!
//! This task renders the SELECTION controls only. Ingest phases, reindex,
//! recall receipts, the remember flow and the office-knowledge toggles are
//! P6's; the extension points below are named rather than stubbed, so nothing
//! here claims to do something it does not.

use leptos::prelude::*;

use super::knowledge::{
    ChatPosture, CorpusQueryMode, CorpusScope, KnowledgeSelection, KnowledgeSource,
};
use super::texts::AiChatWorkspaceTexts;

/// Every corpus scope the rail can offer, derived from what the host actually
/// published — never a hardcoded list, so a host with one folder shows one
/// option.
pub fn corpus_choices(sources: &[KnowledgeSource]) -> Vec<CorpusScope> {
    sources
        .iter()
        .filter_map(|s| match s {
            KnowledgeSource::Corpus { scope, .. } => Some(scope.clone()),
            _ => None,
        })
        .collect()
}

/// The display label for one corpus scope: the host's own path or custom
/// label, falling back to the localized "everything" wording for
/// [`CorpusScope::All`], which has no path of its own.
pub fn scope_label(scope: &CorpusScope, texts: &AiChatWorkspaceTexts) -> String {
    match scope {
        CorpusScope::File(path) | CorpusScope::Folder(path) => path.clone(),
        CorpusScope::All => texts.scope_all.clone(),
        CorpusScope::Custom { label, .. } => label.clone(),
    }
}

/// A stable, per-scope selection value. [`CorpusScope::as_id`] collapses every
/// folder to `"folder"`, which cannot distinguish two folders, so the rail
/// keys on the id plus the path.
pub fn scope_value(scope: &CorpusScope) -> String {
    match scope {
        CorpusScope::File(path) | CorpusScope::Folder(path) => {
            format!("{}:{path}", scope.as_id())
        }
        CorpusScope::Custom { label, .. } => format!("custom:{label}"),
        other => other.as_id().to_owned(),
    }
}

/// Whether one control's visible label is a FIELD label rather than a repeat
/// of one of its own options.
///
/// A select whose accessible name is one of its options ("Fused", "Grounded
/// in this folder") tells a screen-reader user what is currently chosen and
/// nothing about what the control decides. Every rail control is checked
/// against this in `knowledge_rail_label_is_never_one_of_its_own_options`,
/// and the browser suite reads the same property off the rendered DOM.
pub fn label_is_distinct_from_options(label: &str, options: &[String]) -> bool {
    !label.trim().is_empty() && !options.iter().any(|o| o.trim() == label.trim())
}

/// The knowledge-selection rail.
#[component]
pub fn KnowledgeSourceRail(
    /// Everything the host published.
    #[prop(into)]
    sources: Signal<Vec<KnowledgeSource>>,
    /// The live selection.
    #[prop(into)]
    selection: Signal<KnowledgeSelection>,
    /// Fired with the selection the actor asked for. The composite decides
    /// what that costs — a corpus or posture change reopens the session.
    on_select: Callback<KnowledgeSelection>,
    /// Localized copy.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
    /// Unique prefix for the control ids this rail mints.
    #[prop(into)]
    id_prefix: String,
) -> impl IntoView {
    // Stored, not captured: see `settings_rows.rs` for why a captured
    // `String` makes every id closure `FnOnce`.
    let prefix = StoredValue::new(id_prefix);
    let scope_id = move || prefix.with_value(|p| format!("{p}-corpus-scope"));
    let mode_id = move || prefix.with_value(|p| format!("{p}-query-mode"));
    let posture_id = move || prefix.with_value(|p| format!("{p}-posture"));

    let scope_options = move || {
        let t = texts.get();
        let current = selection.get().corpus;
        let current_value = current.as_ref().map(scope_value);
        let none_selected = current_value.is_none();
        let mut rows = vec![view! {
            <option value=String::new() selected=none_selected>
                {t.scope_none.clone()}
            </option>
        }];
        for scope in corpus_choices(&sources.get()) {
            let value = scope_value(&scope);
            let selected = current_value.as_deref() == Some(value.as_str());
            let label = scope_label(&scope, &t);
            rows.push(view! { <option value=value selected=selected>{label}</option> });
        }
        rows.collect_view()
    };

    let mode_options = move || {
        let t = texts.get();
        let active = selection.get().query_mode;
        [
            (CorpusQueryMode::FullText, t.query_full_text.clone()),
            (CorpusQueryMode::Similarity, t.query_similarity.clone()),
            (CorpusQueryMode::Llm, t.query_llm.clone()),
            (CorpusQueryMode::Fused, t.query_fused.clone()),
        ]
        .into_iter()
        .map(|(mode, label)| {
            let value = mode.as_id().to_owned();
            let selected = mode == active;
            view! { <option value=value selected=selected>{label}</option> }
        })
        .collect_view()
    };

    let posture_options = move || {
        let t = texts.get();
        let active = selection.get().posture;
        [
            (ChatPosture::Grounded, t.posture_grounded.clone()),
            (ChatPosture::Assistant, t.posture_assistant.clone()),
        ]
        .into_iter()
        .map(|(posture, label)| {
            let value = posture.as_id().to_owned();
            let selected = posture == active;
            view! { <option value=value selected=selected>{label}</option> }
        })
        .collect_view()
    };

    let pick_scope = move |value: String| {
        let all = corpus_choices(&sources.get_untracked());
        let corpus = all.into_iter().find(|s| scope_value(s) == value);
        let mut next = selection.get_untracked();
        next.corpus = corpus;
        on_select.run(next);
    };

    view! {
        <aside
            class="lds-aichat-knowledge-rail flex flex-col gap-3 rounded-box border border-base-300 bg-base-100 p-4"
            data-ai-chat-knowledge-rail=""
        >
            <h3 class="text-sm font-semibold">{move || texts.get().grounded}</h3>
            <label class="flex flex-col gap-1 text-xs" for=scope_id>
                <span class="opacity-60" data-ai-chat-rail-label="corpus-scope">
                    {move || texts.get().corpus_scope_label}
                </span>
                <select
                    id=scope_id
                    data-ai-chat-corpus-scope=""
                    class="select select-sm select-bordered w-full"
                    on:change=move |e| pick_scope(event_target_value(&e))
                >
                    {scope_options}
                </select>
            </label>
            <label class="flex flex-col gap-1 text-xs" for=mode_id>
                <span class="opacity-60" data-ai-chat-rail-label="query-mode">
                    {move || texts.get().query_mode_label}
                </span>
                <select
                    id=mode_id
                    data-ai-chat-query-mode=""
                    class="select select-sm select-bordered w-full"
                    on:change=move |e| {
                        if let Some(mode) = CorpusQueryMode::parse(&event_target_value(&e)) {
                            let mut next = selection.get_untracked();
                            next.query_mode = mode;
                            on_select.run(next);
                        }
                    }
                >
                    {mode_options}
                </select>
            </label>
            <label class="flex flex-col gap-1 text-xs" for=posture_id>
                <span class="opacity-60" data-ai-chat-rail-label="posture">
                    {move || texts.get().posture_label}
                </span>
                <select
                    id=posture_id
                    data-ai-chat-posture=""
                    class="select select-sm select-bordered w-full"
                    on:change=move |e| {
                        if let Some(posture) = ChatPosture::parse(&event_target_value(&e)) {
                            let mut next = selection.get_untracked();
                            next.posture = posture;
                            on_select.run(next);
                        }
                    }
                >
                    {posture_options}
                </select>
            </label>
        // P6: ingest phase + reindex per corpus, from
        // `KnowledgeSource::Corpus { ingest, .. }`.
        // P6: memory use/capture toggles, from `KnowledgeSource::MemoryStore`.
        // P6: office-knowledge scope toggles, from
        // `KnowledgeSource::OfficeKnowledge`.
        // P6: the recall receipt and the remember flow.
        </aside>
    }
}
