//! `AiChatWorkspace`'s top rail: which engine is running, whether the turn in
//! flight is still moving, what it has cost, what it produced — and, for
//! anything that is refused, broken or merely not enabled, an honest account
//! of which of those it is.
//!
//! The three states this file is most careful to keep apart are *not
//! enabled* (the account or the host says no, and nothing was attempted),
//! *unavailable* (something was attempted and refused to run) and *failed*
//! (something was attempted and broke). They have different hook values,
//! different tones and different copy, because an actor's next move differs
//! in each case and a single "error" badge would erase that.

use leptos::prelude::*;

use super::backend::WorkspaceRefusal;
use super::provider::{
    AvailabilityReasonCode, ProviderCard, card_ready_for_ask, card_unready_reason,
};
use super::status::{TurnRecord, declined_limitations, failure_kind, lifecycle_id, outcome_id};
use super::texts::AiChatWorkspaceTexts;
use crate::components::ai_assistant_workspace::{AttemptLifecycle, RefusalNextAction};
use crate::components::ai_chat::format_usage_subtitle;

/// The failure kind a watchdog-failed turn reports.
///
/// Not an [`AvailabilityReasonCode`]: nothing about the engine was reported
/// unavailable — the host simply never finished, and this workspace stopped
/// waiting. It is minted here rather than derived from the record because
/// the record cannot know: `fail_turn` acts on the `ChatSession`, so a
/// stalled turn's own lifecycle stays `Running` forever.
pub const WATCHDOG_FAILURE_KIND: &str = "watchdog";

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

/// The four usage numbers the header publishes separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UsageFigures {
    /// Tokens covered by the actor's plan (input plus output).
    pub plan_tokens: u64,
    /// Tokens billed per use (input plus output).
    pub metered_tokens: u64,
    /// Reasoning tokens this turn spent.
    pub reasoning_tokens: u64,
    /// Output tokens this turn produced.
    pub output_tokens: u64,
}

/// One turn's usage, split the way an actor actually reads it.
///
/// Two rules are encoded here, and they pull in opposite directions:
///
/// * **Plan and metered are never summed.** They answer different questions
///   ("how much of what I already pay for did this use?" and "what will I be
///   billed?"), so they stay two numbers behind two hooks forever — see
///   [`super::status::UsageTotals`]' own documentation.
/// * **Reasoning and output are turn totals**, added across both buckets.
///   They are counts of one KIND of token, not two kinds of money, and the
///   plan/metered split is still fully visible in the two fields above. The
///   alternative — reporting only the bucket the engine's billing mode
///   picks — renders `output: 0` on a turn that plainly produced an answer
///   whenever a host reports that turn on the other side of the split, which
///   is precisely the kind of quiet lie this phase exists to remove.
///
/// `reasoning_tokens <= output_tokens` holds for every turn a sane host
/// reports, and a renderer may present it as such; this function does not
/// enforce it, because silently clamping a host's number would hide the bug
/// rather than show it.
pub fn usage_figures(record: &TurnRecord) -> Option<UsageFigures> {
    let t = record.usage.as_ref()?;
    Some(UsageFigures {
        plan_tokens: t.plan.input_tokens + t.plan.output_tokens,
        metered_tokens: t.metered.input_tokens + t.metered.output_tokens,
        reasoning_tokens: t.plan.reasoning_tokens + t.metered.reasoning_tokens,
        output_tokens: t.plan.output_tokens + t.metered.output_tokens,
    })
}

/// The honesty state one availability reason puts the workspace in.
///
/// The mapping exists so the strip answers "what kind of problem is this?"
/// rather than repeating the code: a missing key, an expired sign-in and a
/// never-signed-in engine are three different codes but three different
/// FIXES too, so they keep three different states. Everything the actor
/// cannot act on directly — a missing CLI, an unsupported protocol, a
/// runtime that is not running, a model that is not installed, a busy
/// engine — lands on `unavailable`, and everything the ACCOUNT withholds
/// lands on `not_enabled`.
pub fn honesty_state_for_code(code: &AvailabilityReasonCode) -> &'static str {
    match code {
        AvailabilityReasonCode::TierEffectsDisabled | AvailabilityReasonCode::NotArmed => {
            "not_enabled"
        }
        AvailabilityReasonCode::CredentialKeyUnavailable => "key_missing",
        AvailabilityReasonCode::NotSignedIn => "signed_out",
        AvailabilityReasonCode::SignInExpired => "expired",
        AvailabilityReasonCode::BudgetExhausted => "budget_exhausted",
        _ => "unavailable",
    }
}

/// The honesty state and reason code the header should publish.
///
/// Precedence, and every step of it matters:
///
/// 1. **A denied tier outranks everything.** The account says no, so what
///    any individual engine reports about itself is moot — and reporting a
///    per-engine reason here would send the actor to fix the wrong thing.
/// 2. **A refused active engine** is next: it is why nothing can be asked
///    right now.
/// 3. **The watchdog** is a `failed`, not an `unavailable`: we tried, and
///    the attempt never came back.
/// 4. **The live turn's own terminal failure**, typed from its reason code.
/// 5. Otherwise `ready` — published positively, so the strip's silence can
///    never be confused with a check that was skipped.
pub fn honesty_for(
    tier_denied: bool,
    active_reason: Option<&AvailabilityReasonCode>,
    record: Option<&TurnRecord>,
    timed_out: bool,
) -> (&'static str, Option<String>) {
    if tier_denied {
        return (
            "not_enabled",
            Some(
                AvailabilityReasonCode::TierEffectsDisabled
                    .as_code()
                    .to_owned(),
            ),
        );
    }
    if let Some(code) = active_reason {
        return (
            honesty_state_for_code(code),
            Some(code.as_code().to_owned()),
        );
    }
    if timed_out {
        return ("failed", Some(WATCHDOG_FAILURE_KIND.to_owned()));
    }
    match record.map(|r| &r.lifecycle) {
        Some(AttemptLifecycle::Failed { reason }) => ("failed", Some(reason.code.clone())),
        Some(AttemptLifecycle::Unavailable { reason }) => {
            ("unavailable", Some(reason.code.clone()))
        }
        _ => ("ready", None),
    }
}

/// The daisyUI badge classes one honesty state renders in.
///
/// `not_enabled` and `failed` deliberately differ here as well as in their
/// hook value: a proof that only compared hook strings could pass while both
/// states looked identical to a human, which is the failure this phase is
/// about.
pub fn honesty_tone(state: &str) -> &'static str {
    match state {
        "ready" => "badge badge-sm badge-success",
        "failed" | "unavailable" => "badge badge-sm badge-error",
        _ => "badge badge-sm badge-warning",
    }
}

/// What the turn produced, with the watchdog taking precedence.
///
/// `ChatSession::fail_turn` reaches the transport through `cancel()`, so the
/// host's record of a watchdog-failed turn comes back CANCELED — which would
/// otherwise render as "the actor stopped this", the one thing that did not
/// happen. The watchdog owns the outcome whenever it fired.
pub fn header_outcome_id(record: Option<&TurnRecord>, timed_out: bool) -> Option<&'static str> {
    if timed_out {
        return Some("failed");
    }
    record.and_then(outcome_id)
}

/// Whether a cancel discarded its partial, or `None` when this turn was not
/// canceled BY THE ACTOR.
///
/// Same reason as [`header_outcome_id`]: a watchdog failure arrives at the
/// backend as a cancel, and publishing `cancel-discarded="false"` for it
/// would tell a proof — and an actor — that someone pressed Stop.
pub fn header_cancel_discarded(record: Option<&TurnRecord>, timed_out: bool) -> Option<bool> {
    if timed_out {
        return None;
    }
    match record.map(|r| &r.lifecycle) {
        Some(AttemptLifecycle::Canceled { discarded }) => Some(*discarded),
        _ => None,
    }
}

/// The failure kind to publish, watchdog included.
pub fn header_failure_kind(record: Option<&TurnRecord>, timed_out: bool) -> Option<String> {
    if timed_out {
        return Some(WATCHDOG_FAILURE_KIND.to_owned());
    }
    record.and_then(|r| failure_kind(r).map(str::to_owned))
}

/// The workspace's engine status, honesty strip, usage line and refusal.
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
    /// A refusal the composite is currently honest about, or `None`.
    #[prop(into)]
    refusal: Signal<Option<WorkspaceRefusal>>,
    /// Whether the actor's reasoning tier denies every effect. When true no
    /// card may be asked, whatever the card itself reports.
    #[prop(into)]
    tier_denied: Signal<bool>,
    /// Whether the watchdog has failed the live turn.
    #[prop(into)]
    timed_out: Signal<bool>,
    /// Notices lifted off the turn record, already localized.
    #[prop(into)]
    notices: Signal<Vec<(&'static str, String)>>,
    /// Localized copy.
    #[prop(into)]
    texts: Signal<AiChatWorkspaceTexts>,
    /// Invoked when the actor presses the refusal's single next-step button.
    #[prop(into)]
    on_refusal_action: Callback<RefusalNextAction>,
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
    let figures = move || turn.get().as_ref().and_then(usage_figures);
    let tps = move || turn.get().and_then(|r| r.tokens_per_sec);
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
    let active_reason = move || {
        let id = engine_id.get();
        cards
            .get()
            .into_iter()
            .find(|c| c.engine.id == id)
            .and_then(|c| card_unready_reason(&c))
    };
    let honesty = move || {
        honesty_for(
            tier_denied.get(),
            active_reason().as_ref(),
            turn.get().as_ref(),
            timed_out.get(),
        )
    };
    let honesty_copy = move || {
        let t = texts.get();
        let (state, code) = honesty();
        match code.as_deref() {
            // The watchdog is not an availability reason, so it has no entry
            // in that table; routing it through one would print the bare
            // code `watchdog` at the actor.
            Some(WATCHDOG_FAILURE_KIND) => t.turn_timed_out,
            Some(code) => t.availability_reason(&AvailabilityReasonCode::parse(code)),
            None if state == "ready" => t.honesty_ready,
            None => t.unavailable,
        }
    };
    let failure = move || header_failure_kind(turn.get().as_ref(), timed_out.get());
    let outcome = move || header_outcome_id(turn.get().as_ref(), timed_out.get());
    let cancel_discarded = move || {
        header_cancel_discarded(turn.get().as_ref(), timed_out.get()).map(|d| d.to_string())
    };
    let limitations = move || {
        turn.get()
            .as_ref()
            .map(|r| declined_limitations(r).to_vec())
            .unwrap_or_default()
    };
    // Every card, with its own reason, whenever the TIER denies the account:
    // a per-card availability list that showed only individually-broken
    // engines would leave a denied actor reading a healthy-looking picker.
    let unready = move || {
        let t = texts.get();
        let denied = tier_denied.get();
        cards
            .get()
            .into_iter()
            .filter(|c| denied || !card_ready_for_ask(c))
            .filter_map(|c| {
                let code = if denied {
                    Some(AvailabilityReasonCode::TierEffectsDisabled)
                } else {
                    card_unready_reason(&c)
                };
                code.map(|code| {
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
            data-ai-chat-honesty=move || honesty().0
            data-ai-chat-honesty-reason=move || honesty().1
            data-ai-chat-tier-verdict=move || if tier_denied.get() { "denied" } else { "allowed" }
            data-ai-chat-failure=failure
            data-ai-chat-outcome=outcome
            data-ai-chat-cancel-discarded=cancel_discarded
        >
            <div class="flex flex-wrap items-center justify-between gap-3">
                <span class="text-sm font-semibold" data-ai-chat-engine-label="">
                    {active_label}
                </span>
                <span class="text-xs opacity-60" data-ai-chat-turn-status-label="">
                    {status_label}
                </span>
            </div>
            <p class=move || honesty_tone(honesty().0) data-ai-chat-honesty-text="">
                {honesty_copy}
            </p>
            <div class="flex flex-wrap items-center gap-3 text-xs opacity-70">
                <Show when=move || usage().is_some()>
                    <span data-ai-chat-usage="">{move || usage().unwrap_or_default()}</span>
                </Show>
                {move || {
                    let t = texts.get();
                    figures()
                        .map(|f| {
                            view! {
                                <span data-ai-chat-usage-plan=f.plan_tokens.to_string()>
                                    {format!("{}: {}", t.usage_plan, f.plan_tokens)}
                                </span>
                                <span data-ai-chat-usage-metered=f.metered_tokens.to_string()>
                                    {format!("{}: {}", t.usage_metered, f.metered_tokens)}
                                </span>
                                <span data-ai-chat-usage-reasoning=f.reasoning_tokens.to_string()>
                                    {f.reasoning_tokens}
                                </span>
                                <span data-ai-chat-usage-output=f.output_tokens.to_string()>
                                    {f.output_tokens}
                                </span>
                            }
                        })
                }}
                {move || {
                    let t = texts.get();
                    tps()
                        .map(|v| {
                            view! {
                                <span data-ai-chat-usage-tps=format!("{v:.2}")>
                                    {format!("{}: {v:.2}", t.tokens_per_sec)}
                                </span>
                            }
                        })
                }}
                <Show when=move || cost().is_some()>
                    <span data-ai-chat-usage-cost="">{move || cost().unwrap_or_default()}</span>
                </Show>
                <Show when=move || budget.get().is_some()>
                    <span data-ai-chat-usage-budget="">{move || budget.get().unwrap_or_default()}</span>
                </Show>
            </div>
            <Show when=move || !limitations().is_empty()>
                <div class="flex flex-col gap-1 text-xs" data-ai-chat-limitations="">
                    <span class="font-semibold">{move || texts.get().limitations_label}</span>
                    <ul class="flex flex-col gap-1 opacity-70">
                        {move || {
                            limitations()
                                .into_iter()
                                .map(|l| view! { <li data-ai-chat-limitation="">{l}</li> })
                                .collect_view()
                        }}
                    </ul>
                </div>
            </Show>
            {move || {
                let t = texts.get();
                refusal
                    .get()
                    .map(|r| {
                        let action = r.next_action();
                        let label = match action {
                            RefusalNextAction::RetryLater => t.action_retry_later,
                            RefusalNextAction::OpenSettings => t.action_open_settings,
                            RefusalNextAction::NewConversation => t.action_new_conversation,
                        };
                        view! {
                            <div
                                class="flex flex-wrap items-center gap-3 text-xs"
                                role="alert"
                                data-ai-chat-refusal=r.kind.as_str()
                                data-ai-chat-refusal-reason=r
                                    .code
                                    .as_ref()
                                    .map(|c| c.as_code().to_owned())
                            >
                                <span class="text-error">{r.message.clone()}</span>
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs"
                                    data-ai-chat-refusal-action=action.as_str()
                                    on:click=move |_| on_refusal_action.run(action)
                                >
                                    {label}
                                </button>
                            </div>
                        }
                    })
            }}
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
