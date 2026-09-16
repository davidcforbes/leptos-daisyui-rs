//! `AiChatWorkspace`'s right rail: what backed the last turn's answer.
//!
//! This task renders the CITATIONS only. `TurnEvidence` also carries
//! `facts`, `limitations`, `as_of`, `grounding` and the recall receipt, and
//! all of them compile today — they are P6's, and wiring them early would
//! ship an evidence panel nothing proves.

use leptos::prelude::*;

use super::status::{TurnRecord, canceled_partial};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_chat::Citation;

/// The citations backing one turn, or an empty list when the turn produced
/// none (or has not finished).
pub fn citations_of(record: Option<&TurnRecord>) -> Vec<Citation> {
    record
        .and_then(|r| r.evidence.as_ref())
        .map(|e| e.citations.clone())
        .unwrap_or_default()
}

/// The evidence rail.
#[component]
pub fn EvidenceRail(
    /// The turn currently in flight or most recently finished.
    #[prop(into)]
    turn: Signal<Option<TurnRecord>>,
    /// Localized copy.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
) -> impl IntoView {
    let citations = move || citations_of(turn.get().as_ref());
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
        // P6: the recall receipt (`TurnEvidence::recall`).
        // P6: qualified facts (`TurnEvidence::facts`) and their limitations.
        // P6: the grounding verdict (`TurnEvidence::grounding`) and `as_of`.
        </aside>
    }
}
