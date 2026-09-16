//! `AiChatWorkspace`'s right rail: what backed the last turn's answer.
//!
//! Two properties this rail exists to keep:
//!
//! * A FACT IS NEVER RENDERED WITHOUT ITS QUALIFICATION. `AssistantFact`
//!   cannot be constructed without one (`AssistantFact::new` rejects an empty
//!   qualification, and `TurnEvidence::with_facts` fails closed on the whole
//!   list rather than dropping one), so the rail simply renders what the type
//!   guarantees — and the browser proof asserts the document contains no
//!   `[data-ai-chat-fact]` lacking a non-empty
//!   `data-ai-chat-fact-qualification`. A number beside no caveat is a
//!   stronger claim than the host made.
//! * The GROUNDING VERDICT is separate from the outcome. A turn that answered
//!   and a turn that was grounded are two questions: `AnswerOutcome` says
//!   whether an answer was produced, `GroundingVerdict` says whether anything
//!   backed it. `not_found` is the honest middle — a search ran and found
//!   nothing — and it is rendered as its own value, not as an empty
//!   citations list that looks identical to "no search ran".
//! * An ASSISTANT-ONLY VERDICT IS A STRUCTURAL GATE, not a courtesy. A turn
//!   whose verdict reads "General assistance, no documents" must never show
//!   grounded citations or qualified facts, so [`citations_of`] and
//!   [`facts_of`] return empty lists for `GroundingVerdict::AssistantOnly`
//!   even when the `TurnEvidence` itself carries them — a host that reports
//!   both sides of that contradiction is claiming sources it said it did not
//!   read. The fixture transport additionally declines to produce such a
//!   pair, but that fixture-side courtesy is a second layer, not the gate.

use leptos::prelude::*;

use super::knowledge::KnowledgeSelection;
use super::status::{TurnRecord, canceled_partial};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_assistant_workspace::AssistantFact;
use crate::components::ai_chat::Citation;

use super::evidence::{GroundingVerdict, TurnEvidence};

/// The citations backing one turn, or an empty list when the turn produced
/// none (or has not finished).
///
/// Gated on the turn's own grounding verdict rather than read verbatim: an
/// `AssistantOnly` turn is one that consulted no corpus, so rendering the
/// citations a host attached anyway would present sources the verdict says
/// were never read.
pub fn citations_of(record: Option<&TurnRecord>) -> Vec<Citation> {
    match record.and_then(|r| r.evidence.as_ref()) {
        Some(e) if e.grounding != GroundingVerdict::AssistantOnly => e.citations.clone(),
        _ => Vec::new(),
    }
}

/// The evidence one turn carries, or `None` when the turn has not produced
/// any yet.
pub fn evidence_of(record: Option<&TurnRecord>) -> Option<TurnEvidence> {
    record.and_then(|r| r.evidence.clone())
}

/// The qualified facts backing one turn.
///
/// Gated on the turn's own grounding verdict for the same reason
/// [`citations_of`] is: a fact beside an `AssistantOnly` verdict is a
/// qualified claim about a source the turn never consulted.
pub fn facts_of(record: Option<&TurnRecord>) -> Vec<AssistantFact> {
    match record.and_then(|r| r.evidence.as_ref()) {
        Some(e) if e.grounding != GroundingVerdict::AssistantOnly => e.facts.clone(),
        _ => Vec::new(),
    }
}

/// The evidence rail.
#[component]
pub fn EvidenceRail(
    /// The turn currently in flight or most recently finished.
    #[prop(into)]
    turn: Signal<Option<TurnRecord>>,
    /// The knowledge mix the current session was opened against, so the rail
    /// can attribute HOW the corpus was queried. Read from the selection
    /// rather than from the turn because the query mode is a property of the
    /// session, and a turn that never reached a corpus still ran under one.
    #[prop(into)]
    selection: Signal<KnowledgeSelection>,
    /// Localized copy.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
) -> impl IntoView {
    let citations = move || citations_of(turn.get().as_ref());
    let evidence = move || evidence_of(turn.get().as_ref());
    // A canceled turn's surviving prefix, presented as a PARTIAL — never as
    // an answer. `TurnRecord::outcome` carries `Answered { .. }` for a
    // canceled turn too (empty when the cancel discarded it), so reading that
    // field without the lifecycle gate renders a stopped turn as a finished
    // one. `canceled_partial` is that gate.
    let partial = move || {
        turn.get()
            .as_ref()
            .and_then(|r| canceled_partial(r).map(str::to_owned))
    };

    let grounding = move || {
        let t = texts.get();
        evidence().map(|e| {
            let id = e.grounding.as_id();
            let label = t.grounding_label(&e.grounding);
            view! {
                <p class="text-xs font-semibold" data-ai-chat-grounding=id>
                    {label}
                </p>
            }
        })
    };

    let facts = move || {
        let t = texts.get();
        let rows = facts_of(turn.get().as_ref());
        if rows.is_empty() {
            return None;
        }
        Some(view! {
            <div class="flex flex-col gap-2">
                <h4 class="text-xs font-semibold">{t.facts_label.clone()}</h4>
                <ul class="flex flex-col gap-2">
                    {rows
                        .into_iter()
                        .map(|fact| {
                            let label = fact.label().to_owned();
                            let qualification = fact.qualification().to_owned();
                            let current = fact.current().map(str::to_owned);
                            let availability = fact.availability().map(str::to_owned);
                            view! {
                                <li
                                    class="flex flex-col gap-1"
                                    data-ai-chat-fact=""
                                    data-ai-chat-fact-qualification=qualification.clone()
                                >
                                    <span class="text-xs font-semibold">{label}</span>
                                    {current.map(|v| view! { <span class="text-xs">{v}</span> })}
                                    {availability
                                        .map(|a| {
                                            view! { <span class="text-xs opacity-60">{a}</span> }
                                        })}
                                    <span class="text-xs opacity-70">
                                        {qualification.clone()}
                                    </span>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        })
    };

    let limitations = move || {
        let t = texts.get();
        let rows = evidence().map(|e| e.limitations).unwrap_or_default();
        if rows.is_empty() {
            return None;
        }
        Some(view! {
            <div class="flex flex-col gap-2">
                <h4 class="text-xs font-semibold">{t.limitations_label.clone()}</h4>
                <ul class="flex flex-col gap-1">
                    {rows
                        .into_iter()
                        .map(|text| {
                            view! {
                                <li class="text-xs opacity-70" data-ai-chat-limitation="">
                                    {text}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        })
    };

    let recall_receipt = move || {
        let t = texts.get();
        evidence().and_then(|e| e.recall).map(|r| {
            view! {
                <p
                    class="text-xs opacity-60"
                    data-ai-chat-turn-receipt=r.receipt_id.clone()
                >
                    {format!("{}: {}", t.receipt, r.receipt_id)}
                </p>
            }
        })
    };

    let as_of = move || {
        let t = texts.get();
        evidence().and_then(|e| e.as_of).map(|stamp| {
            view! {
                <p class="text-xs opacity-60" data-ai-chat-evidence-as-of=stamp.clone()>
                    {format!("{}: {stamp}", t.as_of_label)}
                </p>
            }
        })
    };

    view! {
        <aside
            class="lds-aichat-evidence-rail flex flex-col gap-3 rounded-box border border-base-300 bg-base-100 p-4"
            data-ai-chat-evidence-rail=""
        >
            <h3 class="text-sm font-semibold">{move || texts.get().receipt}</h3>
            <Show when=move || partial().is_some()>
                <p class="text-xs opacity-70" data-ai-chat-canceled-partial="">
                    {move || {
                        format!(
                            "{}: {}",
                            texts.get().state_canceled_kept,
                            partial().unwrap_or_default(),
                        )
                    }}
                </p>
            </Show>
            {grounding}
            <p
                class="text-xs opacity-60"
                data-ai-chat-query-attribution=move || {
                    selection.get().query_mode.as_id()
                }
            >
                {move || {
                    let t = texts.get();
                    format!(
                        "{}: {}",
                        t.query_attribution_label,
                        t.query_mode_name(selection.get().query_mode),
                    )
                }}
            </p>
            <ul class="flex flex-col gap-2 text-xs">
                {move || {
                    citations()
                        .into_iter()
                        .map(|c| {
                            let href = c.href.clone().unwrap_or_default();
                            let label = c.label.clone();
                            view! {
                                <li data-ai-chat-citation=href.clone()>
                                    <span class="opacity-70">{label}</span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
            {facts}
            {limitations}
            {recall_receipt}
            {as_of}
        </aside>
    }
}
