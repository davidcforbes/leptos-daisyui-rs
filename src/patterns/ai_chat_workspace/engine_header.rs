//! `AiChatWorkspace`'s top rail: which engine is running, whether the turn in
//! flight is still moving, what it has cost, and — for any engine that cannot
//! currently be asked — why.

use leptos::prelude::*;

use super::provider::{ProviderCard, card_ready_for_ask, card_unready_reason};
use super::status::{TurnRecord, lifecycle_id};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_chat::format_usage_subtitle;

/// Engines whose usage is billed per token rather than covered by the plan.
///
/// A fixture-grade policy, deliberately explicit: the header must pick ONE of
/// `UsageTotals`' two `Usage` fields to render, and adding them together would
/// misreport both (see `UsageTotals`' own documentation). A real host knows
/// which of its engines are metered from its plan, not from this list.
pub fn engine_is_metered(engine_id: &str) -> bool {
    engine_id.starts_with("groq") || engine_id == "ollama"
}

/// The header subtitle for one turn's usage, or `None` when the turn reported
/// none.
///
/// `format_usage_subtitle` formats token counts only — it has no cost output
/// at all — so cost is rendered separately by the header, which is also where
/// `cost_incomplete` turns the figure into an explicit floor rather than a
/// total.
pub fn usage_line(record: &TurnRecord, engine_id: &str) -> Option<String> {
    let totals = record.usage.as_ref()?;
    let usage = if engine_is_metered(engine_id) {
        &totals.metered
    } else {
        &totals.plan
    };
    Some(format_usage_subtitle(usage))
}

/// The cost figure for one turn, and whether it is a floor rather than a
/// total. `None` when nothing metered has been priced yet.
pub fn cost_line(record: &TurnRecord) -> Option<(f64, bool)> {
    let totals = record.usage.as_ref()?;
    totals.metered_cost.map(|c| (c, totals.cost_incomplete))
}

/// The workspace's engine picker, turn status and usage line.
#[component]
pub fn EngineHeader(
    /// Every engine the host offers.
    #[prop(into)]
    cards: Signal<Vec<ProviderCard>>,
    /// The engine currently running.
    #[prop(into)]
    engine_id: Signal<String>,
    /// The turn currently in flight or most recently finished.
    #[prop(into)]
    turn: Signal<Option<TurnRecord>>,
    /// The budget line, already localized and formatted by the composite, or
    /// `None` when the host publishes no budget.
    #[prop(into)]
    budget: Signal<Option<String>>,
    /// A refusal the composite is currently honest about (a declined
    /// `open_session`), or `None`.
    #[prop(into)]
    refusal: Signal<Option<String>>,
    /// Notices lifted off the turn record, already localized.
    #[prop(into)]
    notices: Signal<Vec<(&'static str, String)>>,
    /// Localized copy.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
) -> impl IntoView {
    let status_id = move || {
        turn.get()
            .map(|t| lifecycle_id(&t.lifecycle).to_owned())
            .unwrap_or_else(|| "idle".to_owned())
    };
    let status_label = move || {
        let t = texts.get();
        turn.get()
            .map(|r| t.lifecycle_label(&r.lifecycle))
            .unwrap_or_default()
    };
    let usage = move || {
        let id = engine_id.get();
        turn.get().and_then(|r| usage_line(&r, &id))
    };
    let cost = move || {
        let t = texts.get();
        turn.get().and_then(|r| cost_line(&r)).map(|(c, partial)| {
            if partial {
                format!("${c:.4} ({})", t.cost_absent)
            } else {
                format!("${c:.4}")
            }
        })
    };
    let active_label = move || {
        let id = engine_id.get();
        cards
            .get()
            .into_iter()
            .find(|c| c.engine.id == id)
            .map(|c| c.capabilities.label)
            .unwrap_or_default()
    };
    let unready = move || {
        let t = texts.get();
        cards
            .get()
            .into_iter()
            .filter(|c| !card_ready_for_ask(c))
            .filter_map(|c| {
                card_unready_reason(&c).map(|code| {
                    (
                        c.engine.id.clone(),
                        c.picker_label.clone(),
                        code.as_code().to_owned(),
                        t.availability_reason(&code),
                    )
                })
            })
            .collect::<Vec<_>>()
    };

    view! {
        <header
            class="lds-aichat-workspace-header flex flex-col gap-2 rounded-box border border-base-300 bg-base-100 p-4"
            data-ai-chat-workspace-header=""
            data-ai-chat-turn-status=status_id
        >
            <div class="flex flex-wrap items-center justify-between gap-3">
                <span class="text-sm font-semibold" data-ai-chat-engine-label="">
                    {active_label}
                </span>
                <span class="text-xs opacity-60" data-ai-chat-turn-status-label="">
                    {status_label}
                </span>
            </div>
            <div class="flex flex-wrap items-center gap-3 text-xs opacity-70">
                <Show when=move || usage().is_some()>
                    <span data-ai-chat-usage="">{move || usage().unwrap_or_default()}</span>
                </Show>
                <Show when=move || cost().is_some()>
                    <span data-ai-chat-usage-cost="">{move || cost().unwrap_or_default()}</span>
                </Show>
                <Show when=move || budget.get().is_some()>
                    <span data-ai-chat-budget="">{move || budget.get().unwrap_or_default()}</span>
                </Show>
            </div>
            <Show when=move || refusal.get().is_some()>
                <p class="text-xs text-error" role="alert" data-ai-chat-refusal="">
                    {move || refusal.get().unwrap_or_default()}
                </p>
            </Show>
            {move || {
                notices
                    .get()
                    .into_iter()
                    .map(|(kind, body)| {
                        let escalation = (kind == "escalated").then_some("");
                        let truncated = (kind == "truncated").then_some("");
                        view! {
                            <p
                                class="text-xs opacity-70"
                                data-ai-chat-notice=kind
                                data-ai-chat-escalation=escalation
                                data-ai-chat-truncated=truncated
                            >
                                {body}
                            </p>
                        }
                    })
                    .collect_view()
            }}
            <Show when=move || !unready().is_empty()>
                <ul class="flex flex-col gap-1 text-xs opacity-70" data-ai-chat-engine-availability="">
                    {move || {
                        unready()
                            .into_iter()
                            .map(|(id, label, code, reason)| {
                                view! {
                                    <li
                                        data-ai-chat-engine-reason=code
                                        data-ai-chat-engine-reason-for=id
                                    >
                                        {format!("{label}: {reason}")}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
        </header>
    }
}
