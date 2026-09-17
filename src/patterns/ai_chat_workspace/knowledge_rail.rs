//! `AiChatWorkspace`'s left rail: which corpus a turn is grounded in, how it
//! is queried, whether the engine may fall back to general assistance, how
//! each corpus is indexed, and the actor's governed memory store.
//!
//! Three naming decisions here are deliberate and cost nothing to restate:
//!
//! * The ingest rows are addressed by `data-ai-chat-corpus`, carrying
//!   [`scope_value`], NOT by `data-ai-chat-corpus-scope`. The select already
//!   owns that hook, and [`CorpusScope::as_id`] collapses every folder to
//!   `"folder"` — so a shared hook would address neither the control nor a
//!   particular row. The phase itself is on `data-ai-chat-ingest-phase`,
//!   which is what a proof polls.
//! * A candidate memory entry ALWAYS carries
//!   `data-ai-chat-kb-awaiting-curation`. A remembered item is written and
//!   is not findable by recall until a curator confirms it, and a list that
//!   showed the entry without saying so would read as "saved and searchable"
//!   — the exact misreading the real memory service's `proposed` status
//!   produces in a write-then-read-back demo.
//! * Nothing here says or implies that anything EXTRACTS memories. Every
//!   entry in this rail was written by an explicit action.

use leptos::prelude::*;

use super::knowledge::{
    ChatPosture, CorpusQueryMode, CorpusScope, IngestPhase, IngestStatus, KnowledgeSelection,
    KnowledgeSource, MemoryDraft, MemoryRefusal, OfficeKnowledgeScope, RecallReceipt,
};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_assistant_workspace::{
    AssistantMemory, AssistantMemoryEntry, MemoryClass, MemoryState,
};

/// The memory classes the remember control offers, in display order.
///
/// The two curated classes are offered ON PURPOSE. The real memory service
/// refuses `lesson` and `principle` on write — they are distilled by a
/// curator out of accepted items — and hiding them would turn a reproducible,
/// explainable refusal into a capability the actor never learns exists. The
/// refusal is the feature.
pub fn memory_class_choices() -> Vec<MemoryClass> {
    vec![
        MemoryClass::Tone,
        MemoryClass::Language,
        MemoryClass::Verbosity,
        MemoryClass::WorkingMethod,
        MemoryClass::Unknown("lesson".to_owned()),
        MemoryClass::Unknown("principle".to_owned()),
    ]
}

/// Live state for the rail's two write flows, held as separate signals so a
/// control writes its own field without rewriting the others. Shaped like
/// `super::settings_rows::TuningDraft`.
///
/// The composite owns it (not the rail) because the composite is what learns
/// whether a `remember` was ACCEPTED, and a refused draft must survive: an
/// actor who typed three sentences and tripped the matter-number guardrail
/// gets to edit them, not retype them.
#[derive(Clone, Copy)]
pub struct KnowledgeDraft {
    /// The recall query box.
    pub recall_query: RwSignal<String>,
    /// The proposed memory's class.
    pub remember_kind: RwSignal<MemoryClass>,
    /// The proposed memory's text.
    pub remember_body: RwSignal<String>,
}

impl Default for KnowledgeDraft {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeDraft {
    /// An empty draft.
    pub fn new() -> Self {
        Self {
            recall_query: RwSignal::new(String::new()),
            remember_kind: RwSignal::new(MemoryClass::Tone),
            remember_body: RwSignal::new(String::new()),
        }
    }

    /// The typed draft the backend is asked to remember.
    ///
    /// The title is derived from the body's first line rather than collected
    /// separately: [`MemoryDraft`] carries both, but a second input would be
    /// a second thing to translate, label and prove for a field the seeded
    /// backend renders through the body anyway. A host that wants a distinct
    /// title sets it on the draft this returns.
    pub fn to_memory_draft(&self, scope: &str) -> MemoryDraft {
        let body = self.remember_body.get_untracked();
        let title: String = body.trim().lines().next().unwrap_or_default().to_owned();
        MemoryDraft {
            kind: self.remember_kind.get_untracked(),
            title,
            body,
            scope: scope.to_owned(),
        }
    }
}

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

/// Every published corpus with its ingest status, for the phase list.
pub fn corpus_ingests(sources: &[KnowledgeSource]) -> Vec<(CorpusScope, IngestStatus)> {
    sources
        .iter()
        .filter_map(|s| match s {
            KnowledgeSource::Corpus { scope, ingest, .. } => Some((scope.clone(), ingest.clone())),
            _ => None,
        })
        .collect()
}

/// Whether any published corpus is mid-ingest, which is what the composite
/// polls on.
///
/// `Idle` is deliberately NOT mid-ingest: nothing has been requested, so a
/// poll loop keyed on it would never stop.
pub fn any_ingest_in_flight(sources: &[KnowledgeSource]) -> bool {
    corpus_ingests(sources).iter().any(|(_, status)| {
        matches!(
            status.phase,
            IngestPhase::Walking | IngestPhase::Indexing | IngestPhase::Clustering
        )
    })
}

/// The memory store's use/capture flags, when the host published one.
pub fn memory_flags(sources: &[KnowledgeSource]) -> Option<(bool, bool)> {
    sources.iter().find_map(|s| match s {
        KnowledgeSource::MemoryStore {
            enabled,
            capture_enabled,
        } => Some((*enabled, *capture_enabled)),
        _ => None,
    })
}

/// The actor's own memory collection, when the host published one.
pub fn personal_memory(sources: &[KnowledgeSource]) -> Option<AssistantMemory> {
    sources.iter().find_map(|s| match s {
        KnowledgeSource::PersonalMemory(m) => Some(m.clone()),
        _ => None,
    })
}

/// Every curated organizational-knowledge collection the host published.
pub fn office_collections(
    sources: &[KnowledgeSource],
) -> Vec<(
    OfficeKnowledgeScope,
    Vec<crate::components::ai_assistant_workspace::AssistantKnowledgeEntry>,
)> {
    sources
        .iter()
        .filter_map(|s| match s {
            KnowledgeSource::OfficeKnowledge { scope, entries } => {
                Some((scope.clone(), entries.clone()))
            }
            _ => None,
        })
        .collect()
}

/// The published sources with one corpus's ingest status replaced.
///
/// The composite applies `reindex`'s OWN returned status this way instead of
/// re-reading `knowledge()`, and the difference is visible: the fixture
/// advances the phase ladder one step per `knowledge()` call, so refreshing
/// immediately after a reindex would render `indexing` first and the
/// `walking` phase would never exist on screen. A ladder whose first rung is
/// unobservable is a ladder a proof cannot check.
pub fn with_ingest(
    sources: Vec<KnowledgeSource>,
    scope: &CorpusScope,
    status: IngestStatus,
) -> Vec<KnowledgeSource> {
    sources
        .into_iter()
        .map(|source| match source {
            KnowledgeSource::Corpus {
                scope: s,
                query_mode,
                posture,
                ingest,
            } => {
                let replace = &s == scope;
                KnowledgeSource::Corpus {
                    scope: s,
                    ingest: if replace { status.clone() } else { ingest },
                    query_mode,
                    posture,
                }
            }
            other => other,
        })
        .collect()
}

/// Whether one memory entry is still awaiting a curator.
///
/// Pure, and named, because it is the single condition the curation-gap
/// honesty row is keyed to — the rail must never decide it inline twice and
/// get one of them wrong.
pub fn awaits_curation(entry: &AssistantMemoryEntry) -> bool {
    entry.state == MemoryState::Candidate
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
    /// Fired with the scope the actor asked to re-index.
    on_reindex: Callback<CorpusScope>,
    /// Fired with the requested use/capture flags, in that order.
    on_memory_flags: Callback<(bool, bool)>,
    /// Fired with the query the actor asked to recall.
    on_recall: Callback<String>,
    /// Fired with the memory the actor asked to write.
    on_remember: Callback<MemoryDraft>,
    /// Fired with an entry id and the state the actor asked for.
    on_memory_state: Callback<(String, MemoryState)>,
    /// The last recall's receipt, or `None` before one has run.
    #[prop(into)]
    receipt: Signal<Option<RecallReceipt>>,
    /// Whether the memory store failed to answer. A typed honesty state:
    /// when this is true the rail renders NO receipt and NO hits, because a
    /// stale receipt beside an unreachable store reads as a fresh result.
    #[prop(into)]
    memory_offline: Signal<bool>,
    /// The last refusal the guardrail or the host returned, or `None`.
    #[prop(into)]
    refusal: Signal<Option<MemoryRefusal>>,
    /// The two write flows' live state.
    draft: KnowledgeDraft,
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
    let memory_use_id = move || prefix.with_value(|p| format!("{p}-memory-enabled"));
    let memory_capture_id = move || prefix.with_value(|p| format!("{p}-capture-enabled"));
    let recall_id = move || prefix.with_value(|p| format!("{p}-recall"));
    let remember_kind_id = move || prefix.with_value(|p| format!("{p}-remember-kind"));
    let remember_id = move || prefix.with_value(|p| format!("{p}-remember"));

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

    let kind_options = move || {
        let t = texts.get();
        let active = draft.remember_kind.get();
        memory_class_choices()
            .into_iter()
            .map(|class| {
                let value = class.as_str().to_owned();
                let selected = class.as_str() == active.as_str();
                let label = t.memory_class_label(&class);
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

    let flags = move || memory_flags(&sources.get()).unwrap_or((false, false));

    let ingest_rows = move || {
        let t = texts.get();
        corpus_ingests(&sources.get())
            .into_iter()
            .map(|(scope, status)| {
                let value = scope_value(&scope);
                let label = scope_label(&scope, &t);
                let phase_id = status.phase.as_id();
                let phase_label = t.ingest_phase_label(&status.phase);
                let counts = t.ingest_counts_line(&status);
                let as_of = status.as_of.clone();
                // Present ONLY for `Failed`, and carrying the host's own
                // message: a hook that existed on every row with an empty
                // value would make "is this corpus broken?" unanswerable.
                let error = match &status.phase {
                    IngestPhase::Failed(reason) => Some(reason.message.clone()),
                    _ => None,
                };
                let reindex_scope = scope.clone();
                view! {
                    <li
                        class="flex flex-col gap-1 rounded-box border border-base-300 p-3"
                        data-ai-chat-corpus=value.clone()
                        data-ai-chat-ingest-phase=phase_id
                    >
                        <span class="text-xs font-semibold">{label}</span>
                        <span class="text-xs opacity-70">{phase_label}</span>
                        <span class="text-xs opacity-60">{counts}</span>
                        <span class="text-xs opacity-60">{as_of}</span>
                        {error
                            .map(|message| {
                                view! {
                                    <span class="text-xs text-error" data-ai-chat-ingest-error="">
                                        {message}
                                    </span>
                                }
                            })}
                        <button
                            type="button"
                            class="btn btn-xs"
                            data-ai-chat-reindex=value.clone()
                            on:click=move |_| on_reindex.run(reindex_scope.clone())
                        >
                            {t.reindex.clone()}
                        </button>
                    </li>
                }
            })
            .collect_view()
    };

    let hit_rows = move || {
        let t = texts.get();
        let r = receipt.get()?;
        if r.hits.is_empty() {
            return Some(
                view! {
                    <p class="text-xs opacity-70" data-ai-chat-recall-no-hits="">
                        {t.recall_no_hits.clone()}
                    </p>
                }
                .into_any(),
            );
        }
        Some(
            view! {
                <ul class="flex flex-col gap-3">
                    {r
                        .hits
                        .into_iter()
                        .map(|hit| {
                            let attribution = hit.corpus.as_str().to_owned();
                            let lane = t.recall_corpus_label(&hit.corpus);
                            let score = format!("{:.6}", hit.rrf_score);
                            let rank = hit.rank_in_corpus.to_string();
                            view! {
                                <li
                                    class="flex flex-col gap-1 rounded-box border border-base-300 p-3"
                                    data-ai-chat-memory-hit=""
                                    data-ai-chat-memory-attribution=attribution
                                    data-ai-chat-memory-score=score
                                    data-ai-chat-memory-rank=rank
                                >
                                    <span class="text-xs font-semibold">{hit.title}</span>
                                    <span class="text-xs opacity-70">{hit.snippet}</span>
                                    <span class="text-xs opacity-60">{lane}</span>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            }
            .into_any(),
        )
    };

    let receipt_header = move || {
        let t = texts.get();
        receipt.get().map(|r| {
            view! {
                <div
                    class="flex flex-col gap-1"
                    data-ai-chat-memory-receipt=r.receipt_id.clone()
                >
                    <span class="text-xs font-semibold">
                        {format!("{}: {} \u{b7} {}", t.receipt, r.receipt_id, r.as_of)}
                    </span>
                    // The host's own description of the search that ran,
                    // rendered verbatim. Paraphrasing it would claim a
                    // retrieval strategy this workspace cannot observe.
                    <span class="text-xs opacity-60" data-ai-chat-memory-search="">
                        {r.search}
                    </span>
                </div>
            }
        })
    };

    let memory_rows = move || {
        let t = texts.get();
        let entries = personal_memory(&sources.get())
            .map(|m| m.entries)
            .unwrap_or_default();
        entries
            .into_iter()
            .map(|entry| {
                let id = entry.id.clone();
                let confirm_id = entry.id.clone();
                let withdraw_id = entry.id.clone();
                let kind = entry.class.as_str().to_owned();
                let state = entry.state.as_str().to_owned();
                let state_label = t.memory_state_label(&entry.state);
                let kind_label = t.memory_class_label(&entry.class);
                let awaiting = awaits_curation(&entry);
                let awaiting_copy = t.awaiting_curation.clone();
                view! {
                    <li
                        class="flex flex-col gap-1 rounded-box border border-base-300 p-3"
                        data-ai-chat-kb-entry=id
                        data-ai-chat-kb-kind=kind
                        data-ai-chat-kb-state=state
                    >
                        <span class="text-xs">{entry.text.clone()}</span>
                        <span class="text-xs opacity-60">
                            {format!("{kind_label} \u{b7} {state_label}")}
                        </span>
                        <Show when=move || awaiting>
                            <span
                                class="text-xs opacity-70"
                                data-ai-chat-kb-awaiting-curation=""
                            >
                                {awaiting_copy.clone()}
                            </span>
                        </Show>
                        <div class="flex gap-2">
                            <button
                                type="button"
                                class="btn btn-xs"
                                data-ai-chat-kb-confirm=confirm_id.clone()
                                on:click=move |_| {
                                    on_memory_state
                                        .run((confirm_id.clone(), MemoryState::Confirmed))
                                }
                            >
                                {t.confirm.clone()}
                            </button>
                            <button
                                type="button"
                                class="btn btn-xs"
                                data-ai-chat-kb-withdraw=withdraw_id.clone()
                                on:click=move |_| {
                                    on_memory_state
                                        .run((withdraw_id.clone(), MemoryState::Withdrawn))
                                }
                            >
                                {t.withdraw.clone()}
                            </button>
                        </div>
                    </li>
                }
            })
            .collect_view()
    };

    let office_rows = move || {
        let t = texts.get();
        office_collections(&sources.get())
            .into_iter()
            .map(|(scope, entries)| {
                let scope_id = scope.as_id().to_owned();
                let label = match &scope {
                    OfficeKnowledgeScope::GroupImportant => t.kb_scope_group_important.clone(),
                    OfficeKnowledgeScope::Foundation => t.kb_scope_foundation.clone(),
                    OfficeKnowledgeScope::Unknown(id) => id.clone(),
                };
                view! {
                    <li
                        class="flex flex-col gap-1 rounded-box border border-base-300 p-3"
                        data-ai-chat-kb-scope=scope_id
                    >
                        <span class="text-xs font-semibold">{label}</span>
                        <ul class="flex flex-col gap-1">
                            {entries
                                .into_iter()
                                .map(|entry| {
                                    view! {
                                        <li
                                            class="text-xs opacity-70"
                                            data-ai-chat-kb-office-entry=entry.id.clone()
                                        >
                                            {entry.title.clone()}
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </li>
                }
            })
            .collect_view()
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

            <h3 class="text-sm font-semibold">{move || texts.get().corpus_label}</h3>
            <ul class="flex flex-col gap-3" data-ai-chat-corpus-list="">{ingest_rows}</ul>

            <h3 class="text-sm font-semibold">{move || texts.get().memory_store_label}</h3>
            <label class="flex items-center gap-2 text-xs" for=memory_use_id>
                <input
                    id=memory_use_id
                    type="checkbox"
                    class="checkbox checkbox-sm"
                    data-ai-chat-memory-enabled=""
                    prop:checked=move || flags().0
                    on:change=move |e| {
                        on_memory_flags.run((event_target_checked(&e), flags().1))
                    }
                />
                <span class="opacity-60">{move || texts.get().memory_use}</span>
            </label>
            <label class="flex items-center gap-2 text-xs" for=memory_capture_id>
                <input
                    id=memory_capture_id
                    type="checkbox"
                    class="checkbox checkbox-sm"
                    data-ai-chat-capture-enabled=""
                    prop:checked=move || flags().1
                    on:change=move |e| {
                        on_memory_flags.run((flags().0, event_target_checked(&e)))
                    }
                />
                <span class="opacity-60">{move || texts.get().memory_capture}</span>
            </label>
            <label class="flex flex-col gap-1 text-xs" for=recall_id>
                <span class="opacity-60">{move || texts.get().recall_label}</span>
                <input
                    id=recall_id
                    type="text"
                    class="input input-sm input-bordered w-full"
                    data-ai-chat-recall-input=""
                    prop:value=move || draft.recall_query.get()
                    on:input=move |e| draft.recall_query.set(event_target_value(&e))
                />
            </label>
            <button
                type="button"
                class="btn btn-xs"
                data-ai-chat-recall=""
                on:click=move |_| on_recall.run(draft.recall_query.get_untracked())
            >
                {move || texts.get().recall}
            </button>
            <Show when=move || memory_offline.get()>
                <p class="text-xs text-warning" data-ai-chat-memory-offline="">
                    {move || texts.get().memory_offline}
                </p>
            </Show>
            <Show when=move || !memory_offline.get()>
                {receipt_header}
                {hit_rows}
            </Show>

            <label class="flex flex-col gap-1 text-xs" for=remember_kind_id>
                <span class="opacity-60" data-ai-chat-rail-label="remember-kind">
                    {move || texts.get().remember_kind_label}
                </span>
                <select
                    id=remember_kind_id
                    data-ai-chat-remember-kind=""
                    class="select select-sm select-bordered w-full"
                    on:change=move |e| {
                        draft.remember_kind.set(MemoryClass::parse(&event_target_value(&e)))
                    }
                >
                    {kind_options}
                </select>
            </label>
            <label class="flex flex-col gap-2 text-xs" for=remember_id>
                <span class="opacity-60">{move || texts.get().remember_label}</span>
                <textarea
                    id=remember_id
                    class="textarea textarea-sm textarea-bordered w-full"
                    data-ai-chat-remember-input=""
                    prop:value=move || draft.remember_body.get()
                    on:input=move |e| draft.remember_body.set(event_target_value(&e))
                ></textarea>
            </label>
            <button
                type="button"
                class="btn btn-xs"
                data-ai-chat-remember=""
                on:click=move |_| on_remember.run(draft.to_memory_draft("office"))
            >
                {move || texts.get().remember}
            </button>
            {move || {
                let t = texts.get();
                refusal
                    .get()
                    .map(|r| {
                        // The host's own sentence is rendered for an
                        // `Unknown` reason, NEXT to this table's generic
                        // copy — never in place of the closed id set the
                        // hook carries.
                        let host = match &r {
                            MemoryRefusal::Unknown(reason) => Some(reason.clone()),
                            _ => None,
                        };
                        view! {
                            <p class="text-xs text-error" data-ai-chat-guardrail=r.as_str()>
                                {t.memory_refusal_label(&r)}
                                {host.map(|h| format!(" {h}"))}
                            </p>
                        }
                    })
            }}

            <h3 class="text-sm font-semibold">{move || texts.get().personal_memory_label}</h3>
            <ul class="flex flex-col gap-3" data-ai-chat-kb-list="">{memory_rows}</ul>

            <h3 class="text-sm font-semibold">{move || texts.get().office_knowledge_label}</h3>
            <ul class="flex flex-col gap-3" data-ai-chat-kb-office-list="">{office_rows}</ul>
        </aside>
    }
}
